use inf1_pp_ag_core::{instructions::PriceExactOutAccsAg, PricingAg};
use inf1_pp_flatslab_core::{
    instructions::pricing::FlatSlabPpAccs, keys::SLAB_ID, typedefs::SlabEntryPacked,
};
use jiminy_sysvar_rent::Rent;
use solana_account::Account;

use crate::{AccountMap, KeyedUiAccount};

pub fn mock_flatslab_slab(data: Vec<u8>) -> Account {
    Account {
        lamports: Rent::DEFAULT.min_balance(data.len()),
        data,
        owner: inf1_pp_flatslab_core::ID.into(),
        executable: false,
        rent_epoch: u64::MAX,
    }
}

#[derive(Debug, Clone)]
pub struct FlatSlabAccParams {
    pub slab: Vec<u8>,
}

pub type PriceExactOutAccParamsAg = PricingAg<(), FlatSlabAccParams>;

pub fn flatslab_fixture_suf_accs() -> (FlatSlabPpAccs, AccountMap) {
    let (addr, acc) = KeyedUiAccount::from_test_fixtures_json("flatslab-slab").into_keyed_account();
    (
        FlatSlabPpAccs::new(addr.to_bytes()),
        core::iter::once((addr, acc)).collect(),
    )
}

pub fn price_exact_out_accs(params: PriceExactOutAccParamsAg) -> (PriceExactOutAccsAg, AccountMap) {
    match params {
        PricingAg::FlatFee(_) => todo!(),
        PricingAg::FlatSlab(FlatSlabAccParams { slab }) => (
            PricingAg::FlatSlab(FlatSlabPpAccs::MAINNET),
            core::iter::once((SLAB_ID.into(), mock_flatslab_slab(slab))).collect(),
        ),
    }
}

pub fn flatslab_acc_data(
    admin: [u8; 32],
    entries: impl IntoIterator<Item = impl Into<SlabEntryPacked>>,
) -> Vec<u8> {
    let mut packed: Vec<_> = entries.into_iter().map(Into::into).collect();
    packed.sort_unstable_by_key(|s| *s.mint());
    // admin
    admin
        .into_iter()
        .chain(packed.into_iter().flat_map(|p| *p.as_acc_data_arr()))
        .collect()
}
