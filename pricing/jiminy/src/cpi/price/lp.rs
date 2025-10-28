use inf1_pp_core::instructions::{
    price::{
        exact_in::{PriceExactInIxArgs, PriceExactInIxData},
        exact_out::{PriceExactOutIxArgs, PriceExactOutIxData},
        IxAccs,
    },
    IX_DATA_LEN,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, BORSH_IO_ERROR},
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
pub fn cpi_price_exact_in<'cpi, 'accounts, const MAX_CPI_ACCS: usize>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    abr: &'cpi mut Abr,
    pricing_prog: AccountHandle<'accounts>,
    ix_args: PriceExactInIxArgs,
    accs: IxAccountHandles<'accounts, impl AsRef<[AccountHandle<'accounts>]>>,
) -> Result<u64, ProgramError> {
    prepare(
        cpi,
        abr,
        pricing_prog,
        PriceExactInIxData::new(ix_args).as_buf(),
        accs,
    )
    .and_then(invoke)
}

/// Price exact out using CPI
#[inline]
pub fn price_exact_out<'cpi, 'accounts, const MAX_CPI_ACCS: usize>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    abr: &'cpi mut Abr,
    pricing_prog: AccountHandle<'accounts>,
    ix_data: PriceExactOutIxArgs,
    accs: IxAccountHandles<'accounts, impl AsRef<[AccountHandle<'accounts>]>>,
) -> Result<u64, ProgramError> {
    prepare(
        cpi,
        abr,
        pricing_prog,
        PriceExactOutIxData::new(ix_data).as_buf(),
        accs,
    )
    .and_then(invoke)
}

#[inline]
fn prepare<'cpi, 'accounts, const MAX_CPI_ACCS: usize>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    abr: &'cpi mut Abr,
    pricing_prog: AccountHandle<'accounts>,
    ix_data: &'cpi [u8; IX_DATA_LEN],
    accs: IxAccountHandles<'accounts, impl AsRef<[AccountHandle<'accounts>]>>,
) -> Result<CpiBuilder<'cpi, MAX_CPI_ACCS, true>, ProgramError> {
    CpiBuilder::new(cpi, abr)
        .with_prog_handle(pricing_prog)
        .with_ix_data(ix_data)
        .with_accounts_fwd(accs.seq().copied())
}

// Invoke interface shared by all ixs in this lib
#[inline]
fn invoke<const MAX_CPI_ACCS: usize>(
    cpi: CpiBuilder<'_, MAX_CPI_ACCS, true>,
) -> Result<u64, ProgramError> {
    cpi.invoke()?;
    let data_opt = get_return_data::<8>();
    let price = data_opt
        .as_ref()
        // Map the data to bytes
        .map(|d| d.data())
        // Split first chunk for getting the bytes for a number
        .and_then(|s| s.first_chunk::<8>())
        .ok_or(BORSH_IO_ERROR)?;
    Ok(u64::from_le_bytes(*price))
}
