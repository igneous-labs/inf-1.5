use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::caba;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetLstFeeIxAccs<T> {
    /// The program admin
    pub admin: T,

    /// The signer paying for additional slab account rent if required
    pub payer: T,

    /// The slab PDA
    pub slab: T,

    /// Mint of the LST to set fees for
    pub mint: T,
}

impl<T: Copy> SetLstFeeIxAccs<T> {
    #[inline]
    pub const fn memset(v: T) -> Self {
        Self([v; SET_LST_FEE_IX_ACCS_LEN])
    }
}

pub type SetLstFeeIxKeys<'a> = SetLstFeeIxAccs<&'a [u8; 32]>;

pub type SetLstFeeIxKeysOwned = SetLstFeeIxAccs<[u8; 32]>;

pub type SetLstFeeIxAccFlags = SetLstFeeIxAccs<bool>;

pub const SET_LST_FEE_IX_IS_WRITER: SetLstFeeIxAccFlags = SetLstFeeIxAccFlags::memset(false)
    .const_with_payer(true)
    .const_with_slab(true);

pub const SET_LST_FEE_IX_IS_SIGNER: SetLstFeeIxAccFlags = SetLstFeeIxAccFlags::memset(false)
    .const_with_admin(true)
    .const_with_payer(true);

// Data

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetLstFeeIxArgs {
    pub inp_fee_nanos: i32,
    pub out_fee_nanos: i32,
}

impl SetLstFeeIxArgs {
    /// `d` should be slice of instruction data starting from after discriminant
    #[inline]
    pub const fn parse(d: &[u8; 8]) -> Self {
        let (inp_fee_nanos, out_fee_nanos) = match (d.first_chunk(), d.last_chunk()) {
            (Some(i), Some(o)) => (i32::from_le_bytes(*i), i32::from_le_bytes(*o)),
            _ => unreachable!(),
        };
        Self {
            inp_fee_nanos,
            out_fee_nanos,
        }
    }
}

pub const SET_LST_FEE_IX_DISCM: u8 = 253;

pub const SET_LST_FEE_IX_DATA_LEN: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetLstFeeIxData([u8; SET_LST_FEE_IX_DATA_LEN]);

impl SetLstFeeIxData {
    #[inline]
    pub const fn new(
        SetLstFeeIxArgs {
            inp_fee_nanos,
            out_fee_nanos,
        }: SetLstFeeIxArgs,
    ) -> Self {
        const A: usize = SET_LST_FEE_IX_DATA_LEN;

        let mut d = [0u8; A];

        d = caba::<A, 0, 1>(d, &[SET_LST_FEE_IX_DISCM]);
        d = caba::<A, 1, 4>(d, &inp_fee_nanos.to_le_bytes());
        d = caba::<A, 5, 4>(d, &out_fee_nanos.to_le_bytes());

        Self(d)
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; SET_LST_FEE_IX_DATA_LEN] {
        &self.0
    }
}
