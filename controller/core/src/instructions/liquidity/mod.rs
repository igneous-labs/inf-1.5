use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::{caba, csba};

pub mod add;
pub mod remove;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IxPreAccs<T> {
    pub signer: T,
    pub lst_mint: T,
    pub lst_acc: T,
    pub lp_acc: T,
    pub lp_token_mint: T,
    pub protocol_fee_accumulator: T,
    pub lst_token_program: T,
    pub lp_token_program: T,
    pub pool_state: T,
    pub lst_state_list: T,
    pub pool_reserves: T,
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
    .const_with_lst_mint(false)
    .const_with_lst_token_program(false)
    .const_with_lp_token_program(false);

pub const IX_PRE_IS_SIGNER: IxPreAccFlags = IxPreAccFlags::memset(false).const_with_signer(true);

// Data

pub const IX_DATA_LEN: usize = 22;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IxArgs {
    pub lst_value_calc_accs: u8,
    pub lst_index: u32,

    /// In terms of
    /// - LST tokens for AddLiquidity
    /// - LP tokens for RemoveLiquidity
    pub amount: u64,

    /// In terms of
    /// - LP tokens for AddLiquidity
    /// - LST tokens for RemoveLiquidity
    pub min_out: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LiquidityIxData<const DISCM: u8>([u8; IX_DATA_LEN]);

#[inline]
pub const fn new_liq_ix_data(
    discm: u8,
    IxArgs {
        lst_value_calc_accs,
        lst_index,
        amount,
        min_out,
    }: IxArgs,
) -> [u8; IX_DATA_LEN] {
    const A: usize = IX_DATA_LEN;

    let mut d = [0u8; A];

    d = caba::<A, 0, 1>(d, &[discm]);
    d = caba::<A, 1, 1>(d, &[lst_value_calc_accs]);
    d = caba::<A, 2, 4>(d, &lst_index.to_le_bytes());
    d = caba::<A, 6, 8>(d, &amount.to_le_bytes());
    d = caba::<A, 14, 8>(d, &min_out.to_le_bytes());

    d
}

impl<const DISCM: u8> LiquidityIxData<DISCM> {
    #[inline]
    pub const fn new(args: IxArgs) -> Self {
        Self(new_liq_ix_data(DISCM, args))
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; IX_DATA_LEN] {
        &self.0
    }
    #[inline]
    pub const fn parse_no_discm(data: &[u8; 21]) -> IxArgs {
        parse_liq_ix_args(data)
    }
}

#[inline]
pub const fn parse_liq_ix_args(data: &[u8; 21]) -> IxArgs {
    let (lst_value_calc_accs, rest) = csba::<21, 1, 20>(data);
    let (lst_index, rest) = csba::<20, 4, 16>(rest);
    let (amount, rest) = csba::<16, 8, 8>(rest);
    let (min_out, _) = csba::<8, 8, 0>(rest);

    IxArgs {
        lst_value_calc_accs: lst_value_calc_accs[0],
        lst_index: u32::from_le_bytes(*lst_index),
        amount: u64::from_le_bytes(*amount),
        min_out: u64::from_le_bytes(*min_out),
    }
}
