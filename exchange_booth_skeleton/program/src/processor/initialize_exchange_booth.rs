use solana_program::program_pack::Pack;
use solana_program::{
    account_info::next_account_info,
    program::{invoke, invoke_signed},
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use crate::processor::*;
use crate::utils::*;

use crate::{error::ExchangeBoothError, state::ExchangeBooth};
use spl_token::state::Account;

use borsh::{BorshDeserialize, BorshSerialize};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    state_bump: u8,
    vault_base_bump: u8,
    vault_quote_bump: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // KeyPair accounts
    let admin_info = next_account_info(accounts_iter)?;
    let mint_base_info = next_account_info(accounts_iter)?;
    let mint_quote_info = next_account_info(accounts_iter)?;
    let oracle_info = next_account_info(accounts_iter)?;
    let token_program_info = next_account_info(accounts_iter)?;
    let system_program_info = next_account_info(accounts_iter)?;
    let rent_sysvar = next_account_info(accounts_iter)?;

    // PDA accounts
    let state_info = next_account_info(accounts_iter)?;
    let vault_base_info = next_account_info(accounts_iter)?;
    let vault_quote_info = next_account_info(accounts_iter)?;

    // Validate account inputs
    assert_is_signer(admin_info)?;
    assert_is_writable(vault_base_info)?;
    assert_is_writable(vault_quote_info)?;
    assert_is_writable(state_info)?;
    assert_is_system_program(system_program_info)?;
    assert_is_token_program(token_program_info)?;

    msg!("Done validating account infos.");

    // Create PDAs

    // Create state_info account (PDA)
    let state_info_size = ExchangeBooth::get_serialized_size()?;
    invoke_signed(
        // # Account references
        //   0. `[WRITE, SIGNER]` Funding account
        //   1. `[WRITE, SIGNER]` New account
        &system_instruction::create_account(
            &admin_info.key,
            &state_info.key,
            Rent::get()?.minimum_balance(state_info_size),
            state_info_size as u64,
            &program_id,
        ),
        &[admin_info.clone(), state_info.clone()],
        &[&[
            b"state_info",
            admin_info.key.as_ref(),
            mint_base_info.key.as_ref(),
            mint_quote_info.key.as_ref(),
            oracle_info.key.as_ref(),
            &[state_bump],
        ]],
    )?;

    msg!("Created state_info account.");

    // Create vault_base account (PDA)
    let token_account_size = spl_token::state::Account::get_packed_len();
    invoke_signed(
        // # Account references
        //   0. `[WRITE, SIGNER]` Funding account
        //   1. `[WRITE, SIGNER]` New account
        &system_instruction::create_account(
            &admin_info.key,
            &vault_base_info.key,
            Rent::get()?.minimum_balance(token_account_size),
            token_account_size as u64,
            &spl_token::id(),
        ),
        &[admin_info.clone(), vault_base_info.clone()],
        &[&[
            b"vault_base",
            state_info.key.as_ref(),
            mint_base_info.key.as_ref(),
            &[vault_base_bump],
        ]],
    )?;
    msg!("Created vault_base account.");
    // Initialize vault_base token account
    invoke(
        //   0. `[writable]`  The account to initialize.
        //   1. `[]` The mint this account will be associated with.
        //   2. `[]` The new account's owner/multisignature.
        //   3. `[]` Rent sysvar
        &spl_token::instruction::initialize_account(
            &spl_token::id(),
            &vault_base_info.key,
            &mint_base_info.key,
            &spl_token::id(),
        )?,
        &[
            vault_base_info.clone(),
            mint_base_info.clone(),
            vault_base_info.clone(),
            rent_sysvar.clone(),
        ],
    )?;
    msg!("Initialized vault_base token account.");
    // Create vault_quote
    invoke_signed(
        // # Account references
        //   0. `[WRITE, SIGNER]` Funding account
        //   1. `[WRITE, SIGNER]` New account
        &system_instruction::create_account(
            &admin_info.key,
            &vault_quote_info.key,
            Rent::get()?.minimum_balance(token_account_size),
            token_account_size as u64,
            &spl_token::id(),
        ),
        &[admin_info.clone(), vault_quote_info.clone()],
        &[&[
            b"vault_quote",
            state_info.key.as_ref(),
            mint_quote_info.key.as_ref(),
            &[vault_quote_bump],
        ]],
    )?;
    msg!("Created vault_quote account.");
    // Initialize vault_quote token account
    invoke(
        //   0. `[writable]`  The account to initialize.
        //   1. `[]` The mint this account will be associated with.
        //   2. `[]` The new account's owner/multisignature.
        //   3. `[]` Rent sysvar
        &spl_token::instruction::initialize_account(
            &spl_token::id(),
            &vault_quote_info.key,
            &mint_quote_info.key,
            &spl_token::id(),
        )?,
        &[
            vault_quote_info.clone(),
            mint_quote_info.clone(),
            vault_quote_info.clone(),
            rent_sysvar.clone(),
        ],
    )?;
    msg!("Initialized vault_quote token account.");

    // Get decimals
    let decimals_base =
        spl_token::state::Mint::unpack_from_slice(&mint_base_info.try_borrow_data()?)?.decimals;
    let decimals_quote =
        spl_token::state::Mint::unpack_from_slice(&mint_quote_info.try_borrow_data()?)?.decimals;
    let fee = 0;

    // Save state
    let state_struct = ExchangeBooth {
        admin: admin_info.key.clone(),
        mint_base: mint_base_info.key.clone(),
        decimals_base,
        mint_quote: mint_quote_info.key.clone(),
        decimals_quote,
        oracle: oracle_info.key.clone(),
        fee,
    };

    state_struct.serialize(&mut *state_info.try_borrow_mut_data()?)?;

    Ok(())
}
