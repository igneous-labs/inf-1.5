use crate::typedefs::Empty;

/// Used to discriminate between SOL and non-SOL holding tokens
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TokenT<Sol, Other> {
    Sol(Sol),
    Other(Other),
}

impl<Sol, Other> TokenT<Sol, Other> {
    #[inline]
    pub fn map_sep<T, R>(
        self,
        sol: impl FnOnce(Sol) -> T,
        other: impl FnOnce(Other) -> R,
    ) -> TokenT<T, R> {
        match self {
            TokenT::Sol(x) => TokenT::Sol(sol(x)),
            TokenT::Other(x) => TokenT::Other(other(x)),
        }
    }

    #[inline]
    pub fn map_sep_ref<'a, T, R>(
        &'a self,
        sol: impl FnOnce(&'a Sol) -> T,
        other: impl FnOnce(&'a Other) -> R,
    ) -> TokenT<T, R> {
        match self {
            TokenT::Sol(x) => TokenT::Sol(sol(x)),
            TokenT::Other(x) => TokenT::Other(other(x)),
        }
    }

    #[inline]
    pub fn map_comb_ref<'a, T>(
        &'a self,
        sol: impl FnOnce(&'a Sol) -> T,
        other: impl FnOnce(&'a Other) -> T,
    ) -> T {
        match self {
            TokenT::Sol(x) => sol(x),
            TokenT::Other(x) => other(x),
        }
    }
}

impl<T> TokenT<T, T> {
    #[inline]
    pub fn unwrap(self) -> T {
        match self {
            Self::Sol(x) => x,
            Self::Other(x) => x,
        }
    }

    #[inline]
    pub const fn unwrap_ref(&self) -> &T {
        match self {
            Self::Sol(x) => x,
            Self::Other(x) => x,
        }
    }
}

pub type TokenSolEmpty<T> = TokenT<Empty, T>;

impl<T> TokenSolEmpty<T> {
    pub const SOL_EMPTY: Self = TokenSolEmpty::Sol(Empty::SELF);

    #[inline]
    pub fn map<R>(self, f: impl FnOnce(T) -> R) -> TokenSolEmpty<R> {
        self.map_sep(|x| x, f)
    }
}

#[macro_export]
macro_rules! const_map_rebal_sol_empty {
    ($sol_empty:expr, $f:expr) => {{
        match $sol_empty {
            $crate::typedefs::TokenT::Sol(_) => {
                $crate::typedefs::TokenT::Sol($crate::typedefs::Empty::SELF)
            }
            $crate::typedefs::TokenT::Other(_) => $crate::typedefs::TokenT::Other($f),
        }
    }};
}

impl<A, T> AsRef<[T]> for TokenSolEmpty<A>
where
    A: AsRef<[T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        match self {
            Self::Sol(_) => &[],
            Self::Other(a) => a.as_ref(),
        }
    }
}

impl<A, B, T> AsRef<[T]> for TokenT<A, B>
where
    A: AsRef<[T]>,
    B: AsRef<[T]>,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        match self {
            Self::Sol(_) => &[],
            Self::Other(a) => a.as_ref(),
        }
    }
}
