use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::{caba, csba};

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct StartRebalanceIxPreAccs<T> {
    pub rebalance_auth: T,
    pub pool_state: T,
    pub lst_state_list: T,
    pub rebalance_record: T,
    pub out_lst_mint: T,
    pub inp_lst_mint: T,
    pub out_pool_reserves: T,
    pub inp_pool_reserves: T,
    pub withdraw_to: T,
    pub instructions: T,
    pub system_program: T,
    pub out_lst_token_program: T,
}

impl<T: Copy> StartRebalanceIxPreAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; START_REBALANCE_IX_PRE_ACCS_LEN])
    }
}

pub type StartRebalanceIxPreKeys<'a> = StartRebalanceIxPreAccs<&'a [u8; 32]>;

pub type StartRebalanceIxPreKeysOwned = StartRebalanceIxPreAccs<[u8; 32]>;

pub type StartRebalanceIxPreAccFlags = StartRebalanceIxPreAccs<bool>;

impl<T> AsRef<[T]> for StartRebalanceIxPreAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

pub const START_REBALANCE_IX_PRE_IS_WRITER: StartRebalanceIxPreAccFlags =
    StartRebalanceIxPreAccFlags::memset(true)
        .const_with_rebalance_auth(false)
        .const_with_inp_lst_mint(false)
        .const_with_out_lst_mint(false)
        .const_with_instructions(false)
        .const_with_system_program(false)
        .const_with_out_lst_token_program(false);

pub const START_REBALANCE_IX_PRE_IS_SIGNER: StartRebalanceIxPreAccFlags =
    StartRebalanceIxPreAccFlags::memset(false).const_with_rebalance_auth(true);

// Data

pub const START_REBALANCE_IX_DATA_LEN: usize = 34;

pub const START_REBALANCE_IX_DISCM: u8 = 19;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StartRebalanceIxArgs {
    pub out_lst_value_calc_accs: u8,
    pub out_lst_index: u32,
    pub inp_lst_index: u32,
    pub amount: u64,
    pub min_starting_out_lst: u64,
    pub max_starting_inp_lst: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StartRebalanceIxData([u8; START_REBALANCE_IX_DATA_LEN]);

impl StartRebalanceIxData {
    #[inline]
    pub const fn new(
        StartRebalanceIxArgs {
            out_lst_value_calc_accs,
            out_lst_index,
            inp_lst_index,
            amount,
            min_starting_out_lst,
            max_starting_inp_lst,
        }: StartRebalanceIxArgs,
    ) -> Self {
        const A: usize = START_REBALANCE_IX_DATA_LEN;

        let mut d = [0u8; A];

        d = caba::<A, 0, 1>(d, &[START_REBALANCE_IX_DISCM]);
        d = caba::<A, 1, 1>(d, &[out_lst_value_calc_accs]);
        d = caba::<A, 2, 4>(d, &out_lst_index.to_le_bytes());
        d = caba::<A, 6, 4>(d, &inp_lst_index.to_le_bytes());
        d = caba::<A, 10, 8>(d, &amount.to_le_bytes());
        d = caba::<A, 18, 8>(d, &min_starting_out_lst.to_le_bytes());
        d = caba::<A, 26, 8>(d, &max_starting_inp_lst.to_le_bytes());

        Self(d)
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; START_REBALANCE_IX_DATA_LEN] {
        &self.0
    }

    #[inline]
    pub const fn parse_no_discm(
        data: &[u8; START_REBALANCE_IX_DATA_LEN - 1],
    ) -> StartRebalanceIxArgs {
        let (out_lst_value_calc_accs, rest) = csba::<33, 1, 32>(data);
        let (out_lst_index, rest) = csba::<32, 4, 28>(rest);
        let (inp_lst_index, rest) = csba::<28, 4, 24>(rest);
        let (amount, rest) = csba::<24, 8, 16>(rest);
        let (min_starting_out_lst, rest) = csba::<16, 8, 8>(rest);
        let (max_starting_inp_lst, _) = csba::<8, 8, 0>(rest);

        StartRebalanceIxArgs {
            out_lst_value_calc_accs: out_lst_value_calc_accs[0],
            out_lst_index: u32::from_le_bytes(*out_lst_index),
            inp_lst_index: u32::from_le_bytes(*inp_lst_index),
            amount: u64::from_le_bytes(*amount),
            min_starting_out_lst: u64::from_le_bytes(*min_starting_out_lst),
            max_starting_inp_lst: u64::from_le_bytes(*max_starting_inp_lst),
        }
    }
}
