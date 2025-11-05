use jiminy_sysvar_instructions::sysvar::OWNER_ID;
use solana_account::Account;
use solana_instruction::{BorrowedAccountMeta, BorrowedInstruction, Instruction};
use solana_instructions_sysvar::construct_instructions_data;
use solana_pubkey::Pubkey;

/// Clock with everything = 0
pub fn mock_clock() -> Account {
    Account {
        data: vec![0; 40],
        owner: Pubkey::from_str_const("Sysvar1111111111111111111111111111111111111"),
        executable: false,
        // dont-cares
        lamports: 1169280,
        rent_epoch: u64::MAX,
    }
}

pub fn mock_instructions_sysvar(instructions: &[Instruction], curr_idx: u16) -> Account {
    let mut data = construct_instructions_data(
        instructions
            .iter()
            .map(|instruction| BorrowedInstruction {
                program_id: &instruction.program_id,
                accounts: instruction
                    .accounts
                    .iter()
                    .map(|meta| BorrowedAccountMeta {
                        pubkey: &meta.pubkey,
                        is_signer: meta.is_signer,
                        is_writable: meta.is_writable,
                    })
                    .collect(),
                data: &instruction.data,
            })
            .collect::<Vec<_>>()
            .as_slice(),
    );

    *data.split_last_chunk_mut().unwrap().1 = curr_idx.to_le_bytes();

    Account {
        data,
        owner: Pubkey::new_from_array(OWNER_ID),
        lamports: 10_000_000,
        executable: false,
        rent_epoch: 0,
    }
}
