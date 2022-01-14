use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    system_program,
};
use spl_token;

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
        *account_info.key == system_program::id(),
        ProgramError::InvalidArgument,
        &format!("Expected System Program, received: {}", account_info.key),
    )
}

pub fn assert_is_token_program(account_info: &AccountInfo) -> ProgramResult {
    assert_with_msg(
        *account_info.key == spl_token::id(),
        ProgramError::InvalidArgument,
        &format!("Expected Token Program, received: {}", account_info.key),
    )
}

pub fn assert_is_initialized(account_info: &AccountInfo) -> ProgramResult {
    assert_with_msg(
        **account_info.lamports.borrow() > 0,
        ProgramError::UninitializedAccount,
        &format!("Account {} is uninitialized.", account_info.key),
    )
}
