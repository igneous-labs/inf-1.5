use inf1_core::{
    instructions::{swap::IxAccs as SwapIxAccs, sync_sol_value::SyncSolValueIxAccs},
    quote::swap::SwapQuote,
};
use inf1_ctl_jiminy::{
    account_utils::{lst_state_list_checked, pool_state_checked},
    accounts::lst_state_list::LstStateList,
    cpi::SwapIxPreAccountHandles,
    err::Inf1CtlErr,
    instructions::{
        swap::{IxArgs, IxPreAccs, NewIxPreAccsBuilder},
        sync_sol_value::NewSyncSolValueIxPreAccsBuilder,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    pda_onchain::{
        create_raw_pool_reserves_addr, create_raw_protocol_fee_accumulator_addr, POOL_STATE_SIGNER,
    },
    program_err::Inf1CtlCustomProgErr,
    typedefs::{lst_state::LstState, u8bool::U8Bool},
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, INVALID_ACCOUNT_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
};
use sanctum_spl_token_jiminy::{
    instructions::transfer::transfer_checked_ix_account_handle_perms,
    sanctum_spl_token_core::{
        instructions::transfer::{NewTransferCheckedIxAccsBuilder, TransferCheckedIxData},
        state::mint::{Mint, RawMint},
    },
};

use crate::{
    svc::lst_sync_sol_val_unchecked,
    verify::{verify_not_rebalancing_and_not_disabled, verify_pks},
    Cpi,
};

pub type SwapIxAccounts<'a, 'acc> = SwapIxAccs<
    AccountHandle<'acc>,
    SwapIxPreAccountHandles<'acc>,
    &'a [AccountHandle<'acc>],
    &'a [AccountHandle<'acc>],
    &'a [AccountHandle<'acc>],
>;

pub fn swap_checked<'a, 'acc>(
    abr: &Abr,
    accounts: &'a [AccountHandle<'acc>],
    args: &IxArgs,
) -> Result<SwapIxAccounts<'a, 'acc>, ProgramError> {
    if args.amount == 0 {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into());
    }
    let (ix_prefix, suf) = accounts
        .split_first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

    let ix_prefix = IxPreAccs(*ix_prefix);

    let pool_state = *ix_prefix.pool_state();
    let lst_state_list = *ix_prefix.lst_state_list();
    let inp_lst_token_program = *ix_prefix.inp_lst_token_program();
    let out_lst_token_program = *ix_prefix.out_lst_token_program();

    let list = lst_state_list_checked(abr.get(lst_state_list))?;

    let (inp_lst_state, expected_inp_reserves) = get_lst_state_data(
        abr,
        &list,
        args.inp_lst_index as usize,
        inp_lst_token_program,
    )?;
    let (out_lst_state, expected_out_reserves) = get_lst_state_data(
        abr,
        &list,
        args.out_lst_index as usize,
        out_lst_token_program,
    )?;

    if inp_lst_state.mint == out_lst_state.mint {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::SwapSameLst).into());
    }

    let expected_protocol_fee_accumulator = create_raw_protocol_fee_accumulator_addr(
        abr.get(out_lst_token_program).key(),
        &out_lst_state.mint,
        &out_lst_state.protocol_fee_accumulator_bump,
    )
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    // Verify incoming accounts
    let expected_pks = NewIxPreAccsBuilder::start()
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_pool_state(&POOL_STATE_ID)
        .with_protocol_fee_accumulator(&expected_protocol_fee_accumulator)
        .with_inp_pool_reserves(&expected_inp_reserves)
        .with_out_pool_reserves(&expected_out_reserves)
        .with_inp_lst_mint(&inp_lst_state.mint)
        .with_out_lst_mint(&out_lst_state.mint)
        .with_inp_lst_token_program(abr.get(*ix_prefix.inp_lst_mint()).owner())
        .with_out_lst_token_program(abr.get(*ix_prefix.out_lst_mint()).owner())
        // NOTE: For the following accounts, it's okay to use the same ones passed by the user since the CPIs would fail if they're not as expected.
        // User can't pass the `inp_lst_reserves` as `inp_lst_acc` because we're also not doing `invoke_signed` for the `inp_lst` transfer.
        .with_inp_lst_acc(abr.get(*ix_prefix.inp_lst_acc()).key())
        .with_out_lst_acc(abr.get(*ix_prefix.out_lst_acc()).key())
        .with_signer(abr.get(*ix_prefix.signer()).key())
        .build();

    verify_pks(abr, &ix_prefix.0, &expected_pks.0)?;

    if U8Bool(&inp_lst_state.is_input_disabled).as_bool() {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::LstInputDisabled).into());
    }

    let pool = pool_state_checked(abr.get(pool_state))?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    let (inp_calc_all, suf) = suf
        .split_at_checked(args.inp_lst_value_calc_accs.into())
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let (out_calc_all, pricing_all) = suf
        .split_at_checked(args.out_lst_value_calc_accs.into())
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

    let [Some((inp_calc_prog, inp_calc)), Some((out_calc_prog, out_calc)), Some((pricing_prog, pricing))] =
        [inp_calc_all, out_calc_all, pricing_all].map(|arr| arr.split_first())
    else {
        return Err(NOT_ENOUGH_ACCOUNT_KEYS.into());
    };

    verify_pks(
        abr,
        &[*inp_calc_prog, *out_calc_prog, *pricing_prog],
        &[
            &inp_lst_state.sol_value_calculator,
            &out_lst_state.sol_value_calculator,
            &pool.pricing_program,
        ],
    )?;

    Ok(SwapIxAccounts {
        ix_prefix,
        inp_calc_prog: *inp_calc_prog,
        inp_calc,
        out_calc_prog: *out_calc_prog,
        out_calc,
        pricing_prog: *pricing_prog,
        pricing,
    })
}

