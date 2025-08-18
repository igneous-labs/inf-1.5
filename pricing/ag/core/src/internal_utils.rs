/// Example
///
/// ```ignore
/// map_variant_pure!(&self.0, (|p| Display::fmt(&p, f)))
/// ```
///
/// expands to
///
/// ```ignore
/// match self.0 {
///     PricingAg::FlatFee(p) => (|p| Display::fmt(&p, f))(p),
///     PricingAg::FlatSlab(p) => (|p| Display::fmt(&p, f))(p),
/// }
/// ```
macro_rules! map_variant_pure {
    ($ag:expr, $($e:tt)*) => {
        match $ag {
            PricingAg::FlatFee(p) => ($($e)*(p)),
            PricingAg::FlatSlab(p) => ($($e)*(p)),
        }
    };
}
pub(crate) use map_variant_pure;

/// Example
///
/// ```ignore
/// map_variant!(&self.0, (|_| ())
/// ```
///
/// expands to
///
/// ```ignore
/// match self.0 {
///     PricingAg::FlatFee(p) => PricingAg::FlatFee((|_| ())(p)),
///     PricingAg::FlatSlab(p) => PricingAg::FlatSlab((|_|())(p)),
/// }
/// ```
macro_rules! map_variant {
    ($ag:expr, $($e:tt)*) => {
        match $ag {
            PricingAg::FlatFee(p) => PricingAg::FlatFee(($($e)*(p))),
            PricingAg::FlatSlab(p) =>  PricingAg::FlatSlab(($($e)*(p))),
        }
    };
}
pub(crate) use map_variant;

/// Example
///
/// ```ignore
/// map_variant_err!(&self.0, (|p| PriceLpTokensToMint::price_lp_tokens_to_mint(p, input))
/// ```
///
/// expands to
///
/// ```ignore
/// match self.0 {
///     PricingAg::FlatFee(p) => (|p| PriceLpTokensToMint::price_lp_tokens_to_mint(p, input)(p).map_err(PricingAg::FlatFee),
///     PricingAg::FlatSlab(p) => (|p| PriceLpTokensToMint::price_lp_tokens_to_mint(p, input)(p).map_err(PricingAg::FlatSlab),
/// }
/// ```
macro_rules! map_variant_err {
    ($ag:expr, $($e:tt)*) => {
        match $ag {
            PricingAg::FlatFee(p) => (($($e)*(p))).map_err(PricingAg::FlatFee),
            PricingAg::FlatSlab(p) =>  (($($e)*(p))).map_err(PricingAg::FlatSlab),
        }
    };
}
pub(crate) use map_variant_err;
