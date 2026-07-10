use generic_array_struct::generic_array_struct;
use inf1_ctl_jiminy::accounts::pool_state::{PoolStateV2, PoolStateV2Packed};
use inf1_ctl_jiminy::err::Inf1CtlErr;
use inf1_ctl_jiminy::instructions::init::{
    InitIxAccs, InitIxAccsGen, InitIxData, InitIxPreAccs, InitIxPreAccsDestr, InitIxProgs,
    InitIxProgsDestr, INIT_IX_IS_SIGNER, INIT_IX_IS_WRITER, INIT_IX_PRE_ACCS_IDX_INIT_ADMIN,
    INIT_IX_PROGS_KEYS_OWNED,
};
use inf1_ctl_jiminy::keys::CONST_KEYS_OWNED;
use inf1_ctl_jiminy::program_err::Inf1CtlCustomProgErr;
use inf1_std::pda::CONST_PDA_KEYS_OWNED;
use inf1_test_utils::{
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_mint, mock_sys_acc, mollusk_exec,
    pool_state_v2_account, raw_mint, AccountMap,
};

use jiminy_cpi::program_error::MISSING_REQUIRED_SIGNATURE;
use mollusk_svm::program::keyed_account_for_system_program;
use mollusk_svm_programs_token::token::keyed_account as keyed_account_for_token_program;
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::mint::{Mint, RawMint};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use jiminy_entrypoint::program_error::{ProgramError, INVALID_ARGUMENT};

use crate::common::SVM;

type InitKeysOwned = InitIxAccsGen<[u8; 32]>;

fn init_ix(keys: &InitKeysOwned) -> Instruction {
    let accounts =
        keys_signer_writable_to_metas(keys.seq(), INIT_IX_IS_SIGNER.seq(), INIT_IX_IS_WRITER.seq());
    Instruction {
        program_id: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        accounts,
        data: InitIxData::as_buf().into(),
    }
}

#[generic_array_struct(all pub)]
#[derive(Debug)]
pub struct InitTestAccs<T> {
    pub lp_token_mint: T,
    pub pool_state: T,
}

fn init_test_accs(keys: &InitKeysOwned, accs: InitTestAccs<Account>) -> AccountMap {
    let system_prog = keyed_account_for_system_program();
    let token_prog = keyed_account_for_token_program();
    let init_admin = mock_sys_acc(1_000_000_000);
    let payer = mock_sys_acc(1_000_000_000);
    let InitTestAccsDestr {
        lp_token_mint,
        pool_state,
    } = accs.into_destr();

    let accs = InitIxAccs {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            payer,
            init_admin,
            pool_state,
            lp_token_mint,
        }),
        progs: InitIxProgs::from_destr(InitIxProgsDestr {
            token: token_prog.1,
            sys: system_prog.1,
        }),
    };

    keys.seq()
        .copied()
        .map(Into::into)
        .zip(accs.seq().cloned())
        .collect()
}

fn init_test(ix: &Instruction, bef: &AccountMap, expected_err: Option<impl Into<ProgramError>>) {
    let (result, slot) = SVM.with(|svm| {
        (
            mollusk_exec(svm, core::slice::from_ref(ix), bef),
            svm.sysvars.clock.slot,
        )
    });

    match expected_err {
        None => {
            let ok = result.unwrap();
            let pre_addrs = InitIxPreAccs(
                ix.accounts
                    .first_chunk()
                    .unwrap()
                    .each_ref()
                    .map(|a| a.pubkey),
            );
            let pre = InitIxPreAccs(pre_addrs.0.map(|a| a.to_bytes()));

            let [pool_state_aft, mint_aft] = [pre_addrs.pool_state(), pre_addrs.lp_token_mint()]
                .map(|a| ok.resulting_accounts.get(a).unwrap());

            let ps = PoolStateV2Packed::of_acc_data(&pool_state_aft.data)
                .unwrap()
                .into_pool_state_v2();
            assert_eq!(
                pool_state_aft.owner,
                Pubkey::new_from_array(*CONST_KEYS_OWNED.program())
            );
            assert_eq!(ps, PoolStateV2::init(slot, *pre.lp_token_mint()));

            let mint = Mint::try_from_raw(RawMint::of_acc_data(&mint_aft.data).unwrap()).unwrap();
            assert_eq!(mint.freeze_auth(), Some(pre.pool_state()));
            assert_eq!(mint.mint_auth(), Some(pre.pool_state()));
        }
        Some(e) => {
            assert_jiminy_prog_err(&result.unwrap_err(), e);
        }
    }
}

#[test]
fn init_success_from_empty() {
    let keys = InitIxAccs {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            payer: [2u8; 32],
            init_admin: *CONST_KEYS_OWNED.init_admin(),
            pool_state: *CONST_PDA_KEYS_OWNED.pool_state(),
            lp_token_mint: [3u8; 32],
        }),
        progs: INIT_IX_PROGS_KEYS_OWNED,
    };
    let am = init_test_accs(
        &keys,
        InitTestAccs::from_destr(InitTestAccsDestr {
            lp_token_mint: mock_mint(raw_mint(
                Some(*CONST_KEYS_OWNED.init_admin()),
                Some(*CONST_KEYS_OWNED.init_admin()),
                0,
                9,
            )),
            pool_state: mock_sys_acc(0),
        }),
    );
    init_test(&init_ix(&keys), &am, Option::<ProgramError>::None);
}

