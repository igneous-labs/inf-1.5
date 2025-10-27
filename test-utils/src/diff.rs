#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Diff<T> {
    /// assert that val did not change between before and after
    #[default]
    Unchanged,

    /// assert that val changed from old value of `.0` to new value of `.1`
    Changed(T, T),

    /// assert that val changed from old value of `.0` to new value of `.1`
    /// where old value != new value
    StrictChanged(T, T),

    /// no-op, dont care
    Pass,
}

impl<T: core::fmt::Debug + PartialEq> Diff<T> {
    /// # Panics
    /// - if difference is not same as `self`
    #[inline]
    pub fn assert(&self, old: &T, new: &T) {
        match self {
            Diff::Unchanged => assert!(
                old == new,
                "Expected unchanged but {old:#?} changed to {new:#?}"
            ),
            Diff::Changed(expected_old, expected_new)
            | Diff::StrictChanged(expected_old, expected_new) => {
                assert!(
                    expected_old == old,
                    "Expected old to be {expected_old:#?} but got {old:#?}"
                );
                assert!(
                    expected_new == new,
                    "Expected new to be {expected_new:#?} but got {new:#?}"
                );
                if matches!(self, Diff::StrictChanged(..)) {
                    assert!(old != new, "Expected old != new  but got both {old:#?}");
                }
            }
            Diff::Pass => (),
        }
    }
}

#[macro_export]
macro_rules! gas_diff_zip_assert {
    ($diff:expr, $bef:expr, $aft:expr) => {
        $diff
            .0
            .iter()
            .zip(&$bef.0)
            .zip(&$aft.0)
            .for_each(|((diff, bef), aft)| diff.assert(bef, aft));
    };
}
