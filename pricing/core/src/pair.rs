use core::convert::Infallible;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pair<T> {
    pub inp: T,
    pub out: T,
}

impl<T> Pair<T> {
    #[inline]
    pub fn try_map_mbr<R, E>(
        self,
        mut f: impl FnMut(PairMbr<T>) -> Result<R, E>,
    ) -> Result<Pair<R>, E> {
        let Self { inp, out } = self;
        let inp = f(PairMbr::Inp(inp))?;
        let out = f(PairMbr::Out(out))?;
        Ok(Pair { inp, out })
    }

    #[inline]
    pub fn try_map<R, E>(self, mut f: impl FnMut(T) -> Result<R, E>) -> Result<Pair<R>, E> {
        self.try_map_mbr(|x| match x {
            PairMbr::Inp(x) => f(x),
            PairMbr::Out(x) => f(x),
        })
    }

    #[inline]
    pub fn map<R>(self, mut f: impl FnMut(T) -> R) -> Pair<R> {
        self.try_map(|x| Ok::<_, Infallible>(f(x))).unwrap()
    }
}

/// A single member of the [`Pair`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PairMbr<T> {
    Inp(T),
    Out(T),
}

impl<T> PairMbr<T> {
    #[inline]
    pub const fn as_ref_t(&self) -> &T {
        match self {
            Self::Inp(x) => x,
            Self::Out(x) => x,
        }
    }
}

impl<T> AsRef<T> for PairMbr<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.as_ref_t()
    }
}

pub type PairDir = PairMbr<()>;
