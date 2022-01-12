// use core::slice::SlicePattern;
// use std::thread::AccessError;

use borsh::BorshDeserialize;
// use num_traits::ToPrimitive;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    system_program::ID as SYSTEM_PROGRAM_ID,
    sysvar::{rent::Rent, Sysvar},
};

use crate::error::EchoError;
use crate::instruction::EchoInstruction;

pub fn assert_with_msg(statement: bool, err: ProgramError, msg: &str) -> ProgramResult {
    if !statement {
        msg!(msg);
        Err(err)
    } else {
        Ok(())
    }
}

pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = EchoInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        let accounts_iter = &mut accounts.iter();
        match instruction {
            EchoInstruction::Echo { data } => {
                msg!("Instruction: Echo");
                // Get account info for each account
                let echo_buffer_info = next_account_info(accounts_iter)?;

                // Validate accounts input
                // TODO: handle this with a real error
                assert!(echo_buffer_info.is_writable);

                // Write the data to the buffer
                let mut echo_buffer_data = echo_buffer_info.data.borrow_mut();
                for (i, b) in echo_buffer_data.into_iter().enumerate() {
                    if *b != 0 {
                        // TODO: handle with a real error
                        panic!("Echo buffer is not empty.");
                    }
                    if let Some(val) = data.get(i) {
                        *b = *val;
                    }
                }
                Ok(())
            }
            EchoInstruction::InitializeAuthorizedEcho {
                buffer_seed,
                buffer_size,
            } => {
                msg!("Instruction: InitializeAuthorizedEcho");

                let buffer_seed_bytes = u64::to_le_bytes(buffer_seed);

                // Get account info for each account
                let authorized_buffer_info = next_account_info(accounts_iter)?;
                let authority_info = next_account_info(accounts_iter)?;
                let system_program_info = next_account_info(accounts_iter)?;

                // Validate account inputs
                assert_with_msg(
                    authorized_buffer_info.is_writable,
                    ProgramError::InvalidArgument,
                    "Authorized buffer account must be writable.",
                )?;
                assert_with_msg(
                    authority_info.is_signer,
                    ProgramError::MissingRequiredSignature,
                    "Missing authority signature.",
                )?;
                assert_with_msg(
                    *system_program_info.key == SYSTEM_PROGRAM_ID,
                    ProgramError::InvalidArgument,
                    &format!(
                        "Expected System Program, received: {}",
                        system_program_info.key
                    ),
                )?;

                // Find PDA address for authorized_buffer
                let mut seeds: Vec<&[u8]> = vec![
                    b"authority",
                    authority_info.key.as_ref(),
                    &buffer_seed_bytes,
                ];

                let (authorized_buffer_key, bump_seed) =
                    Pubkey::find_program_address(seeds.as_slice(), program_id);

                // Confirm that the PDA address we found matches the one passed into the program
                assert_with_msg(
                    authorized_buffer_key == *authorized_buffer_info.key,
                    ProgramError::InvalidArgument,
                    "Invalid authorized_buffer address.",
                )?;

                // Create the authorized_buffer account
                seeds.push(&[bump_seed]);
                assert_with_msg(
                    buffer_size > 9,
                    ProgramError::InvalidInstructionData,
                    "Buffer size must be > 9.",
                )?;
                invoke_signed(
                    &system_instruction::create_account(
                        // Set authority as fee payer
                        authority_info.key,
                        &authorized_buffer_key,
                        Rent::get()?.minimum_balance(buffer_size as usize),
                        buffer_size,
                        program_id,
                    ),
                    &[
                        authority_info.clone(),
                        authorized_buffer_info.clone(),
                        system_program_info.clone(),
                    ],
                    &[&[
                        b"authority",
                        authority_info.key.as_ref(),
                        &buffer_seed_bytes,
                        &[bump_seed],
                    ]],
                )?;
                let mut authorized_buffer_data = authorized_buffer_info.try_borrow_mut_data()?;

                // Set first byte of authorized buffer to bump seed
                // TODO: use struct here
                // let auth_buffer_header = AuthorizedBufferHeader {
                //     bump_seed,
                //     buffer_seed,
                // };
                authorized_buffer_data[0] = bump_seed;
                authorized_buffer_data[1..9].copy_from_slice(&buffer_seed_bytes);

                Ok(())
            }
            EchoInstruction::AuthorizedEcho { data: _ } => {
                msg!("Instruction: AuthorizedEcho");
                // TODO: is it secure to not store the authority PK when we create the PDA?
                // I think so, because using the authority PK as a seed effectively stores the PK in the PDA

                // Get account info for each account
                let authorized_buffer_info = next_account_info(accounts_iter)?;
                let authority_info = next_account_info(accounts_iter)?;

                // Validate account inputs
                assert_with_msg(
                    authorized_buffer_info.is_writable,
                    ProgramError::InvalidArgument,
                    "Authorized buffer account must be writable.",
                )?;
                assert_with_msg(
                    authority_info.is_signer,
                    ProgramError::MissingRequiredSignature,
                    "Missing authority signature.",
                )?;

                // Derive PDA to confirm authority
                // Get bump_seed and buffer_seed from authorized_buffer data
                let authorized_buffer_data = authorized_buffer_info.try_borrow_data()?;
                let bump_seed = authorized_buffer_data[0];
                let buffer_seed_bytes: [u8; 8] = authorized_buffer_data[1..9].try_into().map_err(
                    |e: std::array::TryFromSliceError| {
                        println!("{:?}", e);
                        ProgramError::Custom(0)
                    },
                )?;
                let seeds = &[
                    b"authority",
                    authority_info.key.as_ref(),
                    &buffer_seed_bytes,
                    &[bump_seed],
                ];
                let authorized_buffer_key = Pubkey::create_program_address(seeds, program_id)?;

                // Confirm that the PDA address we found matches the one passed into the program
                assert_with_msg(
                    authorized_buffer_key == *authorized_buffer_info.key,
                    ProgramError::InvalidArgument,
                    "Invalid authorized_buffer address.",
                )?;

                // TODO: all checks are done, write to the buffer

                Ok(())
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
