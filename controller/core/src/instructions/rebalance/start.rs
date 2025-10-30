use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::caba;

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
    pub fn parse_no_discm(
        data: &[u8; START_REBALANCE_IX_DATA_LEN - 1],
    ) -> StartRebalanceIxArgs {
        let mut out_lst_index = [0u8; 4];
        out_lst_index.copy_from_slice(&data[1..5]);

        let mut inp_lst_index = [0u8; 4];
        inp_lst_index.copy_from_slice(&data[5..9]);

        let mut amount = [0u8; 8];
        amount.copy_from_slice(&data[9..17]);

        let mut min_starting_out_lst = [0u8; 8];
        min_starting_out_lst.copy_from_slice(&data[17..25]);

        let mut max_starting_inp_lst = [0u8; 8];
        max_starting_inp_lst.copy_from_slice(&data[25..33]);

        StartRebalanceIxArgs {
            out_lst_value_calc_accs: data[0],
            out_lst_index: u32::from_le_bytes(out_lst_index),
            inp_lst_index: u32::from_le_bytes(inp_lst_index),
            amount: u64::from_le_bytes(amount),
            min_starting_out_lst: u64::from_le_bytes(min_starting_out_lst),
            max_starting_inp_lst: u64::from_le_bytes(max_starting_inp_lst),
        }
    }
}
