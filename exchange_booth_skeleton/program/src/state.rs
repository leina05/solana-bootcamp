use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

#[derive(BorshSerialize, BorshDeserialize, Debug, Default, Clone)]
pub struct ExchangeBooth {
    pub admin: Pubkey,
    pub mint_base: Pubkey,
    pub decimals_base: u8,
    pub mint_quote: Pubkey,
    pub decimals_quote: u8,
    pub oracle: Pubkey,
    pub fee: u64, // Fee in bps
}

impl ExchangeBooth {
    pub fn get_serialized_size() -> Result<usize, ProgramError> {
        Ok(Self::default().try_to_vec()?.len())
    }
}
