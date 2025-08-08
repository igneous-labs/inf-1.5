use inf1_pp_ag_core::PricingAg;
use inf1_pp_flatfee_std::update::UpdateInnerErr as FlatFeeInnerErr;

use crate::PricingProgAg;

// Re-exports
pub use inf1_pp_std::update::UpdatePricingProg;

pub mod mint_lp;
pub mod price_exact_in;
pub mod price_exact_out;
pub mod redeem_lp;

pub type UpdatePpErr = PricingAg<FlatFeeInnerErr>;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > UpdatePricingProg for PricingProgAg<F, C>
{
    type InnerErr = UpdatePpErr;

    fn update_mint_lp(
        &mut self,
        update_map: impl inf1_update_traits::UpdateMap,
    ) -> Result<(), inf1_update_traits::UpdateErr<Self::InnerErr>> {
        match &mut self.0 {
            PricingAg::FlatFee(p) => p
                .update_mint_lp(update_map)
                .map_err(|e| e.map_inner(PricingAg::FlatFee)),
        }
    }

    fn update_redeem_lp(
        &mut self,
        update_map: impl inf1_update_traits::UpdateMap,
    ) -> Result<(), inf1_update_traits::UpdateErr<Self::InnerErr>> {
        match &mut self.0 {
            PricingAg::FlatFee(p) => p
                .update_redeem_lp(update_map)
                .map_err(|e| e.map_inner(PricingAg::FlatFee)),
        }
    }

    fn update_price_exact_in(
        &mut self,
        swap_mints: &inf1_pp_std::pair::Pair<&[u8; 32]>,
        update_map: impl inf1_update_traits::UpdateMap,
    ) -> Result<(), inf1_update_traits::UpdateErr<Self::InnerErr>> {
        match &mut self.0 {
            PricingAg::FlatFee(p) => p
                .update_price_exact_in(swap_mints, update_map)
                .map_err(|e| e.map_inner(PricingAg::FlatFee)),
        }
    }

    fn update_price_exact_out(
        &mut self,
        swap_mints: &inf1_pp_std::pair::Pair<&[u8; 32]>,
        update_map: impl inf1_update_traits::UpdateMap,
    ) -> Result<(), inf1_update_traits::UpdateErr<Self::InnerErr>> {
        match &mut self.0 {
            PricingAg::FlatFee(p) => p
                .update_price_exact_out(swap_mints, update_map)
                .map_err(|e| e.map_inner(PricingAg::FlatFee)),
        }
    }
}
