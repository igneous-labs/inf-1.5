use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::caba;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetSolValueCalculatorIxPreAccs<T> {
    /// The pool's admin
    pub admin: T,

    /// Mint of the LST to set SOL value calculator for
    pub lst_mint: T,

    /// The pool's state singleton PDA
    pub pool_state: T,

    /// Dynamic list PDA of LstStates for each LST in the pool
    pub lst_state_list: T,

    /// LST reserves token account of the pool.
    ///
    /// The LST's SOL value calculator program suffix accounts follow.
    pub pool_reserves: T,
}

impl<T: Copy> SetSolValueCalculatorIxPreAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; SET_SOL_VALUE_CALCULATOR_IX_PRE_ACCS_LEN])
    }
}

pub type SetSolValueCalculatorIxPreKeys<'a> = SetSolValueCalculatorIxPreAccs<&'a [u8; 32]>;

pub type SetSolValueCalculatorIxPreKeysOwned = SetSolValueCalculatorIxPreAccs<[u8; 32]>;

pub type SetSolValueCalculatorIxPreAccFlags = SetSolValueCalculatorIxPreAccs<bool>;

impl<T> AsRef<[T]> for SetSolValueCalculatorIxPreAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

pub const SET_SOL_VALUE_CALC_IX_PRE_IS_WRITER: SetSolValueCalculatorIxPreAccFlags =
    SetSolValueCalculatorIxPreAccFlags::memset(false)
        .const_with_pool_state(true)
        .const_with_lst_state_list(true);

pub const SET_SOL_VALUE_CALC_IX_PRE_IS_SIGNER: SetSolValueCalculatorIxPreAccFlags =
    SetSolValueCalculatorIxPreAccFlags::memset(false).const_with_admin(true);

// Data

pub const SET_SOL_VALUE_CALC_IX_DISCM: u8 = 9;

pub const SET_SOL_VALUE_CALC_IX_DATA_LEN: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetSolValueCalculatorIxData([u8; SET_SOL_VALUE_CALC_IX_DATA_LEN]);

impl SetSolValueCalculatorIxData {
    #[inline]
    pub const fn new(lst_idx: u32) -> Self {
        const A: usize = SET_SOL_VALUE_CALC_IX_DATA_LEN;

        let mut d = [0u8; A];

        d = caba::<A, 0, 1>(d, &[SET_SOL_VALUE_CALC_IX_DISCM]);
        d = caba::<A, 1, 4>(d, &lst_idx.to_le_bytes());

        Self(d)
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; SET_SOL_VALUE_CALC_IX_DATA_LEN] {
        &self.0
    }

    #[inline]
    pub const fn parse_no_discm(data: &[u8; 4]) -> u32 {
        u32::from_le_bytes(*data)
    }
}
