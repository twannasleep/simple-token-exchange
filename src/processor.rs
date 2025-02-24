// Core Processing Logic
// This module implements the business logic for all token exchange operations

use borsh::{BorshDeserialize, BorshSerialize};                 // For state serialization
use solana_program::{
    account_info::{next_account_info, AccountInfo},            // Account handling
    entrypoint::ProgramResult,                                 // Program result type
    msg,                                                       // Logging
    program::{invoke, invoke_signed},                          // CPI utilities
    program_error::ProgramError,                               // Error handling
    program_pack::Pack,                                        // Account packing
    pubkey::Pubkey,                                           // Public key type
    system_instruction,                                        // System program instructions
    sysvar::{rent::Rent, Sysvar},                            // System variables
};
use spl_token::state::Account as TokenAccount;                // SPL token account type

use crate::{
    error::TokenExchangeError,                                // Custom errors
    instruction::TokenExchangeInstruction,                    // Instruction definitions
    state::{PoolState, UserPosition},                        // Program state
};

/// Main processor struct for handling program logic
pub struct Processor;

impl Processor {
    /// Main processing function that routes instructions to their handlers
    /// 
    /// # Arguments
    /// * `program_id` - The program's public key
    /// * `accounts` - Accounts required for the instruction
    /// * `instruction_data` - Serialized instruction data
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        // Deserialize and route the instruction
        let instruction = TokenExchangeInstruction::unpack(instruction_data)?;

