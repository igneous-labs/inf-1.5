use std::borrow::Borrow;

#[derive(Debug, Clone, Copy)]
pub enum ListChange<D, T> {
    Diff(D),
    Add(T),
    Del,
}

pub(crate) fn assert_list_changes<D, T: core::fmt::Debug + PartialEq>(
    changes: impl IntoIterator<Item = impl Borrow<ListChange<D, T>>>,
    bef: impl IntoIterator<Item = impl Borrow<T>>,
    aft: impl IntoIterator<Item = impl Borrow<T>>,
    assert_diff: impl Fn(&D, &T, &T),
) {
    let changes = changes.into_iter();
    let mut bef = bef.into_iter();
    let mut aft = aft.into_iter();
    changes.for_each(|change| match change.borrow() {
        ListChange::Diff(d) => {
            assert_diff(
                d,
                bef.next().unwrap().borrow(),
                aft.next().unwrap().borrow(),
            );
        }
        ListChange::Add(s) => {
            assert_eq!(s, aft.next().unwrap().borrow());
        }
        ListChange::Del => {
            bef.next().unwrap();
        }
    });
    if bef.next().is_some() {
        panic!("bef not exhausted, probably missing deletion");
    }
    if aft.next().is_some() {
        panic!("aft not exhausted, probably missing addition");
    }
}

#[derive(Debug)]
pub struct ListChanges<'a, D, T> {
    pub list: &'a [T],
    pub changes: Vec<ListChange<D, T>>,
}

impl<'a, D: Default, T> ListChanges<'a, D, T> {
    /// Default should be `Diff::NoChange` for all fields
    pub fn new(list: &'a [T]) -> Self {
        Self {
            list,
            changes: list
                .iter()
                .map(|_| ListChange::Diff(D::default()))
                .collect(),
        }
    }
}

impl<D, T> ListChanges<'_, D, T> {
    pub fn with_push(self, new: T) -> Self {
        let Self { list, mut changes } = self;
        changes.push(ListChange::Add(new));
        Self { list, changes }
    }

    pub fn with_del(self, idx: usize) -> Self {
        let Self { list, mut changes } = self;
        changes[idx] = ListChange::Del;
        Self { list, changes }
    }

    pub fn with_diff(self, idx: usize, diff: D) -> Self {
        let Self { list, mut changes } = self;
        changes[idx] = ListChange::Diff(diff);
        Self { list, changes }
    }

    pub fn build(self) -> Vec<ListChange<D, T>> {
        let Self { list: _, changes } = self;
        changes
    }
}
