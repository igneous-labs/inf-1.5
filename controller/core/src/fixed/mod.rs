//! Binary fixed-point Q numbers
//! (https://en.wikipedia.org/wiki/Q_(number_format))
//!
//! ## Why handroll our own?
//! - `fixed` crate has dependencies that we dont need
//! - we only need multiplication and exponentiation of unsigned ratios <= 1.0
//!
//! ### TODO
//! Consider generalizing and separating this out into its own crate?

pub mod uq0_64;
