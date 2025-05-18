use super::internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FeeAccount {
    pub bump: u8,
    pub padding: u8,
    pub input_fee_bps: i16,
    pub output_fee_bps: i16,
}
impl_cast_from_acc_data!(FeeAccount, unsafe);
impl_cast_to_acc_data!(FeeAccount);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FeeAccountPacked {
    bump: u8,
    padding: u8,
    input_fee_bps: [u8; 2],
    output_fee_bps: [u8; 2],
}
impl_cast_from_acc_data!(FeeAccountPacked);
impl_cast_to_acc_data!(FeeAccountPacked);

impl FeeAccountPacked {
    #[inline]
    pub const fn into_fee_account(self) -> FeeAccount {
        let Self {
            bump,
            padding,
            input_fee_bps,
            output_fee_bps,
        } = self;
        FeeAccount {
            bump,
            padding,
            input_fee_bps: i16::from_le_bytes(input_fee_bps),
            output_fee_bps: i16::from_le_bytes(output_fee_bps),
        }
    }
}

impl From<FeeAccountPacked> for FeeAccount {
    #[inline]
    fn from(value: FeeAccountPacked) -> Self {
        value.into_fee_account()
    }
}
