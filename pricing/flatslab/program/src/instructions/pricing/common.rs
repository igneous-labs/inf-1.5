use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_core::{
    errs::FlatSlabProgramErr,
    instructions::pricing::{IxSufAccs, IxSufKeys},
    keys::SLAB_ID,
    pricing::FlatSlabSwapPricing,
    typedefs::SlabEntryPackedList,
};
use jiminy_cpi::{account::Abr, program_error::NOT_ENOUGH_ACCOUNT_KEYS};
use jiminy_entrypoint::{account::AccountHandle, program_error::ProgramError};

use crate::{err::CustomProgErr, utils::verify_pks};

const EXPECTED_PRICING_IX_SUF_ACC_KEYS: IxSufKeys = IxSufKeys::new([&SLAB_ID]);

pub type PricingIxSufAccHandles<'a> = IxSufAccs<AccountHandle<'a>>;

// Price

pub type PriceIxPreAccs<T> = inf1_pp_core::instructions::price::IxPreAccs<T>;

pub type PriceIxAccHandles<'acc> =
    inf1_pp_core::instructions::price::IxAccs<AccountHandle<'acc>, PricingIxSufAccHandles<'acc>>;

pub fn pricing_accs_checked<'acc>(
    abr: &Abr,
    accounts: &[AccountHandle<'acc>],
) -> Result<PriceIxAccHandles<'acc>, ProgramError> {
    let Some((pre, suf)) = accounts
        .split_first_chunk()
        .and_then(|(pre, rest)| rest.first().map(|suf| (pre, suf)))
    else {
        return Err(NOT_ENOUGH_ACCOUNT_KEYS.into());
    };

    let pre = PriceIxPreAccs::new(*pre);
    let suf = PricingIxSufAccHandles::new([*suf]);

    // check identities (only slab)
    verify_pks(abr, &suf.0, &EXPECTED_PRICING_IX_SUF_ACC_KEYS.0)
        .map_err(|_| CustomProgErr(FlatSlabProgramErr::WrongSlabAcc))?;

    // no need to check signers here, all accounts are non-signers

    Ok(PriceIxAccHandles::new(pre, suf))
}

pub fn swap_pricing(
    abr: &Abr,
    entries: SlabEntryPackedList,
    pair: Pair<AccountHandle>,
) -> Result<FlatSlabSwapPricing, ProgramError> {
    let mints = pair.map(|h| abr.get(h).key());
    entries
        .pricing(&mints)
        .map_err(|e| CustomProgErr(FlatSlabProgramErr::MintNotFound(e)).into())
}

// Liquidity

#[allow(deprecated)]
pub type LpIxPreAccHandles<'a> =
    inf1_pp_core::instructions::deprecated::lp::IxPreAccs<AccountHandle<'a>>;

#[allow(deprecated)]
pub type LpIxAccHandles<'acc> = inf1_pp_core::instructions::deprecated::lp::IxAccs<
    AccountHandle<'acc>,
    PricingIxSufAccHandles<'acc>,
>;

#[allow(deprecated)]
pub fn lp_accs_checked<'acc>(
    abr: &Abr,
    accounts: &[AccountHandle<'acc>],
) -> Result<LpIxAccHandles<'acc>, ProgramError> {
    let Some((pre, suf)) = accounts
        .split_first_chunk()
        .and_then(|(pre, rest)| rest.first().map(|suf| (pre, suf)))
    else {
        return Err(NOT_ENOUGH_ACCOUNT_KEYS.into());
    };

    let pre = LpIxPreAccHandles::new(*pre);
    let suf = PricingIxSufAccHandles::new([*suf]);

    // check identities (only slab)
    verify_pks(abr, &suf.0, &EXPECTED_PRICING_IX_SUF_ACC_KEYS.0)
        .map_err(|_| CustomProgErr(FlatSlabProgramErr::WrongSlabAcc))?;

    // no need to check signers here, all accounts are non-signers

    Ok(LpIxAccHandles::new(pre, suf))
}