#[test]
fn init_fail_nonzero_supply() {
    let keys = InitIxAccs {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            payer: [2u8; 32],
            init_admin: *CONST_KEYS_OWNED.init_admin(),
            pool_state: *CONST_PDA_KEYS_OWNED.pool_state(),
            lp_token_mint: [3u8; 32],
        }),
        progs: INIT_IX_PROGS_KEYS_OWNED,
    };
    let am = init_test_accs(
        &keys,
        InitTestAccs::from_destr(InitTestAccsDestr {
            lp_token_mint: mock_mint(raw_mint(
                Some(*CONST_KEYS_OWNED.init_admin()),
                Some(*CONST_KEYS_OWNED.init_admin()),
                100, // non-zero supply
                9,
            )),
            pool_state: mock_sys_acc(0),
        }),
    );
    init_test(
        &init_ix(&keys),
        &am,
        Some(Inf1CtlCustomProgErr(
            Inf1CtlErr::IncorrectLpMintInitialization,
        )),
    );
}

#[test]
fn init_fail_wrong_decimals() {
    let keys = InitIxAccs {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            payer: [2u8; 32],
            init_admin: *CONST_KEYS_OWNED.init_admin(),
            pool_state: *CONST_PDA_KEYS_OWNED.pool_state(),
            lp_token_mint: [3u8; 32],
        }),
        progs: INIT_IX_PROGS_KEYS_OWNED,
    };
    let am = init_test_accs(
        &keys,
        InitTestAccs::from_destr(InitTestAccsDestr {
            lp_token_mint: mock_mint(raw_mint(
                Some(*CONST_KEYS_OWNED.init_admin()),
                Some(*CONST_KEYS_OWNED.init_admin()),
                0,
                0,
            )),
            pool_state: mock_sys_acc(0),
        }),
    );
    init_test(
        &init_ix(&keys),
        &am,
        Some(Inf1CtlCustomProgErr(
            Inf1CtlErr::IncorrectLpMintInitialization,
        )),
    );
}

#[test]
fn init_fail_already_init() {
    let lp_token_mint_addr = [3u8; 32];
    let keys = InitIxAccs {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            payer: [2u8; 32],
            init_admin: *CONST_KEYS_OWNED.init_admin(),
            pool_state: *CONST_PDA_KEYS_OWNED.pool_state(),
            lp_token_mint: lp_token_mint_addr,
        }),
        progs: INIT_IX_PROGS_KEYS_OWNED,
    };
    let am = init_test_accs(
        &keys,
        InitTestAccs::from_destr(InitTestAccsDestr {
            lp_token_mint: mock_mint(raw_mint(
                Some(*CONST_KEYS_OWNED.init_admin()),
                Some(*CONST_KEYS_OWNED.init_admin()),
                0,
                9,
            )),
            pool_state: pool_state_v2_account(PoolStateV2::init(0, lp_token_mint_addr)),
        }),
    );
    init_test(&init_ix(&keys), &am, Some(INVALID_ARGUMENT));
}

#[test]
fn init_fail_unauthorized() {
    let wrong_auth = [2u8; 32];
    let keys = InitIxAccs {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            payer: wrong_auth,
            init_admin: wrong_auth,
            pool_state: *CONST_PDA_KEYS_OWNED.pool_state(),
            lp_token_mint: [3u8; 32],
        }),
        progs: INIT_IX_PROGS_KEYS_OWNED,
    };
    let am = init_test_accs(
        &keys,
        InitTestAccs::from_destr(InitTestAccsDestr {
            lp_token_mint: mock_mint(raw_mint(
                Some(*CONST_KEYS_OWNED.init_admin()),
                Some(*CONST_KEYS_OWNED.init_admin()),
                0,
                9,
            )),
            pool_state: mock_sys_acc(0),
        }),
    );
    init_test(&init_ix(&keys), &am, Some(INVALID_ARGUMENT));
}

#[test]
fn init_fail_missing_sig() {
    let keys = InitIxAccs {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            payer: [2u8; 32],
            init_admin: *CONST_KEYS_OWNED.init_admin(),
            pool_state: *CONST_PDA_KEYS_OWNED.pool_state(),
            lp_token_mint: [3u8; 32],
        }),
        progs: INIT_IX_PROGS_KEYS_OWNED,
    };
    let am = init_test_accs(
        &keys,
        InitTestAccs::from_destr(InitTestAccsDestr {
            lp_token_mint: mock_mint(raw_mint(
                Some(*CONST_KEYS_OWNED.init_admin()),
                Some(*CONST_KEYS_OWNED.init_admin()),
                0,
                9,
            )),
            pool_state: mock_sys_acc(0),
        }),
    );

    let mut ix = init_ix(&keys);
    ix.accounts[INIT_IX_PRE_ACCS_IDX_INIT_ADMIN].is_signer = false;

    init_test(&ix, &am, Some(MISSING_REQUIRED_SIGNATURE));
}
