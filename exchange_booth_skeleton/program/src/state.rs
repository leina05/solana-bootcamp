use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use std::mem::size_of;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ExchangeBooth {
    admin: Pubkey,
    mint_base: Pubkey,
    decimal_base: u8,
    mint_quote: Pubkey,
    decimal_quote: u8,
    oracle: Pubkey,
    fee: u64, // Fee in bps
}
