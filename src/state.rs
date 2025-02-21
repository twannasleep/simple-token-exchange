use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PoolState {
    pub authority: Pubkey,      // Pool authority
    pub sol_reserve: u64,       // SOL reserve
    pub token_reserve: u64,     // Token reserve
    pub lp_mint: Pubkey,        // LP token mint
    pub fee_rate: u64,          // Fee rate (basis points, e.g., 30 = 0.3%)
    pub token_mint: Pubkey,     // SPL token mint
    pub initialized: bool,      // Whether the pool is initialized
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserPosition {
    pub owner: Pubkey,          // Position owner
    pub lp_tokens: u64,         // LP token amount
}

impl PoolState {
    pub const LEN: usize = 32 + 8 + 8 + 32 + 8 + 32 + 1; // Size of the struct in bytes
}

impl UserPosition {
    pub const LEN: usize = 32 + 8; // Size of the struct in bytes
} 