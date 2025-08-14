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
