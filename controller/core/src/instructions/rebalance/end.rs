use generic_array_struct::generic_array_struct;

use crate::instructions::{internal_utils::caba, rebalance::start::StartRebalanceIxPreAccs};

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct EndRebalanceIxPreAccs<T> {
    pub rebalance_auth: T,
    pub pool_state: T,
    pub lst_state_list: T,
    pub rebalance_record: T,
    pub inp_lst_mint: T,
    pub inp_pool_reserves: T,
}

impl<T: Copy> EndRebalanceIxPreAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; END_REBALANCE_IX_PRE_ACCS_LEN])
    }

    #[inline]
    pub const fn from_start(start: StartRebalanceIxPreAccs<T>) -> Self {
        NewEndRebalanceIxPreAccsBuilder::start()
            .with_rebalance_auth(*start.rebalance_auth())
            .with_inp_lst_mint(*start.inp_lst_mint())
            .with_inp_pool_reserves(*start.inp_pool_reserves())
            .with_lst_state_list(*start.lst_state_list())
            .with_pool_state(*start.pool_state())
            .with_rebalance_record(*start.rebalance_record())
            .build()
    }
}

pub type EndRebalanceIxPreKeys<'a> = EndRebalanceIxPreAccs<&'a [u8; 32]>;

pub type EndRebalanceIxPreKeysOwned = EndRebalanceIxPreAccs<[u8; 32]>;

pub type EndRebalanceIxPreAccFlags = EndRebalanceIxPreAccs<bool>;

impl<T> AsRef<[T]> for EndRebalanceIxPreAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T: Copy> From<StartRebalanceIxPreAccs<T>> for EndRebalanceIxPreAccs<T> {
    #[inline]
    fn from(value: StartRebalanceIxPreAccs<T>) -> Self {
        Self::from_start(value)
    }
}

pub const END_REBALANCE_IX_PRE_IS_WRITER: EndRebalanceIxPreAccFlags =
    EndRebalanceIxPreAccFlags::memset(true)
        .const_with_rebalance_auth(false)
        .const_with_inp_lst_mint(false)
        .const_with_inp_pool_reserves(false);

pub const END_REBALANCE_IX_PRE_IS_SIGNER: EndRebalanceIxPreAccFlags =
    EndRebalanceIxPreAccFlags::memset(false).const_with_rebalance_auth(true);

// Data

pub const END_REBALANCE_IX_DATA_LEN: usize = 1;

pub const END_REBALANCE_IX_DISCM: u8 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EndRebalanceIxData([u8; END_REBALANCE_IX_DATA_LEN]);

impl EndRebalanceIxData {
    #[inline]
    pub const fn new() -> Self {
        const A: usize = END_REBALANCE_IX_DATA_LEN;

        let mut d = [0u8; A];

        d = caba::<A, 0, 1>(d, &[END_REBALANCE_IX_DISCM]);

        Self(d)
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; END_REBALANCE_IX_DATA_LEN] {
        &self.0
    }
}

impl Default for EndRebalanceIxData {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
