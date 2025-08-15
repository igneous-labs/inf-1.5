use inf1_pp_core::{instructions::price::IxAccs as PriceIxAccs, pair::Pair};
use inf1_pp_flatslab_core::{
    instructions::pricing::{FlatSlabPpAccs, IxSufAccs, NewIxSufAccsBuilder},
    keys::SLAB_ID,
};
use solana_account::Account;
use solana_pubkey::Pubkey;

mod exact_in;

type PkAccountTup = (Pubkey, Account);

pub type PriceIxKeysOwned = PriceIxAccs<[u8; 32], FlatSlabPpAccs>;

fn price_keys_owned(Pair { inp, out }: Pair<[u8; 32]>) -> PriceIxKeysOwned {
    let suf = FlatSlabPpAccs(NewIxSufAccsBuilder::start().with_slab(SLAB_ID).build());
    inf1_pp_core::instructions::price::IxAccs {
        ix_prefix: inf1_pp_core::instructions::price::NewIxPreAccsBuilder::start()
            .with_input_mint(inp)
            .with_output_mint(out)
            .build(),
        suf,
    }
}

pub type PriceAccounts =
    inf1_pp_core::instructions::price::IxAccs<PkAccountTup, IxSufAccs<PkAccountTup>>;

pub fn price_ix_accounts(keys: &PriceIxKeysOwned, slab_data: Vec<u8>) -> PriceAccounts {
    PriceIxAccs::new(
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
                Account {
                    data: slab_data,
                    owner: Pubkey::new_from_array(inf1_pp_flatslab_core::ID),
                    lamports: u64::MAX / 2, // dont rly care, long as its enough to be rent exempt
                    executable: false,
                    rent_epoch: u64::MAX,
                },
            ))
            .build(),
    )
}
