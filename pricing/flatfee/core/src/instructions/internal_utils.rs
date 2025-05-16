macro_rules! impl_asref {
    ($TyWithGeneric:ty) => {
        impl<T> AsRef<[T]> for $TyWithGeneric {
            #[inline]
            fn as_ref(&self) -> &[T] {
                &self.0
            }
        }
    };
}
pub(crate) use impl_asref;
