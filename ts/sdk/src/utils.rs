use bs58_fixed_wasm::Bs58Array;
use inf1_core::inf1_ctl_core::typedefs::lst_state::{LstState, LstStatePacked};
use inf1_update_traits::UpdateMap;

use crate::{
    err::{unsupported_mint_err, InfError},
    interface::{Account, AccountMap},
};

pub(crate) fn epoch_from_clock_data(clock_acc_data: &[u8]) -> Option<u64> {
    u64_le_at(clock_acc_data, 16)
}

pub(crate) fn token_supply_from_mint_data(mint_acc_data: &[u8]) -> Option<u64> {
    u64_le_at(mint_acc_data, 36)
}

pub(crate) fn balance_from_token_acc_data(token_acc_data: &[u8]) -> Option<u64> {
    u64_le_at(token_acc_data, 64)
}

fn u64_le_at(data: &[u8], at: usize) -> Option<u64> {
    chunk_at(data, at).map(|c| u64::from_le_bytes(*c))
}

fn chunk_at<const N: usize>(data: &[u8], at: usize) -> Option<&[u8; N]> {
    data.get(at..).and_then(|s| s.first_chunk())
}

pub(crate) fn try_find_lst_state(
    packed: &[LstStatePacked],
    mint: &[u8; 32],
) -> Result<(usize, LstState), InfError> {
    packed
        .iter()
        .enumerate()
        .map(|(i, l)| (i, l.into_lst_state()))
        .find(|(_i, l)| l.mint == *mint)
        .ok_or_else(|| unsupported_mint_err(mint))
}

impl inf1_update_traits::Account for Account {
    #[inline]
    fn data(&self) -> &[u8] {
        &self.data
    }
}

impl UpdateMap for AccountMap {
    type Account<'a> = &'a Account;

    #[inline]
    fn get_account(&self, pk: &[u8; 32]) -> Option<Self::Account<'_>> {
        self.0.get(&Bs58Array(*pk))
    }
}
