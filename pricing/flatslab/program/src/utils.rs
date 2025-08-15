use jiminy_entrypoint::account::AccountHandle;

use crate::Accounts;

/// SystemInstruction::transfer
const MAX_CPI_ACCS: usize = 2;

pub type Cpi = jiminy_cpi::Cpi<MAX_CPI_ACCS>;

pub const SYS_PROG_ID: [u8; 32] = [0u8; 32];

#[inline]
pub fn verify_pks<'a, 'acc, const LEN: usize>(
    accounts: &Accounts<'acc>,
    handles: &'a [AccountHandle<'acc>; LEN],
    expected: &'a [&[u8; 32]; LEN], // we can use &[u8; 32] instead of [u8; 32] here bec we dont have any dynamic PDAs to verify
) -> Result<(), (&'a AccountHandle<'acc>, &'a [u8; 32])> {
    verify_pks_slice(accounts, handles, expected)
}

/// [`verify_pks`] delegates to this to minimize monomorphization  
fn verify_pks_slice<'a, 'acc>(
    accounts: &Accounts<'acc>,
    handles: &'a [AccountHandle<'acc>],
    expected: &'a [&[u8; 32]],
) -> Result<(), (&'a AccountHandle<'acc>, &'a [u8; 32])> {
    handles.iter().zip(expected).try_for_each(|(h, e)| {
        if accounts.get(*h).key() == *e {
            Ok(())
        } else {
            Err((h, *e))
        }
    })
}

#[inline]
pub fn verify_signers<'a, 'acc, const LEN: usize>(
    accounts: &Accounts<'acc>,
    handles: &'a [AccountHandle<'acc>; LEN],
    expected_is_signer: &'a [bool],
) -> Result<(), &'a AccountHandle<'acc>> {
    verify_signers_slice(accounts, handles, expected_is_signer)
}

/// [`verify_signers`] delegates to this to minimize monomorphization
fn verify_signers_slice<'a, 'acc>(
    accounts: &Accounts<'acc>,
    handles: &'a [AccountHandle<'acc>],
    expected_is_signer: &'a [bool],
) -> Result<(), &'a AccountHandle<'acc>> {
    handles
        .iter()
        .zip(expected_is_signer)
        .try_for_each(|(h, should_be_signer)| {
            if *should_be_signer && !accounts.get(*h).is_signer() {
                Err(h)
            } else {
                Ok(())
            }
        })
}