pub fn get_lst_state_data<'a>(
    abr: &'a Abr,
    list: &'a LstStateList,
    idx: usize,
    lst_token_program: AccountHandle<'a>,
) -> Result<(&'a LstState, [u8; 32]), ProgramError> {
    let lst_state = list
        .0
        .get(idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;

    let expected_reserves = create_raw_pool_reserves_addr(
        abr.get(lst_token_program).key(),
        &lst_state.mint,
        &lst_state.pool_reserves_bump,
    )
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    Ok((lst_state, expected_reserves))
}

pub fn sync_inp_out_sol_vals(
    abr: &mut Abr,
    cpi: &mut Cpi,
    args: &IxArgs,
    swap_accs: &SwapIxAccounts<'_, '_>,
) -> Result<(), ProgramError> {
    let SwapIxAccounts {
        ix_prefix,
        inp_calc_prog,
        inp_calc,
        out_calc_prog,
        out_calc,
        ..
    } = *swap_accs;

    let inp_sync_sol_val_accs = SyncSolValueIxAccs {
        ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
            .with_lst_mint(*ix_prefix.inp_lst_mint())
            .with_pool_state(*ix_prefix.pool_state())
            .with_lst_state_list(*ix_prefix.lst_state_list())
            .with_pool_reserves(*ix_prefix.inp_pool_reserves())
            .build(),
        calc_prog: inp_calc_prog,
        calc: inp_calc,
    };
    let out_sync_sol_val_accs = SyncSolValueIxAccs {
        ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
            .with_lst_mint(*ix_prefix.out_lst_mint())
            .with_pool_state(*ix_prefix.pool_state())
            .with_lst_state_list(*ix_prefix.lst_state_list())
            .with_pool_reserves(*ix_prefix.out_pool_reserves())
            .build(),
        calc_prog: out_calc_prog,
        calc: out_calc,
    };

    let sync_sol_val_inputs = [
        (args.inp_lst_index, inp_sync_sol_val_accs),
        (args.out_lst_index, out_sync_sol_val_accs),
    ];

    sync_sol_val_inputs
        .iter()
        .try_for_each(|(idx, accs)| lst_sync_sol_val_unchecked(abr, cpi, *accs, *idx as usize))?;

    Ok(())
}

pub fn transfer_swap_tokens(
    abr: &mut Abr,
    cpi: &mut Cpi,
    quote: &SwapQuote,
    ix_prefix: &IxPreAccs<AccountHandle<'_>>,
) -> Result<(), ProgramError> {
    let inp_lst_decimals = RawMint::of_acc_data(abr.get(*ix_prefix.inp_lst_mint()).data())
        .and_then(Mint::try_from_raw)
        .map(|a| a.decimals())
        .ok_or(INVALID_ACCOUNT_DATA)?;

    let inp_lst_transfer_accs = NewTransferCheckedIxAccsBuilder::start()
        .with_auth(*ix_prefix.signer())
        .with_mint(*ix_prefix.inp_lst_mint())
        .with_src(*ix_prefix.inp_lst_acc())
        .with_dst(*ix_prefix.inp_pool_reserves())
        .build();

    cpi.invoke_fwd_handle(
        abr,
        *ix_prefix.inp_lst_token_program(),
        TransferCheckedIxData::new(quote.0.inp, inp_lst_decimals).as_buf(),
        inp_lst_transfer_accs.0,
    )?;

    let out_lst_decimals = RawMint::of_acc_data(abr.get(*ix_prefix.out_lst_mint()).data())
        .and_then(Mint::try_from_raw)
        .map(|a| a.decimals())
        .ok_or(INVALID_ACCOUNT_DATA)?;

    let protocol_fee_transfer_accs = transfer_checked_ix_account_handle_perms(
        NewTransferCheckedIxAccsBuilder::start()
            .with_auth(*ix_prefix.pool_state())
            .with_mint(*ix_prefix.out_lst_mint())
            .with_src(*ix_prefix.out_pool_reserves())
            .with_dst(*ix_prefix.protocol_fee_accumulator())
            .build(),
    );

    cpi.invoke_signed_handle(
        abr,
        *ix_prefix.out_lst_token_program(),
        TransferCheckedIxData::new(quote.0.protocol_fee, out_lst_decimals).as_buf(),
        protocol_fee_transfer_accs,
        &[POOL_STATE_SIGNER],
    )?;

    let out_lst_transfer_accs = transfer_checked_ix_account_handle_perms(
        NewTransferCheckedIxAccsBuilder::start()
            .with_auth(*ix_prefix.pool_state())
            .with_mint(*ix_prefix.out_lst_mint())
            .with_src(*ix_prefix.out_pool_reserves())
            .with_dst(*ix_prefix.out_lst_acc())
            .build(),
    );

    cpi.invoke_signed_handle(
        abr,
        *ix_prefix.out_lst_token_program(),
        TransferCheckedIxData::new(quote.0.out, out_lst_decimals).as_buf(),
        out_lst_transfer_accs,
        &[POOL_STATE_SIGNER],
    )?;

    Ok(())
}
