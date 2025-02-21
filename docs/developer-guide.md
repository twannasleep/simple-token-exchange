# Developer Guide: Simple Token Exchange Program

## Source Code Structure

```
src/
├── lib.rs           # Program entrypoint and module declarations
├── instruction.rs   # Instruction definitions and unpacking logic
├── processor.rs     # Main program logic and instruction processing
├── state.rs        # Program state definitions
└── error.rs        # Custom error types
```

## Module Details

### 1. lib.rs - Program Entrypoint

```rust
// Main program entrypoint
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult
```

- Serves as the program's entry point
- Delegates processing to the Processor module
- Handles initial instruction routing

### 2. instruction.rs - Instruction Definitions

```rust
pub enum TokenExchangeInstruction {
    InitializePool { ... }
    Swap { ... }
    AddLiquidity { ... }
    RemoveLiquidity { ... }
}
```

Key Components:

- **InitializePool**: Creates new liquidity pool
  - Parameters: `sol_amount`, `token_amount`, `fee_rate`
  - Required Accounts: Signer, Pool State, Token Mint, LP Mint
  
- **Swap**: Executes token swaps
  - Parameters: `amount_in`, `minimum_amount_out`, `is_sol_input`
  - Required Accounts: User accounts, Pool accounts, Token Program
  
- **AddLiquidity**: Provides pool liquidity
  - Parameters: `sol_amount`, `token_amount`, `minimum_lp_tokens`
  - Required Accounts: Provider accounts, Pool accounts, LP Token accounts
  
- **RemoveLiquidity**: Withdraws liquidity
  - Parameters: `lp_tokens`, `minimum_sol`, `minimum_token`
  - Required Accounts: Provider accounts, Pool accounts, LP Token accounts

### 3. processor.rs - Core Logic Implementation

#### Key Functions

1. **process_initialize_pool**

```rust
fn process_initialize_pool(
    accounts: &[AccountInfo],
    program_id: &Pubkey,
    sol_amount: u64,
    token_amount: u64,
    fee_rate: u64,
) -> ProgramResult
```

- Creates new pool state account
- Initializes LP token mint
- Sets initial liquidity parameters

2. **process_swap**

```rust
fn process_swap(
    accounts: &[AccountInfo],
    program_id: &Pubkey,
    amount_in: u64,
    minimum_amount_out: u64,
    is_sol_input: bool,
) -> ProgramResult
```

- Implements constant product formula
- Handles token transfers
- Updates pool reserves

3. **calculate_output_amount**

```rust
fn calculate_output_amount(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_rate: u64,
) -> Result<u64, ProgramError>
```

- Implements core swap calculation
- Applies fees
- Prevents overflow

### 4. state.rs - Program State Definitions

```rust
pub struct PoolState {
    pub authority: Pubkey,
    pub sol_reserve: u64,
    pub token_reserve: u64,
    pub lp_mint: Pubkey,
    pub fee_rate: u64,
    pub token_mint: Pubkey,
    pub initialized: bool,
}
```

State Management:

- Uses Borsh for serialization
- Implements size constants for account allocation
- Maintains pool reserves and configuration

### 5. error.rs - Error Handling

```rust
pub enum TokenExchangeError {
    InvalidInstruction,
    PoolAlreadyInitialized,
    PoolNotInitialized,
    // ... other errors
}
```

Error Types:

- Program-specific errors
- Integration with Solana error handling
- Custom error messages

## Implementation Details

### Constant Product Formula

```rust
// y = (x * k) / (x + dx)
let amount_out = (reserve_out * amount_in_with_fee) / (reserve_in + amount_in_with_fee);
```

### Security Measures

1. **Overflow Protection**

```rust
// Example of checked arithmetic
pool_state.sol_reserve
    .checked_add(amount_in)
    .ok_or(TokenExchangeError::MathOverflow)?
```

2. **Account Validation**

```rust
if !initializer.is_signer {
    return Err(ProgramError::MissingRequiredSignature);
}
```

3. **Slippage Protection**

```rust
if amount_out < minimum_amount_out {
    return Err(TokenExchangeError::SlippageExceeded.into());
}
```

## Testing Guide

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_initialize_pool() {
        // Test pool initialization
    }

    #[test]
    fn test_swap() {
        // Test swap functionality
    }
}
```

### Integration Tests

```typescript
// TypeScript test example
async function testSwap() {
    const tx = new Transaction().add({
        keys: [...],
        programId: PROGRAM_ID,
        data: Buffer.from([...])
    });
}
```

## Common Development Workflows

1. **Adding New Features**
   - Add new instruction variant
   - Implement processor function
   - Update state if needed
   - Add error handling
   - Write tests

2. **Debugging**
   - Use `msg!` macro for logging
   - Check program logs
   - Verify account states
   - Test with minimal amounts

3. **Security Considerations**
   - Always use checked math
   - Validate all accounts
   - Implement slippage protection
   - Check signer privileges

## Best Practices

1. **Code Organization**
   - Keep processor functions focused
   - Use clear error messages
   - Document complex calculations
   - Maintain consistent validation

2. **Error Handling**
   - Use custom errors for clarity
   - Provide detailed error context
   - Handle edge cases explicitly
   - Validate early and often

3. **Testing**
   - Write comprehensive tests
   - Test edge cases
   - Verify error conditions
   - Test with realistic values

4. **Performance**
   - Minimize account lookups
   - Use efficient data structures
   - Optimize computation
   - Reduce transaction size
