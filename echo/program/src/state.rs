use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AuthorizedBuffer {
    pub bump_seed: u8,
    pub buffer_seed: u64,
    pub data: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct VendingMachineBuffer {
    pub bump_seed: u8,
    pub price: u64,
    pub data: Vec<u8>,
}
