use crate::instructions::{
    admin::lst_input::{
        NewSetLstInputIxAccsBuilder, SetLstInputIxAccs, SET_LST_INPUT_IX_IS_SIGNER,
        SET_LST_INPUT_IX_IS_WRITER,
    },
    generic::U32IxData,
};

// Accounts

pub type NewEnableLstInputIxAccsBuilder<T> = NewSetLstInputIxAccsBuilder<T>;

pub type EnableLstInputIxAccs<T> = SetLstInputIxAccs<T>;

pub type EnableLstInputIxKeys<'a> = SetLstInputIxAccs<&'a [u8; 32]>;

pub type EnableLstInputIxKeysOwned = SetLstInputIxAccs<[u8; 32]>;

pub type EnableLstInputIxAccFlags = SetLstInputIxAccs<bool>;

pub const ENABLE_LST_INPUT_IX_IS_WRITER: EnableLstInputIxAccFlags = SET_LST_INPUT_IX_IS_WRITER;

pub const ENABLE_LST_INPUT_IX_IS_SIGNER: EnableLstInputIxAccFlags = SET_LST_INPUT_IX_IS_SIGNER;

// Data

pub const ENABLE_LST_INPUT_IX_DISCM: u8 = 6;

pub const ENABLE_LST_INPUT_IX_DATA_LEN: usize = EnableLstInputIxData::DATA_LEN;

pub type EnableLstInputIxData = U32IxData<ENABLE_LST_INPUT_IX_DISCM>;
