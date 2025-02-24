// Custom Error Definitions
// This module defines all possible errors that can occur in the token exchange program

use num_derive::FromPrimitive;                                      // For converting numbers to enum variants
use solana_program::{decode_error::DecodeError,                     // For error decoding functionality
                    program_error::ProgramError};                    // Base Solana program error type
use thiserror::Error;                                              // For error handling macros

/// Custom error types for the Token Exchange program
/// 
/// These errors provide specific information about what went wrong during
/// program execution, making it easier to debug and handle errors appropriately.
#[derive(Error, Debug, Copy, Clone, FromPrimitive)]
pub enum TokenExchangeError {
    /// Indicates that the instruction data is invalid or malformed
    #[error("Invalid instruction")]
    InvalidInstruction,
    
    /// Attempted to initialize a pool that is already initialized
    #[error("Pool already initialized")]
    PoolAlreadyInitialized,
    
    /// Attempted to interact with a pool that hasn't been initialized
    #[error("Pool not initialized")]
    PoolNotInitialized,
    
    /// The provided authority doesn't match the pool's authority
    #[error("Invalid pool authority")]
    InvalidPoolAuthority,
    
    /// The token mint doesn't match the pool's configured token mint
    #[error("Invalid token mint")]
    InvalidTokenMint,
    
    /// The pool doesn't have enough liquidity for the requested operation
    #[error("Insufficient liquidity")]
    InsufficientLiquidity,
    
    /// The actual output amount would be less than the minimum specified
    #[error("Slippage tolerance exceeded")]
    SlippageExceeded,
    
    /// The provided fee rate is outside the acceptable range
    #[error("Invalid fee rate")]
    InvalidFeeRate,
    
    /// A mathematical operation resulted in overflow or underflow
    #[error("Math operation overflow")]
    MathOverflow,
    
    /// The user position account is invalid or doesn't exist
    #[error("Invalid user position")]
    InvalidUserPosition,
}

/// Converts our custom error into a Solana program error
/// 
/// This implementation allows our errors to be used anywhere a ProgramError
/// is expected, making them compatible with Solana's error handling system.
impl From<TokenExchangeError> for ProgramError {
    fn from(e: TokenExchangeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

/// Implements error decoding for client-side error handling
/// 
/// This allows clients to properly decode and handle our custom errors
/// when they receive them from the program.
impl<T> DecodeError<T> for TokenExchangeError {
    fn type_of() -> &'static str {
        "TokenExchangeError"
    }
} 