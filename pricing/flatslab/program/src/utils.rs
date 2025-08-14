use jiminy_entrypoint::account::AccountHandle;

use crate::Accounts;

#[inline]
pub fn verify_pks<'a, 'acc, const LEN: usize>(
    accounts: &Accounts<'acc>,
    handles: &'a [AccountHandle<'acc>; LEN],
    expected: &'a [[u8; 32]; LEN],
) -> Result<(), (&'a AccountHandle<'acc>, &'a [u8; 32])> {
    verify_pks_slice(accounts, handles, expected)
}

/// [`verify_pks`] delegates to this to minimize monomorphization  
fn verify_pks_slice<'a, 'acc>(
    accounts: &Accounts<'acc>,
    handles: &'a [AccountHandle<'acc>],
    expected: &'a [[u8; 32]],
) -> Result<(), (&'a AccountHandle<'acc>, &'a [u8; 32])> {
    handles.iter().zip(expected).try_for_each(|(h, e)| {
        if accounts.get(*h).key() == e {
            Ok(())
        } else {
            Err((h, e))
        }
    })
}
