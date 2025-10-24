use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::caba;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AddLstIxAccs<T> {
    /// The pool's admin
    pub admin: T,

    /// Account paying the SOL rent for the new space and accounts
    pub payer: T,

    /// Mint of the new LST to add
    pub lst_mint: T,

    /// LST reserves token account to create
    pub pool_reserves: T,

    /// The LST protocol fee accumulator token account to create
    pub protocol_fee_accumulator: T,

    /// The protocol fee accumulator token account authority PDA. PDA ["protocol_fee"]
    pub protocol_fee_accumulator_auth: T,

    /// The LST's SOL value calculator program
    pub sol_value_calculator: T,

    /// The pool's state singleton PDA
    pub pool_state: T,

    /// Dynamic list PDA of LstStates for each LST in the pool
    pub lst_state_list: T,

    /// Associated token account program
    pub associated_token_program: T,

    /// System program
    pub system_program: T,

    /// Token program of the new LST to add
    pub lst_token_program: T,
}

impl<T: Copy> AddLstIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; ADD_LST_IX_ACCS_LEN])
    }
}

pub type AddLstIxKeys<'a> = AddLstIxAccs<&'a [u8; 32]>;

pub type AddLstIxKeysOwned = AddLstIxAccs<[u8; 32]>;

pub type AddLstIxAccFlags = AddLstIxAccs<bool>;

impl<T> AsRef<[T]> for AddLstIxAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

pub const ADD_LST_IX_IS_WRITER: AddLstIxAccFlags = AddLstIxAccFlags::memset(true)
    .const_with_payer(true)
    .const_with_pool_reserves(true)
    .const_with_protocol_fee_accumulator(true)
    .const_with_protocol_fee_accumulator_auth(true)
    .const_with_lst_state_list(true);

pub const ADD_LST_IX_IS_SIGNER: AddLstIxAccFlags = AddLstIxAccFlags::memset(false)
    .const_with_admin(true)
    .const_with_payer(true);

// Data

pub const ADD_LST_IX_DISCM: u8 = 7;

pub const ADD_LST_IX_DATA_LEN: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AddLstIxData([u8; ADD_LST_IX_DATA_LEN]);

impl AddLstIxData {
    #[inline]
    pub const fn new() -> Self {
        const A: usize = ADD_LST_IX_DATA_LEN;

        let mut d = [0u8; A];

        d = caba::<A, 0, 1>(d, &[ADD_LST_IX_DISCM]);

        Self(d)
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; ADD_LST_IX_DATA_LEN] {
        &self.0
    }
}
