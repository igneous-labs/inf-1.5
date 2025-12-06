use generic_array_struct::generic_array_struct;
use jiminy_sysvar_instructions::sysvar::OWNER_ID;
use solana_account::Account;
use solana_clock::Clock;
use solana_instruction::{BorrowedAccountMeta, BorrowedInstruction, Instruction};
use solana_instructions_sysvar::construct_instructions_data;
use solana_pubkey::Pubkey;

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockI64s<T> {
    pub epoch_start_timestamp: T,
    pub unix_timestamp: T,
}

impl<'a> ClockI64s<&'a mut i64> {
    pub fn muts(
        Clock {
            epoch_start_timestamp,
            unix_timestamp,
            ..
        }: &'a mut Clock,
    ) -> Self {
        NewClockI64sBuilder::start()
            .with_epoch_start_timestamp(epoch_start_timestamp)
            .with_unix_timestamp(unix_timestamp)
            .build()
    }
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockU64s<T> {
    pub slot: T,
    pub epoch: T,
    pub leader_schedule_epoch: T,
}

impl<'a> ClockU64s<&'a mut u64> {
    pub fn muts(
        Clock {
            slot,
            epoch,
            leader_schedule_epoch,
            ..
        }: &'a mut Clock,
    ) -> Self {
        NewClockU64sBuilder::start()
            .with_slot(slot)
            .with_epoch(epoch)
            .with_leader_schedule_epoch(leader_schedule_epoch)
            .build()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockArgs<I, U> {
    pub i64s: ClockI64s<I>,
    pub u64s: ClockU64s<U>,
}

impl From<ClockArgs<i64, u64>> for Clock {
    fn from(ClockArgs { u64s, i64s }: ClockArgs<i64, u64>) -> Self {
        Self {
            slot: *u64s.slot(),
            epoch_start_timestamp: *i64s.epoch_start_timestamp(),
            epoch: *u64s.epoch(),
            leader_schedule_epoch: *u64s.leader_schedule_epoch(),
            unix_timestamp: *i64s.unix_timestamp(),
        }
    }
}

pub fn override_clock(
    clock: &mut Clock,
    ClockArgs { i64s, u64s }: &ClockArgs<Option<i64>, Option<u64>>,
) {
    ClockI64s::muts(clock)
        .0
        .into_iter()
        .zip(i64s.0)
        .for_each(|(r, v)| {
            if let Some(v) = v {
                *r = v;
            }
        });
    ClockU64s::muts(clock)
        .0
        .into_iter()
        .zip(u64s.0)
        .for_each(|(r, v)| {
            if let Some(v) = v {
                *r = v;
            }
        });
}

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
