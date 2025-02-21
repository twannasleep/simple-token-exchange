use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::{pubkey::Pubkey, system_program, sysvar};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum TokenExchangeInstruction {
    /// Initialize a new pool
    /// 
    /// Accounts expected:
    /// 0. `[signer]` The account creating the pool
    /// 1. `[writable]` The pool state account
    /// 2. `[]` The token mint
    /// 3. `[writable]` The LP token mint (must be created beforehand)
    /// 4. `[]` The system program
    InitializePool {
        /// Initial SOL amount
        sol_amount: u64,
        /// Initial token amount
        token_amount: u64,
        /// Fee rate in basis points (e.g., 30 = 0.3%)
        fee_rate: u64,
    },

    /// Swap tokens
    /// 
    /// Accounts expected:
    /// 0. `[signer]` The user performing the swap
    /// 1. `[writable]` The pool state account
    /// 2. `[writable]` User's SOL account (system account)
    /// 3. `[writable]` User's token account
    /// 4. `[writable]` Pool's token account
    /// 5. `[]` Token program
    Swap {
        /// Amount to swap
        amount_in: u64,
        /// Minimum amount to receive
        minimum_amount_out: u64,
        /// Whether the input is SOL
        is_sol_input: bool,
    },

    /// Add liquidity to the pool
    /// 
    /// Accounts expected:
    /// 0. `[signer]` The liquidity provider
    /// 1. `[writable]` The pool state account
    /// 2. `[writable]` Provider's SOL account (system account)
    /// 3. `[writable]` Provider's token account
    /// 4. `[writable]` Pool's token account
    /// 5. `[writable]` Provider's LP token account
    /// 6. `[writable]` LP token mint
    /// 7. `[]` Token program
    AddLiquidity {
        /// SOL amount to add
        sol_amount: u64,
        /// Token amount to add
        token_amount: u64,
        /// Minimum LP tokens to receive
        minimum_lp_tokens: u64,
    },

    /// Remove liquidity from the pool
    /// 
    /// Accounts expected:
    /// 0. `[signer]` The liquidity provider
    /// 1. `[writable]` The pool state account
    /// 2. `[writable]` Provider's SOL account (system account)
    /// 3. `[writable]` Provider's token account
    /// 4. `[writable]` Pool's token account
    /// 5. `[writable]` Provider's LP token account
    /// 6. `[writable]` LP token mint
    /// 7. `[]` Token program
    RemoveLiquidity {
        /// LP tokens to burn
        lp_tokens: u64,
        /// Minimum SOL to receive
        minimum_sol: u64,
        /// Minimum tokens to receive
        minimum_token: u64,
    },
}

impl TokenExchangeInstruction {
    /// Unpacks a byte buffer into a TokenExchangeInstruction
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

    fn unpack_u64(input: &[u8], start: usize) -> Result<u64, ProgramError> {
        let value = input
            .get(start..start + 8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(value)
    }
} 