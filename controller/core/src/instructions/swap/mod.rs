use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::{caba, csba};

pub mod exact_in;
pub mod exact_out;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IxPreAccs<T> {
    pub signer: T,
    pub inp_lst_mint: T,
    pub out_lst_mint: T,
    pub inp_lst_acc: T,
    pub out_lst_acc: T,
    pub protocol_fee_accumulator: T,
    pub inp_lst_token_program: T,
    pub out_lst_token_program: T,
    pub pool_state: T,
    pub lst_state_list: T,
    pub inp_pool_reserves: T,
    pub out_pool_reserves: T,
}

impl<T: Copy> IxPreAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; IX_PRE_ACCS_LEN])
    }
}

pub type IxPreKeys<'a> = IxPreAccs<&'a [u8; 32]>;

pub type IxPreKeysOwned = IxPreAccs<[u8; 32]>;

pub type IxPreAccFlags = IxPreAccs<bool>;

impl<T> AsRef<[T]> for IxPreAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

pub const IX_PRE_IS_WRITER: IxPreAccFlags = IxPreAccFlags::memset(true)
    .const_with_signer(false)
    .const_with_inp_lst_mint(false)
    .const_with_out_lst_mint(false)
    .const_with_inp_lst_token_program(false)
    .const_with_out_lst_token_program(false);

pub const IX_PRE_IS_SIGNER: IxPreAccFlags = IxPreAccFlags::memset(false).const_with_signer(true);

// Data

pub const IX_DATA_LEN: usize = 27;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IxArgs {
    pub inp_lst_value_calc_accs: u8,
    pub out_lst_value_calc_accs: u8,
    pub inp_lst_index: u32,
    pub out_lst_index: u32,

    /// - min_amount_out for ExactIn
    /// - max_amount_in for ExactOut
    pub limit: u64,

    pub amount: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IxData<const DISCM: u8>([u8; IX_DATA_LEN]);

impl<const DISCM: u8> IxData<DISCM> {
    #[inline]
    pub const fn new(
        IxArgs {
            inp_lst_value_calc_accs,
            out_lst_value_calc_accs,
            inp_lst_index,
            out_lst_index,
            limit,
            amount,
        }: IxArgs,
    ) -> Self {
        const A: usize = IX_DATA_LEN;

        let mut d = [0u8; A];

        d = caba::<A, 0, 1>(d, &[DISCM]);
        d = caba::<A, 1, 1>(d, &[inp_lst_value_calc_accs]);
        d = caba::<A, 2, 1>(d, &[out_lst_value_calc_accs]);
        d = caba::<A, 3, 4>(d, &inp_lst_index.to_le_bytes());
        d = caba::<A, 7, 4>(d, &out_lst_index.to_le_bytes());
        d = caba::<A, 11, 8>(d, &limit.to_le_bytes());
        d = caba::<A, 19, 8>(d, &amount.to_le_bytes());

        Self(d)
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; IX_DATA_LEN] {
        &self.0
    }

    #[inline]
    pub const fn parse_no_discm(data: &[u8; 26]) -> IxArgs {
        let (input_lst_value_calc_accs, rest) = csba::<26, 1, 25>(data);
        let (out_lst_value_calc_accs, rest) = csba::<25, 1, 24>(rest);
        let (inp_lst_index, rest) = csba::<24, 4, 20>(rest);
        let (out_lst_index, rest) = csba::<20, 4, 16>(rest);
        let (limit, rest) = csba::<16, 8, 8>(rest);
        let (amount, _) = csba::<8, 8, 0>(rest);

        IxArgs {
            inp_lst_value_calc_accs: input_lst_value_calc_accs[0],
            out_lst_value_calc_accs: out_lst_value_calc_accs[0],
            inp_lst_index: u32::from_le_bytes(*inp_lst_index),
            out_lst_index: u32::from_le_bytes(*out_lst_index),
            limit: u64::from_le_bytes(*limit),
            amount: u64::from_le_bytes(*amount),
        }
    }
}
