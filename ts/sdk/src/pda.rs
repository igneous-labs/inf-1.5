use std::iter::once;

use bs58_fixed_wasm::Bs58Array;
use ed25519_compact::{PublicKey, Signature};
use inf1_core::inf1_ctl_core::pda::{pool_reserves_ata_seeds, protocol_fee_accumulator_ata_seeds};
use inf1_svc_ag::inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::{
    ASSOCIATED_TOKEN_PROGRAM, TOKEN_PROGRAM,
};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{err::no_valid_pda_err, interface::B58PK};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
pub struct FoundPda(pub B58PK, pub u8);

#[wasm_bindgen(js_name = findPoolReservesAta)]
pub fn find_pool_reserves_ata(Bs58Array(mint): &B58PK) -> Result<FoundPda, JsError> {
    let [s1, s2, s3] = pool_reserves_ata_seeds(&TOKEN_PROGRAM, mint);
    find_pda(&[s1, s2, s3], &ASSOCIATED_TOKEN_PROGRAM)
        .ok_or_else(no_valid_pda_err)
        .map(|(pk, b)| FoundPda(B58PK::new(pk), b))
}

#[wasm_bindgen(js_name = findProtocolFeeAccumulatorAta)]
pub fn find_protocol_fee_accumulator_ata(Bs58Array(mint): &B58PK) -> Result<FoundPda, JsError> {
    let [s1, s2, s3] = protocol_fee_accumulator_ata_seeds(&TOKEN_PROGRAM, mint);
    find_pda(&[s1, s2, s3], &ASSOCIATED_TOKEN_PROGRAM)
        .ok_or_else(no_valid_pda_err)
        .map(|(pk, b)| FoundPda(B58PK::new(pk), b))
}

/// maximum length of derived `Pubkey` seed
const MAX_SEED_LEN: usize = 32;
/// Maximum number of seeds
const MAX_SEEDS: usize = 16;

const PDA_MARKER: &[u8; 21] = b"ProgramDerivedAddress";

/// Create a PDA without checking that its not on curve
pub(crate) fn create_raw_pda(
    seeds: impl IntoIterator<Item = impl AsRef<[u8]>>,
    program_id: &[u8; 32],
) -> Option<[u8; 32]> {
    let mut seed_len = 0;
    let mut hasher = hmac_sha256::Hash::new();
    seeds.into_iter().try_for_each(|seed| {
        seed_len += 1;
        if seed_len > MAX_SEEDS || seed.as_ref().len() > MAX_SEED_LEN {
            None
        } else {
            hasher.update(seed);
            Some(())
        }
    })?;
    hasher.update(program_id);
    hasher.update(PDA_MARKER);
    Some(hasher.finalize())
}

pub(crate) fn create_pda(
    seeds: impl IntoIterator<Item = impl AsRef<[u8]>>,
    program_id: &[u8; 32],
) -> Option<[u8; 32]> {
    let hash = create_raw_pda(seeds, program_id)?;
    // ed25519_compact only checks whether pubkey is on curve
    // when attempting to verify a signature so we try to verify a dummy one
    match PublicKey::new(hash).verify_incremental(&Signature::new([0u8; 64])) {
        // point is on curve
        //
        // See impl of verify_incremental():
        // https://github.com/jedisct1/rust-ed25519-compact/blob/00af8ee6778da59f57ecbe799a02ae5eb95495d9/src/ed25519.rs#L210
        Ok(_) | Err(ed25519_compact::Error::WeakPublicKey) => None,
        // point is not on curve
        Err(ed25519_compact::Error::InvalidPublicKey) => Some(hash),
        Err(_) => unreachable!(),
    }
}

pub(crate) fn find_pda(seeds: &[&[u8]], program_id: &[u8; 32]) -> Option<([u8; 32], u8)> {
    // Reference: https://github.com/anza-xyz/solana-sdk/blob/4e30766b8d327f0191df6490e48d9ef521956495/pubkey/src/lib.rs#L633
    // if you look at the impl, 0 is not a valid bump seed, only 1-255 are
    (1..=u8::MAX)
        .rev()
        .filter_map(|bump| {
            let bump_arr = [bump];
            let bump_slice = &bump_arr.as_slice();
            create_pda(seeds.iter().chain(once(bump_slice)), program_id).map(|pda| (pda, bump))
        })
        .next()
}
