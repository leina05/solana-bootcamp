// use core::slice::SlicePattern;
// use std::thread::AccessError;

use borsh::{BorshDeserialize, BorshSerialize};
// use num_traits::ToPrimitive;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    system_program::ID as SYSTEM_PROGRAM_ID,
    sysvar::{rent::Rent, Sysvar},
};

use spl_token::{instruction::burn, ID as TOKEN_PROGRAM_ID};

use crate::error::EchoError;
use crate::instruction::EchoInstruction;
use crate::state::{AuthorizedBuffer, VendingMachineBuffer};

pub fn assert_with_msg(statement: bool, err: ProgramError, msg: &str) -> ProgramResult {
    if !statement {
        msg!(msg);
        Err(err)
    } else {
        Ok(())
    }
}

pub fn assert_is_signer(account_info: &AccountInfo) -> ProgramResult {
    assert_with_msg(
        account_info.is_signer,
        ProgramError::MissingRequiredSignature,
        &format!("Missing signature for account {}.", account_info.key),
    )
}

pub fn assert_is_writable(account_info: &AccountInfo) -> ProgramResult {
    assert_with_msg(
        account_info.is_writable,
        ProgramError::InvalidArgument,
        &format!("Account {} must be writable.", account_info.key),
    )
}

pub fn assert_is_system_program(account_info: &AccountInfo) -> ProgramResult {
    assert_with_msg(
        *account_info.key == SYSTEM_PROGRAM_ID,
        ProgramError::InvalidArgument,
        &format!("Expected System Program, received: {}", account_info.key),
    )
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
                assert_is_writable(authorized_buffer_info)?;
                assert_is_signer(authority_info)?;
                assert_is_system_program(system_program_info)?;

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
                let bump_seed_bytes = [bump_seed];
                seeds.push(&bump_seed_bytes);
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
                    &[seeds.as_slice()],
                )?;

                // Get authorized_buffer account data
                let mut authorized_buffer_data = authorized_buffer_info.try_borrow_mut_data()?;

                // Get size of echo_buffer: length_of_account_data - bytes_allocated_for_seeds - bytes_used_for_vec_size
                let echo_buffer_size = authorized_buffer_data.len() - 9 - 4 as usize;

                // Set first byte of authorized buffer to bump seed
                let auth_buffer_struct = AuthorizedBuffer {
                    bump_seed,
                    buffer_seed,
                    data: vec![0; echo_buffer_size],
                };
                auth_buffer_struct.serialize(&mut *authorized_buffer_data)?;
                // authorized_buffer_data[0] = bump_seed;
                // authorized_buffer_data[1..9].copy_from_slice(&buffer_seed_bytes);

                Ok(())
            }
            EchoInstruction::AuthorizedEcho { data } => {
                msg!("Instruction: AuthorizedEcho");
                // TODO: is it secure to not store the authority PK when we create the PDA?
                // I think so, because using the authority PK as a seed effectively stores the PK in the PDA

                // Get account info for each account
                let authorized_buffer_info = next_account_info(accounts_iter)?;
                let authority_info = next_account_info(accounts_iter)?;

                // Validate account inputs
                assert_is_writable(authorized_buffer_info)?;
                assert_is_signer(authority_info)?;

                // Derive PDA to confirm authority
                // Get bump_seed and buffer_seed from authorized_buffer data
                let mut authorized_buffer_data = authorized_buffer_info.try_borrow_mut_data()?;
                let mut auth_buffer_struct =
                    AuthorizedBuffer::try_from_slice(&authorized_buffer_data)?;
                // let bump_seed = authorized_buffer_data[0];
                let buffer_seed_bytes = auth_buffer_struct.buffer_seed.to_le_bytes();
                let seeds = &[
                    b"authority",
                    authority_info.key.as_ref(),
                    &buffer_seed_bytes,
                    &[auth_buffer_struct.bump_seed],
                ];
                let authorized_buffer_key = Pubkey::create_program_address(seeds, program_id)?;

                // Confirm that the PDA address we found matches the one passed into the program
                assert_with_msg(
                    authorized_buffer_key == *authorized_buffer_info.key,
                    ProgramError::InvalidArgument,
                    "Invalid authorized_buffer address.",
                )?;

                // all checks are done, write to the buffer
                // let mut buffer_data: Vec<u8> = Vec::new();
                for i in 0..auth_buffer_struct.data.len() {
                    let data_i = i % data.len();
                    println!(
                        "authorized_buffer_len: {}\ndata_len: {}\ni: {}\ndata_i: {}",
                        auth_buffer_struct.data.len(),
                        data.len(),
                        i,
                        data_i
                    );
                    auth_buffer_struct.data[i] = data[data_i];
                }
                auth_buffer_struct.serialize(&mut *authorized_buffer_data)?;

                Ok(())
            }
            EchoInstruction::InitializeVendingMachineEcho { price, buffer_size } => {
                msg!("Instruction: InitializeVendingMachineEcho");
                let vm_buffer_info = next_account_info(accounts_iter)?;
                let vm_mint_info = next_account_info(accounts_iter)?;
                let payer_info = next_account_info(accounts_iter)?;
                let system_program_info = next_account_info(accounts_iter)?;

                assert_is_writable(vm_buffer_info)?;
                assert_is_signer(payer_info)?;
                assert_is_system_program(system_program_info)?;

                let price_bytes = price.to_le_bytes();
                let mut seeds: Vec<&[u8]> =
                    vec![b"vending_machine", vm_mint_info.key.as_ref(), &price_bytes];

                let (vm_buffer_key, bump_seed) =
                    Pubkey::find_program_address(seeds.as_slice(), program_id);

                assert_with_msg(
                    vm_buffer_key == *vm_buffer_info.key,
                    ProgramError::InvalidArgument,
                    "Invalid Vending Machine Buffer account.",
                )?;
                let bump_seed_bytes = [bump_seed];
                seeds.push(&bump_seed_bytes);

                invoke_signed(
                    &system_instruction::create_account(
                        // Set authority as fee payer
                        payer_info.key,
                        &vm_buffer_key,
                        Rent::get()?.minimum_balance(buffer_size as usize),
                        buffer_size as u64,
                        program_id,
                    ),
                    &[
                        payer_info.clone(),
                        vm_buffer_info.clone(),
                        system_program_info.clone(),
                    ],
                    &[seeds.as_slice()],
                )?;

                // Get authorized_buffer account data
                let mut vm_buffer_data = vm_buffer_info.try_borrow_mut_data()?;

                // Get size of echo_buffer: length_of_account_data - bytes_allocated_for_seeds - bytes_used_for_vec_size
                let buffer_data_size = vm_buffer_data.len() - 9 - 4 as usize;

                // Set first byte of authorized buffer to bump seed
                let vm_buffer_struct = VendingMachineBuffer {
                    bump_seed,
                    price,
                    data: vec![0; buffer_data_size],
                };
                vm_buffer_struct.serialize(&mut *vm_buffer_data)?;
                // authorized_buffer_data[0] = bump_seed;
                // authorized_buffer_data[1..9].copy_from_slice(&buffer_seed_bytes);

                Ok(())
            }
            EchoInstruction::VendingMachineEcho { data } => {
                msg!("Instruction: VendingMachineEcho");
                let vm_buffer_info = next_account_info(accounts_iter)?;
                let user_info = next_account_info(accounts_iter)?;
                let user_token_account_info = next_account_info(accounts_iter)?;
                let vm_mint_info = next_account_info(accounts_iter)?;
                let token_program_info = next_account_info(accounts_iter)?;

                assert_is_writable(vm_buffer_info)?;
                assert_is_writable(user_token_account_info)?;
                assert_is_writable(vm_mint_info)?;
                assert_is_signer(user_info)?;

                let mut vm_buffer_data = vm_buffer_info.try_borrow_mut_data()?;
                let mut vm_buffer_struct = VendingMachineBuffer::try_from_slice(&vm_buffer_data)?;
                let price_bytes = vm_buffer_struct.price.to_le_bytes();
                let seeds = &[
                    b"vending_machine",
                    vm_mint_info.key.as_ref(),
                    &price_bytes,
                    &[vm_buffer_struct.bump_seed],
                ];
                let vm_buffer_key = Pubkey::create_program_address(seeds, program_id)?;

                // Confirm that the PDA address we found matches the one passed into the program
                assert_with_msg(
                    vm_buffer_key == *vm_buffer_info.key,
                    ProgramError::InvalidArgument,
                    "Invalid vm_buffer address.",
                )?;

                // All checks done, burn token
                invoke(
                    &burn(
                        &TOKEN_PROGRAM_ID,
                        &user_token_account_info.key,
                        &vm_mint_info.key,
                        &user_info.key,
                        &[user_info.key],
                        vm_buffer_struct.price,
                    )?,
                    &[
                        user_token_account_info.clone(),
                        vm_mint_info.clone(),
                        user_info.clone(),
                        token_program_info.clone(),
                    ],
                )?;

                // Copy data to buffer
                for i in 0..vm_buffer_struct.data.len() {
                    let data_i = i % data.len();
                    vm_buffer_struct.data[i] = data[data_i];
                }
                vm_buffer_struct.serialize(&mut *vm_buffer_data)?;

                Ok(())
            }
        }
    }
}
