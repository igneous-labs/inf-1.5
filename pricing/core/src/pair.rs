#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pair<T> {
    pub inp: T,
    pub out: T,
}

impl<T> Pair<T> {
    #[inline]
    pub fn try_map<R, E>(self, f: impl FnMut(T) -> Result<R, E>) -> Result<Pair<R>, E> {
        let [inp, out] = [self.inp, self.out].map(f);
        let inp = inp?;
        let out = out?;
        Ok(Pair { inp, out })
    }

    #[inline]
    pub fn map<R>(self, f: impl FnMut(T) -> R) -> Pair<R> {
        let [inp, out] = [self.inp, self.out].map(f);
        Pair { inp, out }
    }
}
