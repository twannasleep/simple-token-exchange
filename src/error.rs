use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, FromPrimitive)]
pub enum TokenExchangeError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    
    #[error("Pool already initialized")]
    PoolAlreadyInitialized,
    
    #[error("Pool not initialized")]
    PoolNotInitialized,
    
    #[error("Invalid pool authority")]
    InvalidPoolAuthority,
    
    #[error("Invalid token mint")]
    InvalidTokenMint,
    
    #[error("Insufficient liquidity")]
    InsufficientLiquidity,
    
    #[error("Slippage tolerance exceeded")]
    SlippageExceeded,
    
    #[error("Invalid fee rate")]
    InvalidFeeRate,
    
    #[error("Math operation overflow")]
    MathOverflow,
    
    #[error("Invalid user position")]
    InvalidUserPosition,
}

impl From<TokenExchangeError> for ProgramError {
    fn from(e: TokenExchangeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for TokenExchangeError {
    fn type_of() -> &'static str {
        "TokenExchangeError"
    }
} 