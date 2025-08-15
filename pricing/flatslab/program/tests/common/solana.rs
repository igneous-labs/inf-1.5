use jiminy_entrypoint::program_error::ProgramError;
use solana_account::Account;
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;

pub fn keys_signer_writable_to_metas<'a>(
    keys: impl Iterator<Item = &'a [u8; 32]>,
    signer: impl Iterator<Item = &'a bool>,
    writable: impl Iterator<Item = &'a bool>,
) -> Vec<AccountMeta> {
    keys.zip(signer)
        .zip(writable)
        .map(|((key, signer), writable)| AccountMeta {
            pubkey: Pubkey::new_from_array(*key),
            is_signer: *signer,
            is_writable: *writable,
        })
        .collect()
}

pub fn assert_prog_err_eq(sol: solana_program_error::ProgramError, us: ProgramError) {
    // TODO: implement `From<&ProgramError> for u64` in solana_program_error upstream so we dont need to clone
    assert_eq!(u64::from(sol.clone()), us.0.get(), "{sol}, {us:#?}");
}

pub fn slab_account(slab_data: Vec<u8>) -> Account {
    Account {
        data: slab_data,
        owner: Pubkey::new_from_array(inf1_pp_flatslab_core::ID),
        lamports: u64::MAX / 2, // dont rly care, long as its enough to be rent exempt
        executable: false,
        rent_epoch: u64::MAX,
    }
}
