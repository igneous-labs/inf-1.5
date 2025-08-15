use inf1_pp_core::{instructions::IxArgs, pair::Pair};
use inf1_pp_flatslab_core::{
    accounts::Slab, errs::FlatSlabProgramErr, keys::LP_MINT_ID, typedefs::MintNotFoundErr, ID,
};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

#[allow(deprecated)]
use inf1_pp_core::instructions::deprecated::lp::redeem::{
    price_lp_tokens_to_redeem_ix_is_signer, price_lp_tokens_to_redeem_ix_is_writer,
    price_lp_tokens_to_redeem_ix_keys_owned, PriceLpTokensToRedeemIxData,
};

use crate::{
    common::{
        mollusk::{silence_mollusk_logs, MOLLUSK},
        props::{non_slab_pks, slab_for_liq, slab_for_swap, MAX_MINTS},
        solana::keys_signer_writable_to_metas,
        tests::should_fail_with_flatslab_prog_err,
    },
    tests::pricing::{
        lp_ix_accounts, lp_keys_owned, price_exact_in_ix, price_ix_accounts, price_keys_owned,
        LpIxKeysOwned,
    },
};

#[allow(deprecated)]
fn price_lp_tokens_to_redeem_ix(args: IxArgs, keys: &LpIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        price_lp_tokens_to_redeem_ix_keys_owned(keys).seq(),
        price_lp_tokens_to_redeem_ix_is_signer(keys).seq(),
        price_lp_tokens_to_redeem_ix_is_writer(keys).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: PriceLpTokensToRedeemIxData::new(args).as_buf().into(),
    }
}

proptest! {
    #[test]
    fn behaviour_should_be_same_as_price_exact_in(
        (slab_data, mint, _, _) in slab_for_liq(MAX_MINTS),
        amt: u64,
        sol_value: u64,
    ) {
        silence_mollusk_logs();

        let args = IxArgs { amt, sol_value };
        MOLLUSK.with(|mollusk| {
            let lp_keys = lp_keys_owned(mint);
            let lp_ix = price_lp_tokens_to_redeem_ix(args, &lp_keys);
            #[allow(deprecated)]
            let lp_accs = lp_ix_accounts(&lp_keys, slab_data.clone()).seq().cloned().collect::<Vec<_>>();

            let price_keys = price_keys_owned(Pair { inp: LP_MINT_ID, out: mint });
            let price_ix = price_exact_in_ix(args, &price_keys);
            let price_accs = price_ix_accounts(&price_keys, slab_data).seq().cloned().collect::<Vec<_>>();

            let [lp_res, price_res] = [(&lp_ix, &lp_accs), (&price_ix, &price_accs)]
                .map(|(i, a)| mollusk.process_instruction(i, a));

            prop_assert_eq!(lp_res.program_result, price_res.program_result);
            prop_assert_eq!(lp_res.return_data, price_res.return_data);

            Ok(())
        }).unwrap();
    }
}

proptest! {
    #[allow(deprecated)]
    #[test]
    fn should_fail_with_mint_not_found_if_lp_mint_entry_not_in_slab(
        (slab_data, Pair { inp, .. }, _,) in slab_for_swap(MAX_MINTS),
        amt: u64,
        sol_value: u64,
    ) {
        silence_mollusk_logs();

        let slab = Slab::of_acc_data(&slab_data).unwrap();
        let entries = slab.entries();
        if entries.find_by_mint(&LP_MINT_ID).is_ok() {
            return Ok(());
        }

        let args = IxArgs { amt, sol_value };
        let keys = lp_keys_owned(inp);
        let ix = price_lp_tokens_to_redeem_ix(args, &keys);
        let accs = lp_ix_accounts(&keys, slab_data);
        should_fail_with_flatslab_prog_err(
            &ix,
            &accs.seq().cloned().collect::<Vec<_>>(),
            FlatSlabProgramErr::MintNotFound(
                // dont-cares, just checking ProgramError code here
                MintNotFoundErr { expected_i: 0, mint: Default::default() }
            )
        );
    }
}

proptest! {
    #[allow(deprecated)]
    #[test]
    fn should_fail_with_mint_not_found_for_unknown_mints(
        (slab_data, _, _, _) in slab_for_liq(MAX_MINTS),
        mint: [u8; 32],
        amt: u64,
        sol_value: u64,
    ) {
        silence_mollusk_logs();

        let slab = Slab::of_acc_data(&slab_data).unwrap();
        let entries = slab.entries();
        if entries.find_by_mint(&mint).is_ok() {
            return Ok(());
        }

        let args = IxArgs { amt, sol_value };
        let keys = lp_keys_owned(mint);
        let ix = price_lp_tokens_to_redeem_ix(args, &keys);
        let accs = lp_ix_accounts(&keys, slab_data);
        should_fail_with_flatslab_prog_err(
            &ix,
            &accs.seq().cloned().collect::<Vec<_>>(),
            FlatSlabProgramErr::MintNotFound(
                // dont-cares, just checking ProgramError code here
                MintNotFoundErr { expected_i: 0, mint: Default::default() }
            )
        );
    }
}

proptest! {
    #[allow(deprecated)]
    #[test]
    fn should_fail_with_wrong_slab_acc_for_wrong_slab_acc(
        (slab_data, mint, _, _) in slab_for_liq(MAX_MINTS),
        wrong_slab_acc in non_slab_pks(),
        amt: u64,
        sol_value: u64,
    ) {
        silence_mollusk_logs();

        let args = IxArgs { amt, sol_value };
        let mut keys = lp_keys_owned(mint);
        keys.suf.0.set_slab(wrong_slab_acc);

        let ix = price_lp_tokens_to_redeem_ix(args, &keys);
        let accs = lp_ix_accounts(&keys, slab_data);
        should_fail_with_flatslab_prog_err(
            &ix,
            &accs.seq().cloned().collect::<Vec<_>>(),
            FlatSlabProgramErr::WrongSlabAcc,
        );
    }
}