        match instruction {
            TokenExchangeInstruction::InitializePool {
                sol_amount,
                token_amount,
                fee_rate,
            } => {
                msg!("Instruction: Initialize Pool");
                Self::process_initialize_pool(accounts, program_id, sol_amount, token_amount, fee_rate)
            }
            TokenExchangeInstruction::Swap {
                amount_in,
                minimum_amount_out,
                is_sol_input,
            } => {
                msg!("Instruction: Swap");
                Self::process_swap(accounts, program_id, amount_in, minimum_amount_out, is_sol_input)
            }
            TokenExchangeInstruction::AddLiquidity {
                sol_amount,
                token_amount,
                minimum_lp_tokens,
            } => {
                msg!("Instruction: Add Liquidity");
                Self::process_add_liquidity(
                    accounts,
                    program_id,
                    sol_amount,
                    token_amount,
                    minimum_lp_tokens,
                )
            }
            TokenExchangeInstruction::RemoveLiquidity {
                lp_tokens,
                minimum_sol,
                minimum_token,
            } => {
                msg!("Instruction: Remove Liquidity");
                Self::process_remove_liquidity(
                    accounts,
                    program_id,
                    lp_tokens,
                    minimum_sol,
                    minimum_token,
                )
            }
        }
    }

    /// Initializes a new liquidity pool with initial SOL and token deposits
    /// 
    /// # Arguments
    /// * `accounts` - Required accounts:
    ///   - Initializer (signer)
    ///   - Pool state account
    ///   - Token mint
    ///   - LP token mint
    ///   - System program
    /// * `program_id` - The program's public key
    /// * `sol_amount` - Initial SOL deposit
    /// * `token_amount` - Initial token deposit
    /// * `fee_rate` - Trading fee in basis points
    fn process_initialize_pool(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        sol_amount: u64,
        token_amount: u64,
        fee_rate: u64,
    ) -> ProgramResult {
        // Get account references
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;
        let pool_account = next_account_info(account_info_iter)?;
        let token_mint = next_account_info(account_info_iter)?;
        let lp_mint = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        // Verify initializer is a signer
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Create and initialize pool state
        let mut pool_state = PoolState {
            authority: *initializer.key,
            sol_reserve: sol_amount,
            token_reserve: token_amount,
            lp_mint: *lp_mint.key,
            fee_rate,
            token_mint: *token_mint.key,
            initialized: true,
        };

        // Save pool state to account
        pool_state.serialize(&mut *pool_account.data.borrow_mut())?;

        Ok(())
    }

    /// Executes a token swap between SOL and SPL tokens
    /// 
    /// Implements constant product AMM formula (x * y = k)
    /// with slippage protection and fee calculation.
    /// 
    /// # Arguments
    /// * `accounts` - Required accounts for the swap
    /// * `program_id` - The program's public key
    /// * `amount_in` - Input token amount
    /// * `minimum_amount_out` - Minimum acceptable output amount
    /// * `is_sol_input` - Whether SOL is the input token
    fn process_swap(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
        is_sol_input: bool,
    ) -> ProgramResult {
        // Get account references
        let account_info_iter = &mut accounts.iter();
        let user = next_account_info(account_info_iter)?;
        let pool_account = next_account_info(account_info_iter)?;
        let user_sol_account = next_account_info(account_info_iter)?;
        let user_token_account = next_account_info(account_info_iter)?;
        let pool_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        // Verify user is a signer
        if !user.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Load and verify pool state
        let mut pool_state = PoolState::try_from_slice(&pool_account.data.borrow())?;
        if !pool_state.initialized {
            return Err(TokenExchangeError::PoolNotInitialized.into());
        }

        // Calculate swap amounts using constant product formula (x * y = k)
        let (amount_out, new_sol_reserve, new_token_reserve) = if is_sol_input {
            // SOL → Token swap
            let amount_out = Self::calculate_output_amount(
                amount_in,
                pool_state.sol_reserve,
                pool_state.token_reserve,
                pool_state.fee_rate,
            )?;
            
            // Check slippage tolerance
            if amount_out < minimum_amount_out {
                return Err(TokenExchangeError::SlippageExceeded.into());
            }

            // Calculate new reserves
            (
                amount_out,
                pool_state.sol_reserve.checked_add(amount_in)
                    .ok_or(TokenExchangeError::MathOverflow)?,
                pool_state.token_reserve.checked_sub(amount_out)
                    .ok_or(TokenExchangeError::MathOverflow)?,
            )
        } else {
            // Token → SOL swap
            let amount_out = Self::calculate_output_amount(
                amount_in,
                pool_state.token_reserve,
                pool_state.sol_reserve,
                pool_state.fee_rate,
            )?;

            // Check slippage tolerance
            if amount_out < minimum_amount_out {
                return Err(TokenExchangeError::SlippageExceeded.into());
            }

            // Calculate new reserves
            (
                amount_out,
                pool_state.sol_reserve.checked_sub(amount_out)
                    .ok_or(TokenExchangeError::MathOverflow)?,
                pool_state.token_reserve.checked_add(amount_in)
                    .ok_or(TokenExchangeError::MathOverflow)?,
            )
        };

        // Update pool state with new reserves
        pool_state.sol_reserve = new_sol_reserve;
        pool_state.token_reserve = new_token_reserve;
        pool_state.serialize(&mut *pool_account.data.borrow_mut())?;

        // Execute token transfers
        if is_sol_input {
            // SOL → Token: Transfer SOL to pool
            invoke(
                &system_instruction::transfer(user.key, pool_account.key, amount_in),
                &[user.clone(), pool_account.clone()],
            )?;

            // Transfer tokens to user
            invoke(
                &spl_token::instruction::transfer(
                    token_program.key,
                    pool_token_account.key,
                    user_token_account.key,
                    pool_account.key,
                    &[],
                    amount_out,
                )?,
                &[
                    pool_token_account.clone(),
                    user_token_account.clone(),
                    pool_account.clone(),
                ],
            )?;
        } else {
            // Token → SOL: Transfer tokens to pool
            invoke(
                &spl_token::instruction::transfer(
                    token_program.key,
                    user_token_account.key,
                    pool_token_account.key,
                    user.key,
                    &[],
                    amount_in,
                )?,
                &[
                    user_token_account.clone(),
                    pool_token_account.clone(),
                    user.clone(),
                ],
            )?;

            // Transfer SOL to user
            **pool_account.try_borrow_mut_lamports()? -= amount_out;
            **user_sol_account.try_borrow_mut_lamports()? += amount_out;
        }

        Ok(())
    }

    /// Adds liquidity to the pool
    /// 
    /// Allows users to deposit both SOL and tokens to the pool
    /// in exchange for LP tokens representing their share.
    /// 
    /// # Arguments
    /// * `accounts` - Required accounts for liquidity provision
    /// * `program_id` - The program's public key
    /// * `sol_amount` - Amount of SOL to deposit
    /// * `token_amount` - Amount of tokens to deposit
    /// * `minimum_lp_tokens` - Minimum acceptable LP tokens
    fn process_add_liquidity(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        sol_amount: u64,
        token_amount: u64,
        minimum_lp_tokens: u64,
    ) -> ProgramResult {
        // Get account references
        let account_info_iter = &mut accounts.iter();
        let provider = next_account_info(account_info_iter)?;
        let pool_account = next_account_info(account_info_iter)?;
        let provider_sol_account = next_account_info(account_info_iter)?;
        let provider_token_account = next_account_info(account_info_iter)?;
        let pool_token_account = next_account_info(account_info_iter)?;
        let provider_lp_account = next_account_info(account_info_iter)?;
        let lp_mint = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        // Verify provider is a signer
        if !provider.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Load pool state
        let mut pool_state = PoolState::try_from_slice(&pool_account.data.borrow())?;
        if !pool_state.initialized {
            return Err(TokenExchangeError::PoolNotInitialized.into());
        }

        // Calculate LP tokens to mint
        let lp_tokens = if pool_state.sol_reserve == 0 {
            // Initial liquidity: Use geometric mean
            ((sol_amount as f64) * (token_amount as f64)).sqrt() as u64
        } else {
            // Subsequent liquidity: Proportional to existing reserves
            let total_supply = spl_token::state::Mint::unpack(&lp_mint.data.borrow())?.supply;
            let sol_ratio = (sol_amount * 1_000_000) / pool_state.sol_reserve;
            let token_ratio = (token_amount * 1_000_000) / pool_state.token_reserve;
            let min_ratio = std::cmp::min(sol_ratio, token_ratio);
            (min_ratio * total_supply) / 1_000_000
        };

        // Check minimum LP tokens
        if lp_tokens < minimum_lp_tokens {
            return Err(TokenExchangeError::SlippageExceeded.into());
        }

        // Update pool state
        pool_state.sol_reserve = pool_state.sol_reserve.checked_add(sol_amount)
            .ok_or(TokenExchangeError::MathOverflow)?;
        pool_state.token_reserve = pool_state.token_reserve.checked_add(token_amount)
            .ok_or(TokenExchangeError::MathOverflow)?;
        pool_state.serialize(&mut *pool_account.data.borrow_mut())?;

        // Transfer assets
        invoke(
            &system_instruction::transfer(provider.key, pool_account.key, sol_amount),
            &[provider.clone(), pool_account.clone()],
        )?;

        invoke(
            &spl_token::instruction::transfer(
                token_program.key,
                provider_token_account.key,
                pool_token_account.key,
                provider.key,
                &[],
                token_amount,
            )?,
            &[
                provider_token_account.clone(),
                pool_token_account.clone(),
                provider.clone(),
            ],
        )?;

        // Mint LP tokens
        invoke(
            &spl_token::instruction::mint_to(
                token_program.key,
                lp_mint.key,
                provider_lp_account.key,
                pool_account.key,
                &[],
                lp_tokens,
            )?,
            &[
                lp_mint.clone(),
                provider_lp_account.clone(),
                pool_account.clone(),
            ],
        )?;

        Ok(())
    }

    /// Removes liquidity from the pool
    /// 
    /// Allows LP token holders to burn their tokens and withdraw
    /// their proportional share of SOL and tokens from the pool.
    /// 
    /// # Arguments
    /// * `accounts` - Required accounts for liquidity removal
    /// * `program_id` - The program's public key
    /// * `lp_tokens` - Amount of LP tokens to burn
    /// * `minimum_sol` - Minimum SOL to withdraw
    /// * `minimum_token` - Minimum tokens to withdraw
    fn process_remove_liquidity(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        lp_tokens: u64,
        minimum_sol: u64,
        minimum_token: u64,
    ) -> ProgramResult {
        // Get account references
        let account_info_iter = &mut accounts.iter();
        let provider = next_account_info(account_info_iter)?;
        let pool_account = next_account_info(account_info_iter)?;
        let provider_sol_account = next_account_info(account_info_iter)?;
        let provider_token_account = next_account_info(account_info_iter)?;
        let pool_token_account = next_account_info(account_info_iter)?;
        let provider_lp_account = next_account_info(account_info_iter)?;
        let lp_mint = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        // Verify provider is a signer
        if !provider.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Load pool state
        let mut pool_state = PoolState::try_from_slice(&pool_account.data.borrow())?;
        if !pool_state.initialized {
            return Err(TokenExchangeError::PoolNotInitialized.into());
        }

        // Calculate withdrawal amounts
        let total_supply = spl_token::state::Mint::unpack(&lp_mint.data.borrow())?.supply;
        let sol_amount = (pool_state.sol_reserve * lp_tokens) / total_supply;
        let token_amount = (pool_state.token_reserve * lp_tokens) / total_supply;

        // Check minimum amounts
        if sol_amount < minimum_sol || token_amount < minimum_token {
            return Err(TokenExchangeError::SlippageExceeded.into());
        }

        // Update pool state
        pool_state.sol_reserve = pool_state.sol_reserve.checked_sub(sol_amount)
            .ok_or(TokenExchangeError::MathOverflow)?;
        pool_state.token_reserve = pool_state.token_reserve.checked_sub(token_amount)
            .ok_or(TokenExchangeError::MathOverflow)?;
        pool_state.serialize(&mut *pool_account.data.borrow_mut())?;

        // Burn LP tokens
        invoke(
            &spl_token::instruction::burn(
                token_program.key,
                provider_lp_account.key,
                lp_mint.key,
                provider.key,
                &[],
                lp_tokens,
            )?,
            &[
                provider_lp_account.clone(),
                lp_mint.clone(),
                provider.clone(),
            ],
        )?;

        // Transfer assets back to provider
        **pool_account.try_borrow_mut_lamports()? -= sol_amount;
        **provider_sol_account.try_borrow_mut_lamports()? += sol_amount;

        invoke(
            &spl_token::instruction::transfer(
                token_program.key,
                pool_token_account.key,
                provider_token_account.key,
                pool_account.key,
                &[],
                token_amount,
            )?,
            &[
                pool_token_account.clone(),
                provider_token_account.clone(),
                pool_account.clone(),
            ],
        )?;

        Ok(())
    }

    /// Calculates output amount for a swap using constant product formula
    /// 
    /// Implements x * y = k formula with fee adjustment
    /// 
    /// # Arguments
    /// * `amount_in` - Input token amount
    /// * `reserve_in` - Input token reserve
    /// * `reserve_out` - Output token reserve
    /// * `fee_rate` - Fee rate in basis points
    /// 
    /// # Returns
    /// * Amount of output tokens to receive
    fn calculate_output_amount(
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
        fee_rate: u64,
    ) -> Result<u64, ProgramError> {
        // Calculate amount after fees
        let amount_in_with_fee = amount_in
            .checked_mul(10000 - fee_rate)
            .ok_or(TokenExchangeError::MathOverflow)?
            .checked_div(10000)
            .ok_or(TokenExchangeError::MathOverflow)?;

        // Calculate output amount: (y * dx) / (x + dx)
        let numerator = reserve_out
            .checked_mul(amount_in_with_fee)
            .ok_or(TokenExchangeError::MathOverflow)?;
        let denominator = reserve_in
            .checked_add(amount_in_with_fee)
            .ok_or(TokenExchangeError::MathOverflow)?;

        numerator
            .checked_div(denominator)
            .ok_or(TokenExchangeError::MathOverflow.into())
    }
} 