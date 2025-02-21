use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::state::Account as TokenAccount;

use crate::{
    error::TokenExchangeError,
    instruction::TokenExchangeInstruction,
    state::{PoolState, UserPosition},
};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
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

    fn process_initialize_pool(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        sol_amount: u64,
        token_amount: u64,
        fee_rate: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;
        let pool_account = next_account_info(account_info_iter)?;
        let token_mint = next_account_info(account_info_iter)?;
        let lp_mint = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut pool_state = PoolState {
            authority: *initializer.key,
            sol_reserve: sol_amount,
            token_reserve: token_amount,
            lp_mint: *lp_mint.key,
            fee_rate,
            token_mint: *token_mint.key,
            initialized: true,
        };

        pool_state.serialize(&mut *pool_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_swap(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
        is_sol_input: bool,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user = next_account_info(account_info_iter)?;
        let pool_account = next_account_info(account_info_iter)?;
        let user_sol_account = next_account_info(account_info_iter)?;
        let user_token_account = next_account_info(account_info_iter)?;
        let pool_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        if !user.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut pool_state = PoolState::try_from_slice(&pool_account.data.borrow())?;
        if !pool_state.initialized {
            return Err(TokenExchangeError::PoolNotInitialized.into());
        }

        // Calculate swap amounts using constant product formula (x * y = k)
        let (amount_out, new_sol_reserve, new_token_reserve) = if is_sol_input {
            let amount_out = Self::calculate_output_amount(
                amount_in,
                pool_state.sol_reserve,
                pool_state.token_reserve,
                pool_state.fee_rate,
            )?;
            
            if amount_out < minimum_amount_out {
                return Err(TokenExchangeError::SlippageExceeded.into());
            }

            (
                amount_out,
                pool_state.sol_reserve.checked_add(amount_in)
                    .ok_or(TokenExchangeError::MathOverflow)?,
                pool_state.token_reserve.checked_sub(amount_out)
                    .ok_or(TokenExchangeError::MathOverflow)?,
            )
        } else {
            let amount_out = Self::calculate_output_amount(
                amount_in,
                pool_state.token_reserve,
                pool_state.sol_reserve,
                pool_state.fee_rate,
            )?;

            if amount_out < minimum_amount_out {
                return Err(TokenExchangeError::SlippageExceeded.into());
            }

            (
                amount_out,
                pool_state.sol_reserve.checked_sub(amount_out)
                    .ok_or(TokenExchangeError::MathOverflow)?,
                pool_state.token_reserve.checked_add(amount_in)
                    .ok_or(TokenExchangeError::MathOverflow)?,
            )
        };

        // Update pool state
        pool_state.sol_reserve = new_sol_reserve;
        pool_state.token_reserve = new_token_reserve;
        pool_state.serialize(&mut *pool_account.data.borrow_mut())?;

        // Transfer tokens
        if is_sol_input {
            // Transfer SOL from user to pool
            invoke(
                &system_instruction::transfer(user.key, pool_account.key, amount_in),
                &[user.clone(), pool_account.clone()],
            )?;

            // Transfer tokens from pool to user
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
            // Transfer tokens from user to pool
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

            // Transfer SOL from pool to user
            **pool_account.try_borrow_mut_lamports()? -= amount_out;
            **user_sol_account.try_borrow_mut_lamports()? += amount_out;
        }

        Ok(())
    }

    fn process_add_liquidity(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        sol_amount: u64,
        token_amount: u64,
        minimum_lp_tokens: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let provider = next_account_info(account_info_iter)?;
        let pool_account = next_account_info(account_info_iter)?;
        let provider_sol_account = next_account_info(account_info_iter)?;
        let provider_token_account = next_account_info(account_info_iter)?;
        let pool_token_account = next_account_info(account_info_iter)?;
        let provider_lp_account = next_account_info(account_info_iter)?;
        let lp_mint = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        if !provider.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut pool_state = PoolState::try_from_slice(&pool_account.data.borrow())?;
        if !pool_state.initialized {
            return Err(TokenExchangeError::PoolNotInitialized.into());
        }

        // Calculate LP tokens to mint
        let lp_tokens_to_mint = if pool_state.sol_reserve == 0 && pool_state.token_reserve == 0 {
            // Initial liquidity provision
            (sol_amount as f64 * token_amount as f64).sqrt() as u64
        } else {
            // Subsequent liquidity provision
            let sol_ratio = (sol_amount as f64 / pool_state.sol_reserve as f64) * 1_000_000.0;
            let token_ratio = (token_amount as f64 / pool_state.token_reserve as f64) * 1_000_000.0;
            let min_ratio = sol_ratio.min(token_ratio);
            
            ((min_ratio / 1_000_000.0) * pool_state.sol_reserve as f64) as u64
        };

        if lp_tokens_to_mint < minimum_lp_tokens {
            return Err(TokenExchangeError::SlippageExceeded.into());
        }

        // Transfer SOL from provider to pool
        invoke(
            &system_instruction::transfer(provider.key, pool_account.key, sol_amount),
            &[provider.clone(), pool_account.clone()],
        )?;

        // Transfer tokens from provider to pool
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

        // Mint LP tokens to provider
        invoke(
            &spl_token::instruction::mint_to(
                token_program.key,
                lp_mint.key,
                provider_lp_account.key,
                pool_account.key,
                &[],
                lp_tokens_to_mint,
            )?,
            &[
                lp_mint.clone(),
                provider_lp_account.clone(),
                pool_account.clone(),
            ],
        )?;

        // Update pool state
        pool_state.sol_reserve = pool_state.sol_reserve.checked_add(sol_amount)
            .ok_or(TokenExchangeError::MathOverflow)?;
        pool_state.token_reserve = pool_state.token_reserve.checked_add(token_amount)
            .ok_or(TokenExchangeError::MathOverflow)?;
        pool_state.serialize(&mut *pool_account.data.borrow_mut())?;

        Ok(())
    }

    fn process_remove_liquidity(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        lp_tokens: u64,
        minimum_sol: u64,
        minimum_token: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let provider = next_account_info(account_info_iter)?;
        let pool_account = next_account_info(account_info_iter)?;
        let provider_sol_account = next_account_info(account_info_iter)?;
        let provider_token_account = next_account_info(account_info_iter)?;
        let pool_token_account = next_account_info(account_info_iter)?;
        let provider_lp_account = next_account_info(account_info_iter)?;
        let lp_mint = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        if !provider.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut pool_state = PoolState::try_from_slice(&pool_account.data.borrow())?;
        if !pool_state.initialized {
            return Err(TokenExchangeError::PoolNotInitialized.into());
        }

        // Calculate amounts to return
        let total_lp_supply = spl_token::state::Mint::unpack(&lp_mint.data.borrow())?.supply;
        
        let sol_amount = (pool_state.sol_reserve as u128)
            .checked_mul(lp_tokens as u128)
            .ok_or(TokenExchangeError::MathOverflow)?
            .checked_div(total_lp_supply as u128)
            .ok_or(TokenExchangeError::MathOverflow)? as u64;

        let token_amount = (pool_state.token_reserve as u128)
            .checked_mul(lp_tokens as u128)
            .ok_or(TokenExchangeError::MathOverflow)?
            .checked_div(total_lp_supply as u128)
            .ok_or(TokenExchangeError::MathOverflow)? as u64;

        if sol_amount < minimum_sol || token_amount < minimum_token {
            return Err(TokenExchangeError::SlippageExceeded.into());
        }

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

        // Transfer SOL from pool to provider
        **pool_account.try_borrow_mut_lamports()? -= sol_amount;
        **provider_sol_account.try_borrow_mut_lamports()? += sol_amount;

        // Transfer tokens from pool to provider
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

        // Update pool state
        pool_state.sol_reserve = pool_state.sol_reserve.checked_sub(sol_amount)
            .ok_or(TokenExchangeError::MathOverflow)?;
        pool_state.token_reserve = pool_state.token_reserve.checked_sub(token_amount)
            .ok_or(TokenExchangeError::MathOverflow)?;
        pool_state.serialize(&mut *pool_account.data.borrow_mut())?;

        Ok(())
    }

    fn calculate_output_amount(
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
        fee_rate: u64,
    ) -> Result<u64, ProgramError> {
        // Calculate fee
        let amount_in_with_fee = amount_in
            .checked_mul(10000 - fee_rate)
            .ok_or(TokenExchangeError::MathOverflow)?
            .checked_div(10000)
            .ok_or(TokenExchangeError::MathOverflow)?;

        // Calculate output amount using constant product formula
        let numerator = amount_in_with_fee
            .checked_mul(reserve_out)
            .ok_or(TokenExchangeError::MathOverflow)?;
        
        let denominator = reserve_in
            .checked_add(amount_in_with_fee)
            .ok_or(TokenExchangeError::MathOverflow)?;

        numerator
            .checked_div(denominator)
            .ok_or(TokenExchangeError::MathOverflow.into())
    }
} 