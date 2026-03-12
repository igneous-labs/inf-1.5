use generic_array_struct::generic_array_struct;

pub mod exact_in;
pub mod exact_out;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IxPreAccs<T> {
    pub signer: T,
    pub inp_lst_mint: T,
    pub out_lst_mint: T,
    pub inp_lst_acc: T,
    pub out_lst_acc: T,
    pub protocol_fee_accumulator: T,
    pub inp_lst_token_program: T,
    pub out_lst_token_program: T,
    pub pool_state: T,
    pub lst_state_list: T,
    pub inp_pool_reserves: T,
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
    .const_with_inp_lst_mint(false)
    .const_with_out_lst_mint(false)
    .const_with_inp_lst_token_program(false)
    .const_with_out_lst_token_program(false);

pub const IX_PRE_IS_SIGNER: IxPreAccFlags = IxPreAccFlags::memset(false).const_with_signer(true);
