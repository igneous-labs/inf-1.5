use generic_array_struct::generic_array_struct;
use sanctum_spl_token_core::state::account::RawTokenAccount;

use crate::{gas_diff_zip_assert, Diff};

#[derive(Debug, Clone, Copy, Default)]
pub struct RawTokenAccArgs<T, U, V, W> {
    pub state: T,
    /// byte arrays of length 4
    pub ba4s: U,
    pub ba8s: V,
    pub ba32s: W,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RawTokenAccBa4s<T> {
    pub delegate_coption_discm: T,
    pub native_rent_exemption_coption_discm: T,
    pub close_auth_coption_discm: T,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RawTokenAccBa8s<T> {
    pub amount: T,
    pub native_rent_exemption: T,
    pub delegated_amount: T,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RawTokenAccBa32s<T> {
    pub mint: T,
    pub auth: T,
    pub delegate: T,
    pub close_auth: T,
}

pub type GenRawTokenAccArgs = RawTokenAccArgs<
    u8,
    RawTokenAccBa4s<[u8; 4]>,
    RawTokenAccBa8s<[u8; 8]>,
    RawTokenAccBa32s<[u8; 32]>,
>;

pub fn raw_token_acc_to_gen_args(
    RawTokenAccount {
        mint,
        auth,
        amount,
        delegate_coption_discm,
        delegate,
        state,
        native_rent_exemption_coption_discm,
        native_rent_exemption,
        delegated_amount,
        close_auth_coption_discm,
        close_auth,
    }: &RawTokenAccount,
) -> GenRawTokenAccArgs {
    GenRawTokenAccArgs {
        state: *state,
        ba4s: NewRawTokenAccBa4sBuilder::start()
            .with_close_auth_coption_discm(*close_auth_coption_discm)
            .with_delegate_coption_discm(*delegate_coption_discm)
            .with_native_rent_exemption_coption_discm(*native_rent_exemption_coption_discm)
            .build(),
        ba8s: NewRawTokenAccBa8sBuilder::start()
            .with_amount(*amount)
            .with_delegated_amount(*delegated_amount)
            .with_native_rent_exemption(*native_rent_exemption)
            .build(),
        ba32s: NewRawTokenAccBa32sBuilder::start()
            .with_auth(*auth)
            .with_close_auth(*close_auth)
            .with_delegate(*delegate)
            .with_mint(*mint)
            .build(),
    }
}

pub type DiffsRawTokenAccArgs = RawTokenAccArgs<
    Diff<u8>,
    RawTokenAccBa4s<Diff<[u8; 4]>>,
    RawTokenAccBa8s<Diff<[u8; 8]>>,
    RawTokenAccBa32s<Diff<[u8; 32]>>,
>;

pub fn assert_token_acc_diffs(
    bef: &RawTokenAccount,
    aft: &RawTokenAccount,
    DiffsRawTokenAccArgs {
        state,
        ba4s,
        ba8s,
        ba32s,
    }: &DiffsRawTokenAccArgs,
) {
    let [RawTokenAccArgs {
        state: bef_state,
        ba4s: bef_ba4s,
        ba8s: bef_ba8s,
        ba32s: bef_ba32s,
    }, RawTokenAccArgs {
        state: aft_state,
        ba4s: aft_ba4s,
        ba8s: aft_ba8s,
        ba32s: aft_ba32s,
    }] = [bef, aft].map(raw_token_acc_to_gen_args);
    state.assert(&bef_state, &aft_state);
    gas_diff_zip_assert!(ba4s, bef_ba4s, aft_ba4s);
    gas_diff_zip_assert!(ba8s, bef_ba8s, aft_ba8s);
    gas_diff_zip_assert!(ba32s, bef_ba32s, aft_ba32s);
}

/// Note: `Changed`, not `StrictChanged`
pub fn token_acc_bal_diff_changed(bef: &RawTokenAccount, change: i128) -> DiffsRawTokenAccArgs {
    let old_bal = u64::from_le_bytes(bef.amount);
    let new_bal = if change < 0 {
        old_bal - u64::try_from(-change).unwrap()
    } else {
        old_bal + u64::try_from(change).unwrap()
    };
    DiffsRawTokenAccArgs {
        ba8s: RawTokenAccBa8s::default()
            .with_amount(Diff::Changed(bef.amount, new_bal.to_le_bytes())),
        ..Default::default()
    }
}
