/// Example
///
/// ```ignore
/// map_variant_method!(&self.0, accounts_to_update_all(all_mints))
/// ```
///
/// expands to
///
/// ```ignore
/// match self.0 {
///     PricingAg::FlatFee(p) => PricingAg::FlatFee(p.accounts_to_update_all(all_mints)),
///     PricingAg::FlatSlab(p) => PricingAg::FlatSlab(p.accounts_to_update_all(all_mints)),
/// }
/// ```
macro_rules! map_variant_method {
    ($ag:expr, $($e:tt)*) => {
        match $ag {
            PricingAg::FlatFee(p) => PricingAg::FlatFee(p.$($e)*),
            PricingAg::FlatSlab(p) => PricingAg::FlatSlab(p.$($e)*),
        }
    };
}
pub(crate) use map_variant_method;

/// Example
///
/// ```ignore
/// map_variant_method_fallible!(&self.0, price_exact_in_for(mints))
/// ```
///
/// expands to
///
/// ```ignore
/// match self.0 {
///     PricingAg::FlatFee(p) => p.price_exact_in_for(mints).map(PricingAg::FlatFee).map_err(|e| PricingAg::FlatFee(e.into())),
///     PricingAg::FlatSlab(p) => p.price_exact_in_for(mints).map(PricingAg::FlatSlab).map_err(|e| PricingAg::FlatSlab(e.into())),
/// }
/// ```
macro_rules! map_variant_method_fallible {
    ($ag:expr, $($e:tt)*) => {
        match $ag {
            PricingAg::FlatFee(p) => (p.$($e)*).map(PricingAg::FlatFee).map_err(|e| PricingAg::FlatFee(e.into())),
            PricingAg::FlatSlab(p) => (p.$($e)*).map(PricingAg::FlatSlab).map_err(|e| PricingAg::FlatSlab(e.into())),
        }
    };
}
pub(crate) use map_variant_method_fallible;
