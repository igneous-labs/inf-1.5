use inf1_ctl_jiminy::{
    accounts::pool_state::PoolStateV2,
    err::Inf1CtlErr,
    instructions::init::{InitIxPreAccs, InitIxPreAccsDestr, INIT_IX_PRE_IS_SIGNER},
    keys::CONST_KEYS_OWNED,
    pda::CONST_PDA_KEYS_OWNED,
    pda_onchain::POOL_STATE_SIGNER,
    program_err::Inf1CtlCustomProgErr,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};
use jiminy_sysvar_clock::Clock;
use jiminy_sysvar_rent::Rent;
use sanctum_spl_token_jiminy::sanctum_spl_token_core::instructions::set_auth::{
    AuthType, SetAuthIxAccs, SetAuthIxAccsDestr, SetAuthIxData,
};
use sanctum_system_jiminy::{
    instructions::assign::assign_invoke_signed,
    sanctum_system_core::instructions::{
        assign::NewAssignIxAccsBuilder, transfer::NewTransferIxAccsBuilder,
    },
};

use crate::{
    token::checked_mint_of,
    utils::{accs_split_first_chunk, pay_for_rent_exempt_shortfall},
    verify::{verify_pks, verify_pks_raw, verify_signers},
    Cpi,
};

const ALLOWED_DECIMALS: u8 = 9;

#[inline]
pub fn init_pre_accs_checked<'acc>(
    abr: &Abr,
    accs: &[AccountHandle<'acc>],
) -> Result<InitIxPreAccs<AccountHandle<'acc>>, ProgramError> {
    let (accs, _) = accs_split_first_chunk(accs)?;
    let accs = InitIxPreAccs(*accs);

    verify_pks(
        abr,
        &accs.0,
        &InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            init_admin: CONST_KEYS_OWNED.init_admin(),
            pool_state: CONST_PDA_KEYS_OWNED.pool_state(),

            // Free: mint properties verified below
            lp_token_mint: abr.get(*accs.lp_token_mint()).key(),

            // Free: payer can be anyone, no lof since we never invoke_signed with payer
            payer: abr.get(*accs.payer()).key(),
        })
        .0,
    )?;

    verify_signers(abr, &accs.0, &INIT_IX_PRE_IS_SIGNER.0)?;

    verify_pks_raw(
        &[accs.pool_state(), accs.lp_token_mint()].map(|h| abr.get(*h).owner()),
        &[CONST_KEYS_OWNED.sys_prog(), CONST_KEYS_OWNED.tokenkeg()],
    )?;

    let mint = checked_mint_of(abr.get(*accs.lp_token_mint()))?;
    if mint.supply() != 0 || mint.decimals() != ALLOWED_DECIMALS {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::IncorrectLpMintInitialization).into());
    }

    Ok(accs)
}

#[inline]
pub fn process_init(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &InitIxPreAccs<AccountHandle>,
    clock: &Clock,
    rent: &Rent,
) -> Result<(), ProgramError> {
    [AuthType::MintTokens, AuthType::FreezeAccount]
        .iter()
        .try_for_each(|auth_ty| {
            cpi.invoke_fwd(
                abr,
                CONST_KEYS_OWNED.tokenkeg(),
                SetAuthIxData::new(*auth_ty, Some(CONST_PDA_KEYS_OWNED.pool_state())).as_buf(),
                SetAuthIxAccs::from_destr(SetAuthIxAccsDestr {
                    set: *accs.lp_token_mint(),
                    auth: *accs.init_admin(),
                })
                .0,
            )
        })?;
    assign_invoke_signed(
        abr,
        cpi,
        NewAssignIxAccsBuilder::start()
            .with_assign(*accs.pool_state())
            .build(),
        CONST_KEYS_OWNED.program(),
        &[POOL_STATE_SIGNER],
    )?;

    let lp_token_mint_addr = *abr.get(*accs.lp_token_mint()).key();

    let pool_state_acc = abr.get_mut(*accs.pool_state());
    pool_state_acc.realloc(core::mem::size_of::<PoolStateV2>())?;

    // safety: acc data is 8-byte aligned
    *unsafe { PoolStateV2::of_acc_data_mut(pool_state_acc.data_mut()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))? =
        PoolStateV2::init(clock.slot, lp_token_mint_addr);

    pay_for_rent_exempt_shortfall(
        abr,
        cpi,
        &NewTransferIxAccsBuilder::start()
            .with_from(*accs.payer())
            .with_to(*accs.pool_state())
            .build(),
        rent,
    )?;

    Ok(())
}
