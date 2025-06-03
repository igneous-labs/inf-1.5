use core::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NotEnoughLiquidityErr {
    pub required: u64,
    pub available: u64,
}

impl Display for NotEnoughLiquidityErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "Not enough liquidity. Tokens required: {}. Available: {}",
            self.required, self.available
        ))
    }
}

impl Error for NotEnoughLiquidityErr {}
