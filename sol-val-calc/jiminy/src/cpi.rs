use core::ops::RangeInclusive;

use inf1_svc_core::instructions::{
    lst_to_sol::LstToSolIxData, sol_to_lst::SolToLstIxData, IxAccs, IX_DATA_LEN,
};
use jiminy_cpi::{
    account::{AccountHandle, Accounts},
    program_error::{ProgramError, BORSH_IO_ERROR},
    Cpi, CpiBuilder, PreparedCpi,
};
use jiminy_return_data::get_return_data;

/// `S: AsRef<[AccountHandle]>`
/// -> use [`IxAccountHandles::seq`] with [`jiminy_cpi::Cpi::invoke_fwd`]
pub type SvcIxAccountHandles<'a, S> = IxAccs<AccountHandle<'a>, S>;

// creating a newtype here enforces that the contained `PreparedCpi`
// came from `prep_cpi`
/// A prepared SOL value calculator program interface CPI
#[derive(Debug)]
#[repr(transparent)]
pub struct SvcPreparedCpi<'cpi, const MAX_CPI_ACCS: usize>(PreparedCpi<'cpi, MAX_CPI_ACCS>);

/// # Safety
/// - Same rules as [`CpiBuilder::build`]
#[inline]
pub unsafe fn prep_cpi_sol_to_lst<
    'cpi,
    'accounts,
    const MAX_CPI_ACCS: usize,
    const MAX_ACCS: usize,
    S: AsRef<[AccountHandle<'accounts>]>,
>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    accounts: &Accounts<'accounts, MAX_ACCS>,
    accs: SvcIxAccountHandles<'accounts, S>,
    prog_id: &[u8; 32],
    lamports: u64,
) -> Result<SvcPreparedCpi<'cpi, MAX_CPI_ACCS>, ProgramError> {
    prep_cpi(
        cpi,
        accounts,
        accs,
        prog_id,
        SolToLstIxData::new(lamports).as_buf(),
    )
}

/// # Safety
/// - Same rules as [`CpiBuilder::build`]
#[inline]
pub unsafe fn prep_cpi_lst_to_sol<
    'cpi,
    'accounts,
    const MAX_CPI_ACCS: usize,
    const MAX_ACCS: usize,
    S: AsRef<[AccountHandle<'accounts>]>,
>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    accounts: &Accounts<'accounts, MAX_ACCS>,
    accs: SvcIxAccountHandles<'accounts, S>,
    prog_id: &[u8; 32],
    lst_amt: u64,
) -> Result<SvcPreparedCpi<'cpi, MAX_CPI_ACCS>, ProgramError> {
    prep_cpi(
        cpi,
        accounts,
        accs,
        prog_id,
        LstToSolIxData::new(lst_amt).as_buf(),
    )
}

/// # Safety
/// - Same rules as [`CpiBuilder::build`]
#[inline]
unsafe fn prep_cpi<
    'cpi,
    'accounts,
    const MAX_CPI_ACCS: usize,
    const MAX_ACCS: usize,
    S: AsRef<[AccountHandle<'accounts>]>,
>(
    cpi: &'cpi mut Cpi<MAX_CPI_ACCS>,
    accounts: &Accounts<'accounts, MAX_ACCS>,
    accs: SvcIxAccountHandles<'accounts, S>,
    prog_id: &[u8; 32],
    ix_data: &[u8; IX_DATA_LEN],
) -> Result<SvcPreparedCpi<'cpi, MAX_CPI_ACCS>, ProgramError> {
    Ok(SvcPreparedCpi(
        CpiBuilder::new(cpi, accounts)
            .with_prog_id(prog_id)
            .with_ix_data(ix_data)
            .try_with_accounts_fwd(accs.seq().copied())?
            .build(),
    ))
}

impl<const MAX_CPI_ACCS: usize> SvcPreparedCpi<'_, MAX_CPI_ACCS> {
    #[inline]
    pub fn invoke<const MAX_ACCS: usize>(
        self,
        accounts: &mut Accounts<'_, MAX_ACCS>,
    ) -> Result<RangeInclusive<u64>, ProgramError> {
        self.0.invoke(accounts)?;
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
}
