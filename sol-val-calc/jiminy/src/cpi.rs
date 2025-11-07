use core::ops::RangeInclusive;

use inf1_svc_core::instructions::{
    lst_to_sol::LstToSolIxData, sol_to_lst::SolToLstIxData, IxAccs, IX_DATA_LEN,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, BORSH_IO_ERROR},
    Cpi, CpiBuilder,
};
use jiminy_return_data::get_return_data;

pub type IxAccountHandles<'a, P> = IxAccs<AccountHandle<'a>, P>;

#[inline]
pub fn cpi_sol_to_lst<'cpi, 'accounts, const MAX_CPI_ACCS: usize>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    abr: &'cpi mut Abr,
    svc_prog: AccountHandle<'accounts>,
    lamports: u64,
    accs: IxAccountHandles<'accounts, impl AsRef<[AccountHandle<'accounts>]>>,
) -> Result<RangeInclusive<u64>, ProgramError> {
    prepare(
        cpi,
        abr,
        svc_prog,
        SolToLstIxData::new(lamports).as_buf(),
        accs,
    )
    .and_then(invoke)
}

#[inline]
pub fn cpi_lst_to_sol<'cpi, 'accounts, const MAX_CPI_ACCS: usize>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    abr: &'cpi mut Abr,
    svc_prog: AccountHandle<'accounts>,
    lst_amt: u64,
    accs: IxAccountHandles<'accounts, impl AsRef<[AccountHandle<'accounts>]>>,
) -> Result<RangeInclusive<u64>, ProgramError> {
    prepare(
        cpi,
        abr,
        svc_prog,
        LstToSolIxData::new(lst_amt).as_buf(),
        accs,
    )
    .and_then(invoke)
}

// just splitting prepare() and invoke() into 2 fns here
// in case we need to expose them to public in the future

#[inline]
fn prepare<'cpi, 'accounts, const MAX_CPI_ACCS: usize>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    abr: &'cpi mut Abr,
    svc_prog: AccountHandle<'accounts>,
    ix_data: &'cpi [u8; IX_DATA_LEN],
    accs: IxAccountHandles<'accounts, impl AsRef<[AccountHandle<'accounts>]>>,
) -> Result<CpiBuilder<'cpi, MAX_CPI_ACCS, true>, ProgramError> {
    CpiBuilder::new(cpi, abr)
        .with_prog_handle(svc_prog)
        .with_ix_data(ix_data)
        .with_accounts_fwd(accs.seq().copied())
}

#[inline]
fn invoke<const MAX_CPI_ACCS: usize>(
    cpi: CpiBuilder<'_, MAX_CPI_ACCS, true>,
) -> Result<RangeInclusive<u64>, ProgramError> {
    cpi.invoke()?;
    let data_opt = get_return_data::<16>();
    let (min, max) = data_opt
        .as_ref()
        .map(|d| d.data())
        .and_then(|s| s.split_first_chunk::<8>())
        .and_then(|(min, rem)| {
            rem.split_first_chunk::<8>()
                .map(|(max, _rem_must_be_empty_because_ret_data_max_len_16)| (min, max))
        })
        .ok_or(BORSH_IO_ERROR)?;
    Ok(u64::from_le_bytes(*min)..=u64::from_le_bytes(*max))
}
