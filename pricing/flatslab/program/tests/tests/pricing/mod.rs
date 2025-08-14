use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

mod exact_in;

pub fn ix_accs(ix: &Instruction, slab_data: Vec<u8>) -> [(Pubkey, Account); 3] {
    let ([inp, out, slab], _) = ix.accounts.split_first_chunk().unwrap();
    [
        (inp.pubkey, Account::default()),
        (out.pubkey, Account::default()),
        (
            slab.pubkey,
            Account {
                data: slab_data,
                owner: Pubkey::new_from_array(inf1_pp_flatslab_core::ID),
                lamports: u64::MAX / 2, // dont rly care, long as its enough to be rent exempt
                executable: false,
                rent_epoch: u64::MAX,
            },
        ),
    ]
}
