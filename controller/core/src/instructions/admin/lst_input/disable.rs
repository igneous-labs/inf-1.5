use crate::instructions::{
    admin::lst_input::{
        NewSetLstInputIxAccsBuilder, SetLstInputIxAccs, SET_LST_INPUT_IX_IS_SIGNER,
        SET_LST_INPUT_IX_IS_WRITER,
    },
    generic::U32IxData,
};

// Accounts

pub type NewDisableLstInputIxAccsBuilder<T> = NewSetLstInputIxAccsBuilder<T>;

pub type DisableLstInputIxAccs<T> = SetLstInputIxAccs<T>;

pub type DisableLstInputIxKeys<'a> = SetLstInputIxAccs<&'a [u8; 32]>;

pub type DisableLstInputIxKeysOwned = SetLstInputIxAccs<[u8; 32]>;

pub type DisableLstInputIxAccFlags = SetLstInputIxAccs<bool>;

pub const DISABLE_LST_INPUT_IX_IS_WRITER: DisableLstInputIxAccFlags = SET_LST_INPUT_IX_IS_WRITER;

pub const DISABLE_LST_INPUT_IS_SIGNER: DisableLstInputIxAccFlags = SET_LST_INPUT_IX_IS_SIGNER;

// Data

pub const DISABLE_LST_INPUT_IX_DISCM: u8 = 5;

pub const DISABLE_LST_INPUT_IX_DATA_LEN: usize = DisableLstInputIxData::DATA_LEN;

pub type DisableLstInputIxData = U32IxData<DISABLE_LST_INPUT_IX_DISCM>;
