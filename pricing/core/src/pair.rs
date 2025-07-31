#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pair<T> {
    pub inp: T,
    pub out: T,
}
