use inf1_pp_reserve_v2_core::{
    accounts::pricing_state_account_size,
    instructions::admin::{
        common::{AdminIxPreAccs, AdminIxPreAccsDestr, ADMIN_IX_PRE_IS_SIGNER},
        remove_fee_entry::{
            RemoveFeeEntryIxAccs, RemoveFeeEntryIxAccsGen, RemoveFeeEntryIxSufAccs,
        },
    },
    keys::CONST_KEYS_OWNED,
    pda::CONST_PDA_KEYS_OWNED,
    typedefs::FeeEntryPacked,
};
use inf1_pp_reserve_v2_jiminy::account_utils::pricing_state_checked;
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};
use jiminy_sysvar_rent::{sysvar::SimpleSysvar, Rent};

use inf1_pp_reserve_v2_core::errs::ReserveV2ProgramErr;
use inf1_pp_reserve_v2_jiminy::program_err::CustomProgErr;

use crate::utils::{asfc, verify_pks, verify_signers};

pub type RemoveFeeEntryIxAccHandles<'a> = RemoveFeeEntryIxAccsGen<AccountHandle<'a>>;

pub fn remove_fee_entry_accs_checked<'acc>(
    abr: &Abr,
    accounts: &[AccountHandle<'acc>],
) -> Result<RemoveFeeEntryIxAccHandles<'acc>, ProgramError> {
    let (pre_accs, suf_accs) = asfc(accounts)?;
    let pre = AdminIxPreAccs(*pre_accs);
    let (suf, _) = asfc(suf_accs)?;
    let suf = RemoveFeeEntryIxSufAccs(*suf);

    let (stored_admin, _) = pricing_state_checked(abr.get(*pre.pricing_state()))?;

    verify_pks(
        abr,
        &pre.0,
        &AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
            pricing_state: CONST_PDA_KEYS_OWNED.pricing_state(),
            admin: stored_admin,
        })
        .0,
    )?;

    verify_signers(abr, &pre.0, &ADMIN_IX_PRE_IS_SIGNER.0)?;

    let mint_pk = abr.get(*suf.mint()).key();
    if *mint_pk == *CONST_KEYS_OWNED.lp_mint() || *mint_pk == *CONST_KEYS_OWNED.wsol_mint() {
        return Err(CustomProgErr(ReserveV2ProgramErr::CantRemoveRequiredMint).into());
    }

    // No further verification required for suf, both accs are non-signers and free
    // - idempotent means we do nothing if mint not found
    // - rent can be refunded to any address, including the pricing_state itself

    Ok(RemoveFeeEntryIxAccs { pre, suf })
}

pub fn process_remove_fee_entry<'acc>(
    abr: &mut Abr,
    accs: RemoveFeeEntryIxAccHandles<'acc>,
) -> Result<(), ProgramError> {
    let mint = *abr.get(*accs.suf.mint()).key();
    let pricing_state = abr.get_mut(*accs.pre.pricing_state());

    let (_, entries) = pricing_state_checked(pricing_state)?;

    let idx = match entries.find_idx_by_mint(&mint) {
        Ok(i) => i,
        // mint already doesnt exist, success
        Err(_) => return Ok(()),
    };

    let old_acc_len = pricing_state.data().len();
    let entry_size = core::mem::size_of::<FeeEntryPacked>();
    let removed_start = pricing_state_account_size(idx);
    let removed_end = removed_start + entry_size;
    pricing_state
        .data_mut()
        .copy_within(removed_end..old_acc_len, removed_start);
    pricing_state.shrink_by(entry_size)?;

    let lamports_surplus = pricing_state
        .lamports()
        .saturating_sub(Rent::get()?.min_balance(pricing_state.data_len()));
    if lamports_surplus > 0 {
        abr.transfer_direct(
            *accs.pre.pricing_state(),
            *accs.suf.refund_rent_to(),
            lamports_surplus,
        )?;
    }

    Ok(())
}
