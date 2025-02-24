// Instruction Definitions
// This module defines all instructions that can be executed by the token exchange program

use borsh::{BorshDeserialize, BorshSerialize};                     // For instruction data serialization
use solana_program::instruction::AccountMeta;                       // For account metadata
use solana_program::program_error::ProgramError;                    // For error handling
use solana_program::{pubkey::Pubkey, system_program, sysvar};      // For Solana primitives

/// Defines all instructions supported by the Token Exchange program
/// 
/// Each variant represents a different operation that can be performed,
/// along with its required parameters and expected accounts.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum TokenExchangeInstruction {
    /// Creates and initializes a new liquidity pool
    /// 
    /// This instruction sets up a new trading pair between SOL and an SPL token.
    /// It requires initial liquidity to be provided in both assets.
    /// 
    /// Accounts expected:
    /// 0. `[signer]` The account creating the pool (will be the pool authority)
    /// 1. `[writable]` The pool state account (PDA to store pool data)
    /// 2. `[]` The token mint (SPL token to be traded)
    /// 3. `[writable]` The LP token mint (must be created beforehand)
    /// 4. `[]` The system program (for rent and account creation)
    InitializePool {
        /// Initial amount of SOL to deposit
        sol_amount: u64,
        /// Initial amount of SPL tokens to deposit
        token_amount: u64,
        /// Trading fee percentage in basis points (1 bp = 0.01%)
        fee_rate: u64,
    },

    /// Executes a token swap between SOL and SPL tokens
    /// 
    /// Allows users to trade between SOL and the pool's SPL token.
    /// Implements constant product AMM formula (x * y = k).
    /// 
    /// Accounts expected:
    /// 0. `[signer]` The user performing the swap
    /// 1. `[writable]` The pool state account (stores reserves)
    /// 2. `[writable]` User's SOL account (system account)
    /// 3. `[writable]` User's token account (SPL token account)
    /// 4. `[writable]` Pool's token account (holds pool's SPL tokens)
    /// 5. `[]` Token program (for SPL token operations)
    Swap {
        /// Amount of input token (SOL or SPL) to swap
        amount_in: u64,
        /// Minimum amount of output token to receive (slippage protection)
        minimum_amount_out: u64,
        /// Direction of the swap (true = SOL→Token, false = Token→SOL)
        is_sol_input: bool,
    },

    /// Adds liquidity to the pool
    /// 
    /// Allows liquidity providers to deposit both SOL and SPL tokens
    /// in exchange for LP tokens representing their share of the pool.
    /// 
    /// Accounts expected:
    /// 0. `[signer]` The liquidity provider
    /// 1. `[writable]` The pool state account
    /// 2. `[writable]` Provider's SOL account (system account)
    /// 3. `[writable]` Provider's token account (SPL tokens to deposit)
    /// 4. `[writable]` Pool's token account
    /// 5. `[writable]` Provider's LP token account (to receive LP tokens)
    /// 6. `[writable]` LP token mint
    /// 7. `[]` Token program
    AddLiquidity {
        /// Amount of SOL to deposit
        sol_amount: u64,
        /// Amount of SPL tokens to deposit
        token_amount: u64,
        /// Minimum LP tokens to accept (slippage protection)
        minimum_lp_tokens: u64,
    },

    /// Removes liquidity from the pool
    /// 
    /// Allows liquidity providers to burn their LP tokens
    /// and withdraw their share of SOL and SPL tokens.
    /// 
    /// Accounts expected:
    /// 0. `[signer]` The liquidity provider
    /// 1. `[writable]` The pool state account
    /// 2. `[writable]` Provider's SOL account (to receive SOL)
    /// 3. `[writable]` Provider's token account (to receive tokens)
    /// 4. `[writable]` Pool's token account
    /// 5. `[writable]` Provider's LP token account (tokens to burn)
    /// 6. `[writable]` LP token mint
    /// 7. `[]` Token program
    RemoveLiquidity {
        /// Amount of LP tokens to burn
        lp_tokens: u64,
        /// Minimum SOL to accept (slippage protection)
        minimum_sol: u64,
        /// Minimum tokens to accept (slippage protection)
        minimum_token: u64,
    },
}

impl TokenExchangeInstruction {
    /// Deserializes a byte buffer into a TokenExchangeInstruction
    /// 
    /// The first byte (tag) determines which instruction variant to deserialize.
    /// Remaining bytes are parsed according to the instruction's parameters.
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(ProgramError::InvalidInstructionData)?;
        Ok(match tag {
            0 => Self::InitializePool {
                sol_amount: Self::unpack_u64(rest, 0)?,
                token_amount: Self::unpack_u64(rest, 8)?,
                fee_rate: Self::unpack_u64(rest, 16)?,
            },
            1 => Self::Swap {
                amount_in: Self::unpack_u64(rest, 0)?,
                minimum_amount_out: Self::unpack_u64(rest, 8)?,
                is_sol_input: rest[16] != 0,
            },
            2 => Self::AddLiquidity {
                sol_amount: Self::unpack_u64(rest, 0)?,
                token_amount: Self::unpack_u64(rest, 8)?,
                minimum_lp_tokens: Self::unpack_u64(rest, 16)?,
            },
            3 => Self::RemoveLiquidity {
                lp_tokens: Self::unpack_u64(rest, 0)?,
                minimum_sol: Self::unpack_u64(rest, 8)?,
                minimum_token: Self::unpack_u64(rest, 16)?,
            },
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }

    /// Helper function to deserialize a u64 from a byte slice
    /// 
    /// # Arguments
    /// * `input` - The byte slice containing the u64
    /// * `start` - Starting position in the slice
    /// 
    /// # Returns
    /// * `Result<u64, ProgramError>` - The deserialized number or an error
    fn unpack_u64(input: &[u8], start: usize) -> Result<u64, ProgramError> {
        let value = input
            .get(start..start + 8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(value)
    }
} 