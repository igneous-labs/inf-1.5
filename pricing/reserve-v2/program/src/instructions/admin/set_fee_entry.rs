use inf1_pp_reserve_v2_core::{
    accounts::pricing_state_account_size,
    instructions::admin::{
        common::{AdminIxPreAccs, AdminIxPreAccsDestr, ADMIN_IX_PRE_IS_SIGNER},
        set_fee_entry::{SetFeeEntryIxAccs, SetFeeEntryIxSufAccs},
    },
    pda::CONST_PDA_KEYS_OWNED,
    typedefs::{FeeEntry, FeeEntryNanos, FeeNanos, MintNotFoundErr, ThresholdNanos},
};
use inf1_pp_reserve_v2_jiminy::account_utils::{pricing_state_checked, pricing_state_checked_mut};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, INVALID_ACCOUNT_DATA},
};
use sanctum_system_jiminy::sanctum_system_core::instructions::transfer::NewTransferIxAccsBuilder;

use crate::{
    utils::{asfc, pay_for_rent_exempt_shortfall, verify_pks, verify_signers},
    Cpi,
};

pub type SetFeeEntryIxAccHandles<'a> = SetFeeEntryIxAccs<
    AdminIxPreAccs<AccountHandle<'a>>,
    SetFeeEntryIxSufAccs<AccountHandle<'a>>,
    (),
>;

pub fn set_fee_entry_accs_checked<'acc>(
    abr: &Abr,
    accounts: &[AccountHandle<'acc>],
) -> Result<SetFeeEntryIxAccHandles<'acc>, ProgramError> {
    let (pre_accs, rest) = asfc(accounts)?;
    let pre = AdminIxPreAccs(*pre_accs);
    let (suf_accs, _) = asfc(rest)?;
    let suf = SetFeeEntryIxSufAccs(*suf_accs);

    let (expected_admin, _) = pricing_state_checked(abr.get(*pre.pricing_state()))?;
    verify_pks(
        abr,
        &pre.0,
        &AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
            pricing_state: CONST_PDA_KEYS_OWNED.pricing_state(),
            admin: expected_admin,
        })
        .0,
    )?;

    verify_signers(abr, &pre.0, &ADMIN_IX_PRE_IS_SIGNER.0)?;

    Ok(SetFeeEntryIxAccs {
        pre,
        suf,
        sys_prog: (),
    })
}

pub fn process_set_fee_entry<'acc>(
    abr: &mut Abr,
    accs: SetFeeEntryIxAccHandles<'acc>,
    threshold_nanos: ThresholdNanos,
    fees: FeeEntryNanos<FeeNanos>,
) -> Result<(), ProgramError> {
    let mut cpi = Cpi::new();
    let mint = *abr.get(*accs.suf.mint()).key();
    let pricing_state = abr.get_mut(*accs.pre.pricing_state());

    let (_, mut entries) = pricing_state_checked_mut(pricing_state)?;

    match entries.find_by_mint_mut(&mint) {
        Ok(entry) => {
            *entry = FeeEntry {
                mint,
                threshold_nanos: threshold_nanos.get(),
                fee_nanos: FeeEntryNanos(fees.0.map(|x| x.get())),
            };
        }
        Err(MintNotFoundErr { expected_i, .. }) => {
            let entry_size = core::mem::size_of::<FeeEntry>();
            let old_len = pricing_state.data().len();
            let add_start = pricing_state_account_size(expected_i);
            let add_end = add_start
                .checked_add(entry_size)
                .ok_or(INVALID_ACCOUNT_DATA)?;
            pricing_state.grow_by(entry_size)?;
            pricing_state
                .data_mut()
                .copy_within(add_start..old_len, add_end);

            let (_, entries) = pricing_state_checked_mut(pricing_state)?;
            *entries.0.get_mut(expected_i).ok_or(INVALID_ACCOUNT_DATA)? = FeeEntry {
                mint,
                threshold_nanos: threshold_nanos.get(),
                fee_nanos: FeeEntryNanos(fees.0.map(|x| x.get())),
            };

            pay_for_rent_exempt_shortfall(
                abr,
                &mut cpi,
                NewTransferIxAccsBuilder::start()
                    .with_from(*accs.suf.payer())
                    .with_to(*accs.pre.pricing_state())
                    .build(),
            )?;
        }
    }

    Ok(())
}
