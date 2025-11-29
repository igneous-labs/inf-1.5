use generic_array_struct::generic_array_struct;

pub mod exact_out;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IxPreAccs<T> {
    pub signer: T,
    pub inp_mint: T,
    pub out_mint: T,
    pub inp_acc: T,
    pub out_acc: T,
    pub inp_token_program: T,
    pub out_token_program: T,
    pub pool_state: T,
    pub lst_state_list: T,

    /// Set to LP mint if `inp_lst_mint = LP mint`
    /// to enable write permissions for burning LP tokens
    pub inp_pool_reserves: T,

    /// Set to LP mint if `out_lst_mint = LP mint`
    /// to enable write permissions for minting LP tokens
    pub out_pool_reserves: T,
}

impl<T: Copy> IxPreAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; IX_PRE_ACCS_LEN])
    }
}

pub type IxPreKeys<'a> = IxPreAccs<&'a [u8; 32]>;

pub type IxPreKeysOwned = IxPreAccs<[u8; 32]>;

pub type IxPreAccFlags = IxPreAccs<bool>;

impl<T> AsRef<[T]> for IxPreAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

pub const IX_PRE_IS_WRITER: IxPreAccFlags = IxPreAccFlags::memset(true)
    .const_with_signer(false)
    .const_with_inp_mint(false)
    .const_with_out_mint(false)
    .const_with_inp_token_program(false)
    .const_with_out_token_program(false);

pub const IX_PRE_IS_SIGNER: IxPreAccFlags = IxPreAccFlags::memset(false).const_with_signer(true);

// v1 conversions

impl<T: Clone> IxPreAccs<T> {
    #[inline]
    pub fn clone_from_v1(v1: &super::v1::IxPreAccs<T>) -> Self {
        NewIxPreAccsBuilder::start()
            .with_inp_acc(v1.inp_lst_acc().clone())
            .with_inp_mint(v1.inp_lst_mint().clone())
            .with_inp_token_program(v1.inp_lst_token_program().clone())
            .with_inp_pool_reserves(v1.inp_pool_reserves().clone())
            .with_out_acc(v1.out_lst_acc().clone())
            .with_out_mint(v1.out_lst_mint().clone())
            .with_out_token_program(v1.out_lst_token_program().clone())
            .with_out_pool_reserves(v1.out_pool_reserves().clone())
            .with_pool_state(v1.pool_state().clone())
            .with_lst_state_list(v1.lst_state_list().clone())
            .with_signer(v1.signer().clone())
            .build()
    }

    #[inline]
    fn clone_from_liq_common(
        v1: &crate::instructions::liquidity::IxPreAccs<T>,
    ) -> IxPreAccsBuilder<T, true, false, false, false, false, false, false, true, true, false, false>
    {
        NewIxPreAccsBuilder::start()
            .with_lst_state_list(v1.lst_state_list().clone())
            .with_pool_state(v1.pool_state().clone())
            .with_signer(v1.signer().clone())
    }

    #[inline]
    pub fn clone_from_add_liq(v1: &crate::instructions::liquidity::IxPreAccs<T>) -> Self {
        Self::clone_from_liq_common(v1)
            .with_inp_acc(v1.lst_acc().clone())
            .with_inp_mint(v1.lst_mint().clone())
            .with_inp_token_program(v1.lst_token_program().clone())
            .with_inp_pool_reserves(v1.pool_reserves().clone())
            .with_out_acc(v1.lp_acc().clone())
            .with_out_mint(v1.lp_token_mint().clone())
            .with_out_token_program(v1.lp_token_program().clone())
            .with_out_pool_reserves(v1.lp_token_mint().clone())
            .build()
    }

    #[inline]
    pub fn clone_from_rem_liq(v1: &crate::instructions::liquidity::IxPreAccs<T>) -> Self {
        Self::clone_from_liq_common(v1)
            .with_out_acc(v1.lst_acc().clone())
            .with_out_mint(v1.lst_mint().clone())
            .with_out_token_program(v1.lst_token_program().clone())
            .with_out_pool_reserves(v1.pool_reserves().clone())
            .with_inp_acc(v1.lp_acc().clone())
            .with_inp_mint(v1.lp_token_mint().clone())
            .with_inp_token_program(v1.lp_token_program().clone())
            .with_inp_pool_reserves(v1.lp_token_mint().clone())
            .build()
    }
}

impl<T: Clone> From<&super::v1::IxPreAccs<T>> for IxPreAccs<T> {
    #[inline]
    fn from(v1: &super::v1::IxPreAccs<T>) -> Self {
        Self::clone_from_v1(v1)
    }
}

impl<T: Clone> From<super::v1::IxPreAccs<T>> for IxPreAccs<T> {
    #[inline]
    fn from(v1: super::v1::IxPreAccs<T>) -> Self {
        Self::clone_from_v1(&v1)
    }
}
