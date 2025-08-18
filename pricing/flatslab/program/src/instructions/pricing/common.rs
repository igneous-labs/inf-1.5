use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_core::{
    errs::FlatSlabProgramErr,
    instructions::pricing::{IxSufAccs, IxSufKeys},
    keys::SLAB_ID,
    pricing::FlatSlabSwapPricing,
    typedefs::SlabEntryPackedList,
};
use jiminy_cpi::program_error::NOT_ENOUGH_ACCOUNT_KEYS;
use jiminy_entrypoint::{account::AccountHandle, program_error::ProgramError};

use crate::{err::CustomProgErr, utils::verify_pks, Accounts};

pub type PricingIxSufAccHandles<'a> = IxSufAccs<AccountHandle<'a>>;

const EXPECTED_PRICING_IX_SUF_ACC_KEYS: IxSufKeys = IxSufKeys::new([&SLAB_ID]);

// Price

pub type PriceIxPreAccHandles<'a> = inf1_pp_core::instructions::price::IxPreAccs<AccountHandle<'a>>;

pub fn pricing_accs_checked<'acc>(
    accounts: &Accounts<'acc>,
) -> Result<(PriceIxPreAccHandles<'acc>, PricingIxSufAccHandles<'acc>), ProgramError> {
    use inf1_pp_core::instructions::price::IX_PRE_ACCS_LEN;

    let Some((pre, suf)) = accounts
        .as_slice()
        .split_first_chunk::<IX_PRE_ACCS_LEN>()
        .and_then(|(pre, rest)| rest.first().map(|suf| (pre, suf)))
    else {
        return Err(NOT_ENOUGH_ACCOUNT_KEYS.into());
    };

    let pre = PriceIxPreAccHandles::new(*pre);
    let suf = PricingIxSufAccHandles::new([*suf]);

    // check identities (only slab)
    verify_pks(accounts, &suf.0, &EXPECTED_PRICING_IX_SUF_ACC_KEYS.0)
        .map_err(|_| CustomProgErr(FlatSlabProgramErr::WrongSlabAcc))?;

    // no need to check signers here, all accounts are non-signers

    Ok((pre, suf))
}

pub fn swap_pricing(
    accounts: &Accounts,
    entries: SlabEntryPackedList,
    pair: Pair<AccountHandle>,
) -> Result<FlatSlabSwapPricing, ProgramError> {
    let mints = pair.map(|h| accounts.get(h).key());
    entries
        .pricing(&mints)
        .map_err(|e| CustomProgErr(FlatSlabProgramErr::MintNotFound(e)).into())
}

// Liquidity

#[allow(deprecated)]
pub type LpIxPreAccHandles<'a> =
    inf1_pp_core::instructions::deprecated::lp::IxPreAccs<AccountHandle<'a>>;

#[allow(deprecated)]
pub fn lp_accs_checked<'acc>(
    accounts: &Accounts<'acc>,
) -> Result<(LpIxPreAccHandles<'acc>, PricingIxSufAccHandles<'acc>), ProgramError> {
    use inf1_pp_core::instructions::deprecated::lp::IX_PRE_ACCS_LEN;

    let Some((pre, suf)) = accounts
        .as_slice()
        .split_first_chunk::<IX_PRE_ACCS_LEN>()
        .and_then(|(pre, rest)| rest.first().map(|suf| (pre, suf)))
    else {
        return Err(NOT_ENOUGH_ACCOUNT_KEYS.into());
    };

    let pre = LpIxPreAccHandles::new(*pre);
    let suf = PricingIxSufAccHandles::new([*suf]);

    // check identities (only slab)
    verify_pks(accounts, &suf.0, &EXPECTED_PRICING_IX_SUF_ACC_KEYS.0)
        .map_err(|_| CustomProgErr(FlatSlabProgramErr::WrongSlabAcc))?;

    // no need to check signers here, all accounts are non-signers

    Ok((pre, suf))
}
