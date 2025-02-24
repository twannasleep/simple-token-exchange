// Program State Definitions
// This module defines the state structures used by the token exchange program

use borsh::{BorshDeserialize, BorshSerialize}; // For account data serialization
use solana_program::pubkey::Pubkey;            // For handling Solana public keys

/// Represents the state of a liquidity pool in the token exchange
/// 
/// This structure stores all necessary information about a single token-SOL pool,
/// including reserves, fee configuration, and authority information.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PoolState {
    pub authority: Pubkey,      // The pool administrator's public key
    pub sol_reserve: u64,       // Current SOL balance in the pool
    pub token_reserve: u64,     // Current SPL token balance in the pool
    pub lp_mint: Pubkey,        // Mint address for LP tokens
    pub fee_rate: u64,          // Trading fee in basis points (1 bp = 0.01%, e.g., 30 = 0.3%)
    pub token_mint: Pubkey,     // Mint address of the SPL token in the pool
    pub initialized: bool,      // Pool initialization status flag
}

/// Represents a liquidity provider's position in the pool
/// 
/// Tracks an individual user's liquidity provision and their share of the pool
/// through LP tokens.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserPosition {
    pub owner: Pubkey,          // The liquidity provider's public key
    pub lp_tokens: u64,         // Amount of LP tokens owned by this user
}

impl PoolState {
    /// Total size of the PoolState structure when serialized
    /// 
    /// Breakdown:
    /// - authority (Pubkey): 32 bytes
    /// - sol_reserve (u64): 8 bytes
    /// - token_reserve (u64): 8 bytes
    /// - lp_mint (Pubkey): 32 bytes
    /// - fee_rate (u64): 8 bytes
    /// - token_mint (Pubkey): 32 bytes
    /// - initialized (bool): 1 byte
    pub const LEN: usize = 32 + 8 + 8 + 32 + 8 + 32 + 1;
}

impl UserPosition {
    /// Total size of the UserPosition structure when serialized
    /// 
    /// Breakdown:
    /// - owner (Pubkey): 32 bytes
    /// - lp_tokens (u64): 8 bytes
    pub const LEN: usize = 32 + 8;
} 