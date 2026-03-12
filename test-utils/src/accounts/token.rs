//! TODO: these can probably go into a `sanctum-spl-token-test-utils` crate in the
//! sanctum-spl-token repo

use jiminy_sysvar_rent::Rent;
use sanctum_spl_token_core::state::{
    account::{RawTokenAccount, TokenAccount},
    mint::RawMint,
};
use solana_account::Account;
use solido_legacy_core::TOKENKEG_PROGRAM;

use crate::WSOL_MINT;

const TOKEN_ACC_RENT_EXEMPTION: u64 = Rent::DEFAULT.min_balance(RawTokenAccount::ACCOUNT_LEN);

// TODO: these should probably be in `sanctum_spl_token_core`
const COPTION_NONE: [u8; 4] = [0; 4];
const COPTION_SOME: [u8; 4] = [1, 0, 0, 0];

/// Max possible wSOL token acc balance (not u64::MAX due to `native_rent_exemption`)
pub const MAX_WSOL_BALANCE: u64 = u64::MAX - TOKEN_ACC_RENT_EXEMPTION;

/// Adapted from
/// https://github.com/igneous-labs/sanctum-solana-utils/blob/dc8426210a11e2c74ff21ae272dee953d457d0cd/sanctum-solana-test-utils/src/token/tokenkeg.rs#L44-L84
pub fn raw_token_acc(mint: [u8; 32], auth: [u8; 32], amt: u64) -> RawTokenAccount {
    let (native_rent_exemption_coption_discm, native_rent_exemption) =
        if mint == WSOL_MINT.to_bytes() {
            (COPTION_SOME, TOKEN_ACC_RENT_EXEMPTION.to_le_bytes())
        } else {
            (COPTION_NONE, [0; 8])
        };
    RawTokenAccount {
        mint,
        auth,
        amount: amt.to_le_bytes(),
        delegate_coption_discm: [0; 4],
        delegate: [0; 32],
        state: 1u8,
        native_rent_exemption_coption_discm,
        native_rent_exemption,
        delegated_amount: [0; 8],
        close_auth_coption_discm: [0; 4],
        close_auth: [0; 32],
    }
}

pub fn mock_token_acc_with_prog(a: RawTokenAccount, token_prog: [u8; 32]) -> Account {
    let lamports = match a.native_rent_exemption_coption_discm {
        COPTION_NONE => TOKEN_ACC_RENT_EXEMPTION,
        COPTION_SOME => [a.amount, a.native_rent_exemption]
            .map(u64::from_le_bytes)
            .iter()
            .sum(),
        _err => unreachable!(),
    };
    Account {
        lamports,
        data: a.as_acc_data_arr().into(),
        owner: token_prog.into(),
        executable: false,
        rent_epoch: u64::MAX,
    }
}

pub fn mock_token_acc(a: RawTokenAccount) -> Account {
    mock_token_acc_with_prog(a, TOKENKEG_PROGRAM)
}

/// Adapted from
/// https://github.com/igneous-labs/sanctum-solana-utils/blob/dc8426210a11e2c74ff21ae272dee953d457d0cd/sanctum-solana-test-utils/src/token/tokenkeg.rs#L86-L115
pub fn raw_mint(
    mint_auth: Option<[u8; 32]>,
    freeze_auth: Option<[u8; 32]>,
    supply: u64,
    decimals: u8,
) -> RawMint {
    let [(mint_auth_discm, mint_auth), (freeze_auth_discm, freeze_auth)] = [mint_auth, freeze_auth]
        .map(|opt| opt.map_or_else(|| (COPTION_NONE, [0u8; 32]), |x| (COPTION_SOME, x)));
    RawMint {
        mint_auth_discm,
        mint_auth,
        supply: supply.to_le_bytes(),
        decimals,
        is_init: 1,
        freeze_auth_discm,
        freeze_auth,
    }
}

pub fn mock_mint_with_prog(a: RawMint, token_prog: [u8; 32]) -> Account {
    Account {
        lamports: 1_461_600, // solana rent 82
        data: a.as_acc_data_arr().into(),
        owner: token_prog.into(),
        executable: false,
        rent_epoch: u64::MAX,
    }
}

pub fn mock_mint(a: RawMint) -> Account {
    mock_mint_with_prog(a, TOKENKEG_PROGRAM)
}

pub fn get_token_account_amount(token_acc_data: &[u8]) -> u64 {
    RawTokenAccount::of_acc_data(token_acc_data)
        .and_then(TokenAccount::try_from_raw)
        .expect("invalid token account")
        .amount()
}

pub fn get_mint_supply(mint_acc_data: &[u8]) -> u64 {
    u64::from_le_bytes(
        RawMint::of_acc_data(mint_acc_data)
            .expect("invalid mint account")
            .supply,
    )
}
