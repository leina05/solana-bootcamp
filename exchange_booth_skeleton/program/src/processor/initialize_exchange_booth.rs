use solana_program::program_pack::Pack;
use solana_program::{account_info::next_account_info, program::invoke_signed, system_instruction};

use crate::processor::*;
use crate::utils::*;

use crate::{error::ExchangeBoothError, state::ExchangeBooth};
use spl_token::state::Account;

use borsh::{BorshDeserialize, BorshSerialize};

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // KeyPair accounts
    let admin_info = next_account_info(accounts_iter)?;
    let base_mint_info = next_account_info(accounts_iter)?;
    let quote_mint_info = next_account_info(accounts_iter)?;
    let oracle_info = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // PDA accounts
    let vault_base_info = next_account_info(accounts_iter)?;
    let vault_quote_info = next_account_info(accounts_iter)?;
    let state_info = next_account_info(accounts_iter)?;

    // Validate account inputs
    assert_is_signer(admin_info)?;
    assert_is_writable(vault_base_info)?;
    assert_is_writable(vault_quote_info)?;
    assert_is_writable(state_info)?;
    assert_is_system_program(system_program)?;
    assert_is_token_program(token_program)?;

    // Create PDAs
    let token_account_size = spl_token::state::Account::get_packed_len();
    // TODO:
    // invoke_signed(system_instruction::create_account(&admin_info.key, &vault_base_info.key, rent::get().get_minimum_balance_for_rent_exemption(token_account_size)), )

    Ok(())
}
