use std::thread::AccessError;

use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::EchoError;
use crate::instruction::EchoInstruction;

pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = EchoInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match instruction {
            EchoInstruction::Echo { data: data_to_copy } => {
                msg!("Instruction: Echo");
                let account_iter = &mut accounts.iter();
                let echo_buffer = next_account_info(account_iter)?;
                assert!(echo_buffer.is_writable);
                let mut echo_buffer_data = echo_buffer.data.borrow_mut();
                for (i, b) in echo_buffer_data.into_iter().enumerate() {
                    if *b != 0 {
                        panic!("Echo buffer is not empty.");
                    }
                    if let Some(val) = data_to_copy.get(i) {
                        *b = *val;
                    }
                }
                let echo_buffer_data_len = echo_buffer_data.len();
                msg!("Echo buffer size: {}", echo_buffer_data_len);
                Ok(())
            }
            EchoInstruction::InitializeAuthorizedEcho {
                buffer_seed: _,
                buffer_size: _,
            } => {
                msg!("Instruction: InitializeAuthorizedEcho");
                Err(EchoError::NotImplemented.into())
            }
            EchoInstruction::AuthorizedEcho { data: _ } => {
                msg!("Instruction: AuthorizedEcho");
                Err(EchoError::NotImplemented.into())
            }
            EchoInstruction::InitializeVendingMachineEcho {
                price: _,
                buffer_size: _,
            } => {
                msg!("Instruction: InitializeVendingMachineEcho");
                Err(EchoError::NotImplemented.into())
            }
            EchoInstruction::VendingMachineEcho { data: _ } => {
                msg!("Instruction: VendingMachineEcho");
                Err(EchoError::NotImplemented.into())
            }
        }
    }

}
