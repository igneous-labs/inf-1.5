//! TODO state definition
// TODO: we're missing fetching POOL_PROGDATA and verifying that last_upgrade_slot has not changed.
// Have been omitted for now because pool programs rarely change and the program data acc is huge,
// but this can result in SDKs giving quotes that are no longer applicable due to stake pool prog upgrade.

#![cfg_attr(not(test), no_std)]

pub mod instructions;
pub mod pda;
