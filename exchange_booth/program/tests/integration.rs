use solana_program::program_pack::Pack;

// #![cfg(feature = "test-bpf")]

use {
    assert_matches::*,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_instruction, system_program, sysvar,
    },
    solana_sdk::{
        signature::{Keypair, Signer},
        transaction::Transaction,
    },
    solana_validator::test_validator::*,
    spl_token::state::Mint,
};

use exchangebooth::{instruction::ExchangeBoothInstruction, state::ExchangeBooth};

#[test]
fn test_validator_transaction() -> anyhow::Result<()> {
    // Set up test validator
    solana_logger::setup_with_default("solana_program_runtime=debug");
    let eb_program_id = Pubkey::new_unique();

    let (test_validator, admin) = TestValidatorGenesis::default()
        .add_program("exchangebooth", eb_program_id)
        .start();
    let rpc_client = test_validator.get_rpc_client();

    let blockhash = rpc_client.get_latest_blockhash().unwrap();

    // Create account keypairs
    let mint_base = Keypair::new();
    let mint_quote = Keypair::new();

    // TODO: add oracle program to test validator
    let oracle_pk = Pubkey::new_unique();

    println!()

    // Create instructions
    let mint_account_size = Mint::get_packed_len();
    let create_mint_base_ix = system_instruction::create_account(
        &admin.pubkey(),
        &mint_base.pubkey(),
        rpc_client
            .get_minimum_balance_for_rent_exemption(mint_account_size)
            .unwrap(),
        mint_account_size as u64, // allocate size of data to buffer account data
        &spl_token::id(),
    );
    let initialize_mint_base_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint_base.pubkey(),
        &admin.pubkey(),
        None,
        0,
    )?;
    let create_mint_quote_ix = system_instruction::create_account(
        &admin.pubkey(),
        &mint_quote.pubkey(),
        rpc_client
            .get_minimum_balance_for_rent_exemption(mint_account_size)
            .unwrap(),
        mint_account_size as u64, // allocate size of data to buffer account data
        &spl_token::id(),
    );
    let initialize_mint_quote_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint_quote.pubkey(),
        &admin.pubkey(),
        None,
        0,
    )?;

    // Create InitializeExchangeBooth Instruction
    // Find PDA addresses
    let admin_pk = admin.pubkey();
    let mint_quote_pk = mint_quote.pubkey();
    let mint_base_pk = mint_base.pubkey();
    let state_seeds = &[
        b"state_info",
        admin_pk.as_ref(),
        mint_base_pk.as_ref(),
        mint_quote_pk.as_ref(),
        // oracle_pk.as_ref(),
    ];
    let (state_pk, state_bump) = Pubkey::find_program_address(state_seeds, &eb_program_id);
    let vault_base_seeds = &[b"vault_base", state_pk.as_ref(), mint_base_pk.as_ref()];
    let (vault_base_pk, vault_base_bump) =
        Pubkey::find_program_address(vault_base_seeds, &eb_program_id);
    let vault_quote_seeds = &[b"vault_quote", state_pk.as_ref(), mint_quote_pk.as_ref()];
    let (vault_quote_pk, vault_quote_bump) =
        Pubkey::find_program_address(vault_quote_seeds, &eb_program_id);
    let initialize_eb_ix = Instruction {
        accounts: vec![
            //   0. `[signer]` EB admin account.
            AccountMeta::new_readonly(admin.pubkey(), true),
            //   1. `[]` Mint account of the base currency.
            AccountMeta::new_readonly(mint_base.pubkey(), false),
            //   2. `[]` Mint account of the quote currency.
            AccountMeta::new_readonly(mint_quote.pubkey(), false),
            //   3. `[]` Oracle program account.
            AccountMeta::new_readonly(oracle_pk, false),
            //   4. `[]` Token Program.
            AccountMeta::new_readonly(spl_token::id(), false),
            //   5. `[]` System Program.
            AccountMeta::new_readonly(system_program::id(), false),
            //   6. `[]` Rent Sysvar.
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            //   7. `[]` EB state account (PDA).
            AccountMeta::new(state_pk, false),
            //   8. `[]` Vault token account of the base currency (PDA).
            AccountMeta::new(vault_base_pk, false),
            //   9. `[]` Vault token account of the quote currency (PDA).
            AccountMeta::new(vault_quote_pk, false),
        ],
        data: ExchangeBoothInstruction::InititializeExchangeBooth {
            vault_base_bump,
            vault_quote_bump,
            state_bump,
        }
        .try_to_vec()?,
        program_id: eb_program_id,
    };

    // Create transaction
    let instructions = [
        create_mint_base_ix,
        create_mint_quote_ix,
        initialize_mint_base_ix,
        initialize_mint_quote_ix,
        // initialize_eb_ix,
    ];
    let signers = [&admin, &mint_base, &mint_quote];
    let mut transaction = Transaction::new_signed_with_payer(
        instructions.as_ref(),
        Some(&admin_pk),
        &signers,
        blockhash,
    );

    // Sign and send transaction
    transaction.sign(&signers, blockhash);
    rpc_client.send_and_confirm_transaction(&transaction)?;

    // Confirm ExchangeBooth state has been saved
    let eb_state_data = rpc_client.get_account_data(&state_pk)?;
    let eb_state_struct = ExchangeBooth::try_from_slice(eb_state_data.as_ref())?;
    assert!(eb_state_struct.)

    Ok(())
}
