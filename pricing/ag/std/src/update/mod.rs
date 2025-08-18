use inf1_pp_ag_core::PricingAg;
use inf1_pp_flatfee_std::update::UpdateInnerErr as FlatFeeInnerErr;
use inf1_pp_flatslab_std::update::FlatSlabPricingUpdateErr as FlatSlabInnerErr;
use inf1_update_traits::{UpdateErr, UpdateMap};

use crate::PricingProgAg;

// Re-exports
pub use inf1_pp_std::update::UpdatePricingProg;

pub mod all;
pub mod mint_lp;
pub mod price_exact_in;
pub mod price_exact_out;
pub mod redeem_lp;

pub type UpdatePpErr = PricingAg<FlatFeeInnerErr, FlatSlabInnerErr>;

/// Example
///
/// ```ignore
/// map_update_method!(&mut self.0, update_mint_lp(inp_mint, update_map))
/// ```
///
/// expands to
///
/// ```ignore
/// match self.0 {
///     PricingAg::FlatFee(p) =>  p
///          .update_mint_lp(inp_mint, update_map)
///          .map_err(|e| e.map_inner(PricingAg::FlatFee)),
///     PricingAg::FlatSlab(p) => p
///         .update_mint_lp(inp_mint, update_map)
///         .map_err(|e| e.map_inner(PricingAg::FlatSlab)),
/// }
/// ```
macro_rules! map_update_method {
    ($ag:expr, $($e:tt)*) => {
        match $ag {
            PricingAg::FlatFee(p) => p.$($e)*.map_err(|e| e.map_inner(PricingAg::FlatFee)),
            PricingAg::FlatSlab(p) => p.$($e)*.map_err(|e| e.map_inner(PricingAg::FlatSlab)),
        }
    };
}

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > UpdatePricingProg for PricingProgAg<F, C>
{
    type InnerErr = UpdatePpErr;

    fn update_mint_lp(
        &mut self,
        inp_mint: &[u8; 32],
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        map_update_method!(&mut self.0, update_mint_lp(inp_mint, update_map))
    }

    fn update_redeem_lp(
        &mut self,
        out_mint: &[u8; 32],
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        map_update_method!(&mut self.0, update_redeem_lp(out_mint, update_map))
    }

    fn update_price_exact_in(
        &mut self,
        swap_mints: &inf1_pp_std::pair::Pair<&[u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        map_update_method!(&mut self.0, update_price_exact_in(swap_mints, update_map))
    }

    fn update_price_exact_out(
        &mut self,
        swap_mints: &inf1_pp_std::pair::Pair<&[u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        map_update_method!(&mut self.0, update_price_exact_out(swap_mints, update_map))
    }

    fn update_all(
        &mut self,
        all_mints: impl IntoIterator<Item = [u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        map_update_method!(&mut self.0, update_all(all_mints, update_map))
    }
}
