use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::instruction::ExchangeBoothInstruction;

pub mod close_exchange_booth;
pub mod deposit;
pub mod exchange;
pub mod initialize_exchange_booth;
pub mod withdraw;

pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = ExchangeBoothInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match instruction {
            ExchangeBoothInstruction::InititializeExchangeBooth {
                vault_base_bump,
                vault_quote_bump,
                state_bump,
            } => {
                msg!("Instruction: InitializeExchangeBooth");
                initialize_exchange_booth::process(
                    program_id,
                    accounts,
                    state_bump,
                    vault_base_bump,
                    vault_quote_bump,
                )?;
            }
            ExchangeBoothInstruction::Deposit { mint, amount } => {
                msg!("Instruction: Deposit");
                deposit::process(program_id, accounts, &mint, amount)?;
            }
            ExchangeBoothInstruction::Withdraw { mint, amount } => {
                msg!("Instruction: Withdraw");
                withdraw::process(program_id, accounts, &mint, amount)?;
            }
            ExchangeBoothInstruction::Exchange { input_mint, amount } => {
                msg!("Instruction: Withdraw");
                exchange::process(program_id, accounts, &input_mint, amount)?;
            }
            ExchangeBoothInstruction::CloseExchangeBooth {} => {
                msg!("Instruction: CloseExchangeBooth");
                close_exchange_booth::process(program_id, accounts)?;
            }
        }

        Ok(())
    }
}
