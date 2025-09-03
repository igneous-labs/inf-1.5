use inf1_pp_core::{
    instructions::{
        price::{
            exact_in::{
                price_exact_in_ix_is_signer, price_exact_in_ix_is_writer,
                price_exact_in_ix_keys_owned, PriceExactInIxData,
            },
            IxAccs as PriceIxAccs,
        },
        IxArgs,
    },
    pair::Pair,
};
use inf1_pp_flatslab_core::{
    instructions::pricing::{FlatSlabPpAccs, IxSufAccs, NewIxSufAccsBuilder},
    keys::SLAB_ID,
    ID,
};
use inf1_test_utils::{keys_signer_writable_to_metas, PkAccountTup};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

#[allow(deprecated)]
use inf1_pp_core::instructions::deprecated::lp::IxAccs as LpIxAccs;

use crate::common::solana::slab_account;

mod exact_in;
mod exact_out;
mod mint_lp;
mod redeem_lp;

// Price

pub fn price_exact_in_ix(args: IxArgs, keys: &PriceIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        price_exact_in_ix_keys_owned(keys).seq(),
        price_exact_in_ix_is_signer(keys).seq(),
        price_exact_in_ix_is_writer(keys).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: PriceExactInIxData::new(args).as_buf().into(),
    }
}

pub type PriceIxKeysOwned = PriceIxAccs<[u8; 32], FlatSlabPpAccs>;

pub fn price_keys_owned(Pair { inp, out }: Pair<[u8; 32]>) -> PriceIxKeysOwned {
    PriceIxKeysOwned::new(
        inf1_pp_core::instructions::price::NewIxPreAccsBuilder::start()
            .with_input_mint(inp)
            .with_output_mint(out)
            .build(),
        FlatSlabPpAccs(NewIxSufAccsBuilder::start().with_slab(SLAB_ID).build()),
    )
}

pub type PriceAccounts = PriceIxAccs<PkAccountTup, IxSufAccs<PkAccountTup>>;

pub fn price_ix_accounts(keys: &PriceIxKeysOwned, slab_data: Vec<u8>) -> PriceAccounts {
    PriceAccounts::new(
        inf1_pp_core::instructions::price::NewIxPreAccsBuilder::start()
            .with_input_mint((
                Pubkey::new_from_array(*keys.ix_prefix.input_mint()),
                Account::default(),
            ))
            .with_output_mint((
                Pubkey::new_from_array(*keys.ix_prefix.output_mint()),
                Account::default(),
            ))
            .build(),
        NewIxSufAccsBuilder::start()
            .with_slab((
                Pubkey::new_from_array(*keys.suf.0.slab()),
                slab_account(slab_data),
            ))
            .build(),
    )
}

// LP

#[allow(deprecated)]
pub type LpIxKeysOwned = LpIxAccs<[u8; 32], FlatSlabPpAccs>;

#[allow(deprecated)]
pub fn lp_keys_owned(mint: [u8; 32]) -> LpIxKeysOwned {
    LpIxKeysOwned::new(
        inf1_pp_core::instructions::deprecated::lp::NewIxPreAccsBuilder::start()
            .with_mint(mint)
            .build(),
        FlatSlabPpAccs(NewIxSufAccsBuilder::start().with_slab(SLAB_ID).build()),
    )
}

#[allow(deprecated)]
pub type LpAccounts = LpIxAccs<PkAccountTup, IxSufAccs<PkAccountTup>>;

#[allow(deprecated)]
pub fn lp_ix_accounts(keys: &LpIxKeysOwned, slab_data: Vec<u8>) -> LpAccounts {
    LpAccounts::new(
        inf1_pp_core::instructions::deprecated::lp::NewIxPreAccsBuilder::start()
            .with_mint((
                Pubkey::new_from_array(*keys.ix_prefix.mint()),
                Account::default(),
            ))
            .build(),
        NewIxSufAccsBuilder::start()
            .with_slab((
                Pubkey::new_from_array(*keys.suf.0.slab()),
                slab_account(slab_data),
            ))
            .build(),
    )
}
