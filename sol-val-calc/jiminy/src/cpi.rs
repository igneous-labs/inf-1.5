use core::ops::{Range, RangeInclusive};

use inf1_svc_core::instructions::{
    lst_to_sol::LstToSolIxData, sol_to_lst::SolToLstIxData, IxPreAccs, IX_DATA_LEN,
};
use jiminy_cpi::{
    account::{AccountHandle, Accounts},
    program_error::{ProgramError, BORSH_IO_ERROR, NOT_ENOUGH_ACCOUNT_KEYS},
    Cpi, CpiBuilder,
};
use jiminy_return_data::get_return_data;

#[inline]
pub fn cpi_sol_to_lst<'cpi, 'accounts, const MAX_CPI_ACCS: usize, const MAX_ACCS: usize>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    accounts: &'cpi mut Accounts<'accounts, MAX_ACCS>,
    svc_prog: AccountHandle<'accounts>,
    lamports: u64,
    ix_prefix: IxPreAccs<AccountHandle<'accounts>>,
    suf_range: &Range<usize>,
) -> Result<RangeInclusive<u64>, ProgramError> {
    prepare(
        cpi,
        accounts,
        svc_prog,
        SolToLstIxData::new(lamports).as_buf(),
        ix_prefix,
        suf_range,
    )
    .and_then(invoke)
}

#[inline]
pub fn cpi_lst_to_sol<'cpi, 'accounts, const MAX_CPI_ACCS: usize, const MAX_ACCS: usize>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    accounts: &'cpi mut Accounts<'accounts, MAX_ACCS>,
    svc_prog: AccountHandle<'accounts>,
    lst_amt: u64,
    ix_prefix: IxPreAccs<AccountHandle<'accounts>>,
    suf_range: &Range<usize>,
) -> Result<RangeInclusive<u64>, ProgramError> {
    prepare(
        cpi,
        accounts,
        svc_prog,
        LstToSolIxData::new(lst_amt).as_buf(),
        ix_prefix,
        suf_range,
    )
    .and_then(invoke)
}

// just splitting prepare() and invoke() into 2 fns here
// in case we need to expose them to public in the future

#[inline]
fn prepare<'cpi, 'accounts, const MAX_CPI_ACCS: usize, const MAX_ACCS: usize>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    accounts: &'cpi mut Accounts<'accounts, MAX_ACCS>,
    svc_prog: AccountHandle<'accounts>,
    ix_data: &'cpi [u8; IX_DATA_LEN],
    ix_prefix: IxPreAccs<AccountHandle<'accounts>>,
    suf_range: &Range<usize>,
) -> Result<CpiBuilder<'cpi, 'accounts, MAX_CPI_ACCS, MAX_ACCS, true>, ProgramError> {
    let mut res = CpiBuilder::new(cpi, accounts)
        .with_prog_handle(svc_prog)
        .with_ix_data(ix_data);
    res.try_derive_accounts_fwd(|accounts| {
        let suf = accounts
            .as_slice()
            .get(suf_range.clone())
            .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
        // very unfortunate we cant use
        // IxAccs<AccountHandle<'a>, S>
        // because cannot return reference to temporary in closure
        Ok(ix_prefix.0.into_iter().chain(suf.iter().copied()))
    })?;
    Ok(res)
}

#[inline]
fn invoke<const MAX_CPI_ACCS: usize, const MAX_ACCS: usize>(
    cpi: CpiBuilder<'_, '_, MAX_CPI_ACCS, MAX_ACCS, true>,
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
