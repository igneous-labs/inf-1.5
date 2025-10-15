use core::ops::Range;

use inf1_pp_core::instructions::{
    price::{
        exact_in::{PriceExactInIxArgs, PriceExactInIxData},
        exact_out::{PriceExactOutIxArgs, PriceExactOutIxData},
        IxAccs, IxPreAccs,
    },
    IX_DATA_LEN,
};
use jiminy_cpi::{
    account::{AccountHandle, Accounts},
    program_error::{ProgramError, BORSH_IO_ERROR, NOT_ENOUGH_ACCOUNT_KEYS},
    Cpi, CpiBuilder,
};
use jiminy_return_data::get_return_data;

pub type IxAccountHandles<'a, P> = IxAccs<AccountHandle<'a>, P>;

/// `P: AsRef<[AccountHandle]>`
/// -> use [`PriceExactInIxAccountHandles::seq`] with [`jiminy_cpi::Cpi::invoke_fwd`]
pub type PriceExactInIxAccountHandles<'a, P> = IxAccountHandles<'a, P>;

/// `P: AsRef<[AccountHandle]>`
/// -> use [`PriceExactOutIxAccountHandles::seq`] with [`jiminy_cpi::Cpi::invoke_fwd`]
pub type PriceExactOutIxAccountHandles<'a, P> = IxAccountHandles<'a, P>;

// just splitting prepare() and invoke() into 2 fns here
// in case we need to expose them to public in the future

/// Price exact in using CPI
#[inline]
pub fn cpi_price_exact_in<'cpi, 'accounts, const MAX_CPI_ACCS: usize, const MAX_ACCS: usize>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    accounts: &'cpi mut Accounts<'accounts, MAX_ACCS>,
    svc_prog: AccountHandle<'accounts>,
    ix_args: PriceExactInIxArgs,
    ix_prefix: IxPreAccs<AccountHandle<'accounts>>,
    suf_range: Range<usize>,
) -> Result<u64, ProgramError> {
    prepare(
        cpi,
        accounts,
        svc_prog,
        PriceExactInIxData::new(ix_args).as_buf(),
        ix_prefix,
        suf_range,
    )
    .and_then(invoke)
}

/// Price exact out using CPI
#[inline]
pub fn price_exact_out<'cpi, 'accounts, const MAX_CPI_ACCS: usize, const MAX_ACCS: usize>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    accounts: &'cpi mut Accounts<'accounts, MAX_ACCS>,
    svc_prog: AccountHandle<'accounts>,
    ix_data: PriceExactOutIxArgs,
    ix_prefix: IxPreAccs<AccountHandle<'accounts>>,
    suf_range: Range<usize>,
) -> Result<u64, ProgramError> {
    prepare(
        cpi,
        accounts,
        svc_prog,
        PriceExactOutIxData::new(ix_data).as_buf(),
        ix_prefix,
        suf_range,
    )
    .and_then(invoke)
}

#[inline]
fn prepare<'cpi, 'accounts, const MAX_CPI_ACCS: usize, const MAX_ACCS: usize>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    accounts: &'cpi mut Accounts<'accounts, MAX_ACCS>,
    svc_prog: AccountHandle<'accounts>,
    ix_data: &'cpi [u8; IX_DATA_LEN],
    ix_prefix: IxPreAccs<AccountHandle<'accounts>>,
    suf_range: Range<usize>,
) -> Result<CpiBuilder<'cpi, 'accounts, MAX_CPI_ACCS, MAX_ACCS, true>, ProgramError> {
    let mut res = CpiBuilder::new(cpi, accounts)
        .with_prog_handle(svc_prog)
        .with_ix_data(ix_data);
    res.try_derive_accounts_fwd(|accounts| {
        let suf = accounts
            .as_slice()
            .get(suf_range)
            .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
        // very unfortunate we cant use
        // IxAccs<AccountHandle<'a>, S>
        // because cannot return reference to temporary in closure
        Ok(ix_prefix.0.into_iter().chain(suf.iter().copied()))
    })?;
    Ok(res)
}

// Invoke interface shared by all ixs in this lib
#[inline]
fn invoke<const MAX_CPI_ACCS: usize, const MAX_ACCS: usize>(
    cpi: CpiBuilder<'_, '_, MAX_CPI_ACCS, MAX_ACCS, true>,
) -> Result<u64, ProgramError> {
    cpi.invoke()?;
    let data_opt = get_return_data::<16>();
    let price = data_opt
        .as_ref()
        // Map the data to bytes
        .map(|d| d.data())
        // Split first chunk for getting the bytes for a number
        .and_then(|s| s.split_first_chunk::<8>())
        .ok_or(BORSH_IO_ERROR)?;
    Ok(u64::from_le_bytes(*price.0))
}
