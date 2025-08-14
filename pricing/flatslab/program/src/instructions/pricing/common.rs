use inf1_pp_core::instructions::price::{IxPreAccs, IX_PRE_ACCS_LEN};
use inf1_pp_flatslab_core::{
    errs::FlatSlabProgramErr, instructions::pricing::IxSufAccs, keys::SLAB_ID,
};
use jiminy_entrypoint::{
    account::AccountHandle,
    program_error::{BuiltInProgramError, ProgramError},
};

use crate::{err::CustomProgErr, utils::verify_pks, Accounts};

pub type PricingIxPreAccHandles<'a> = IxPreAccs<AccountHandle<'a>>;

pub type PricingIxSufAccHandles<'a> = IxSufAccs<AccountHandle<'a>>;

const EXPECTED_PRICING_IX_SUF_ACC_KEYS: IxSufAccs<[u8; 32]> = IxSufAccs::new([SLAB_ID]);

pub fn pricing_accs_checked<'acc>(
    accounts: &mut Accounts<'acc>,
) -> Result<(PricingIxPreAccHandles<'acc>, PricingIxSufAccHandles<'acc>), ProgramError> {
    let Some((pre, suf)) = accounts
        .as_slice()
        .split_first_chunk::<IX_PRE_ACCS_LEN>()
        .and_then(|(pre, rest)| rest.first().map(|suf| (pre, suf)))
    else {
        return Err(ProgramError::from_builtin(
            BuiltInProgramError::NotEnoughAccountKeys,
        ));
    };

    let pre = PricingIxPreAccHandles::new(*pre);
    let suf = PricingIxSufAccHandles::new([*suf]);

    // check identities (only slab)
    verify_pks(accounts, &suf.0, &EXPECTED_PRICING_IX_SUF_ACC_KEYS.0)
        .map_err(|_| CustomProgErr(FlatSlabProgramErr::WrongSlabAcc))?;

    // no need to check signers here, all accounts are non-signers

    Ok((pre, suf))
}
