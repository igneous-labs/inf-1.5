use inf1_core::inf1_ctl_core::typedefs::lst_state::{LstState, LstStatePacked};
use inf1_pp_ag_std::{inf1_pp_flatfee_std::FlatFeePricing, PricingAg, PricingAgTy, PricingProgAg};

use crate::err::InfErr;

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
) -> Result<(usize, LstState), InfErr> {
    packed
        .iter()
        .enumerate()
        .map(|(i, l)| (i, l.into_lst_state()))
        .find(|(_i, l)| l.mint == *mint)
        .ok_or(InfErr::UnsupportedMint { mint: *mint })
}

pub(crate) fn try_default_pricing_prog_from_program_id<
    F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
    C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
>(
    pp_prog_id: &[u8; 32],
    find_pda: F,
    create_pda: C,
) -> Result<PricingProgAg<F, C>, InfErr> {
    PricingAgTy::try_from_program_id(pp_prog_id)
        .map(|ty| match ty {
            PricingAgTy::FlatFee(_) => PricingProgAg(PricingAg::FlatFee(
                pricing_prog_flat_fee_default(find_pda, create_pda),
            )),
        })
        .ok_or(InfErr::UnknownPp {
            pp_prog_id: *pp_prog_id,
        })
}

fn pricing_prog_flat_fee_default<
    F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
    C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
>(
    find_pda: F,
    create_pda: C,
) -> FlatFeePricing<F, C> {
    FlatFeePricing::new(None, Default::default(), find_pda, create_pda)
}
