// Simple Token Exchange Program
// A Solana program that enables swapping between SOL and SPL tokens using an AMM model

use solana_program::{
    account_info::AccountInfo, // For handling account information
    entrypoint,               // Macro for declaring program entry point
    entrypoint::ProgramResult,// Type for program result handling
    pubkey::Pubkey,          // For handling public keys
};

// Module declarations for program components
pub mod error;      // Custom error definitions
pub mod instruction;// Instruction handling and definitions
pub mod processor;  // Core business logic implementation
pub mod state;      // Program state and account structures

use crate::processor::Processor;

// Declare the program's entry point using Solana's entrypoint macro
entrypoint!(process_instruction);

/// Program entrypoint - The gateway for all interactions with this Solana program
/// 
/// # Arguments
/// * `program_id` - The public key of this program's instance
/// * `accounts` - Array of accounts involved in this instruction
/// * `instruction_data` - Serialized instruction data
/// 
/// # Returns
/// * `ProgramResult` - Result indicating success or containing an error
pub fn process_instruction(
    program_id: &Pubkey,      // The program's own public key
    accounts: &[AccountInfo],  // Accounts required for the instruction
    instruction_data: &[u8],   // Serialized instruction parameters
) -> ProgramResult {
    // Delegate processing to the Processor module
    Processor::process(program_id, accounts, instruction_data)
} 