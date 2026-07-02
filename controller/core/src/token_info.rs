use generic_array_struct::generic_array_struct;

use crate::keys::CONST_KEYS_OWNED;

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenInfo<T> {
    /// This token's SPL token program
    /// (either tokenkeg or token-22)
    pub program: T,

    pub mint: T,
}

impl<'a> TokenInfo<&'a [u8; 32]> {
    #[inline]
    pub const fn tokenkeg(mint: &'a [u8; 32]) -> Self {
        Self::const_from_destr(TokenInfoDestr {
            program: CONST_KEYS_OWNED.tokenkeg(),
            mint,
        })
    }
}
