# Token Exchange Program Workflow

## Program Overview

This Solana program implements a simple token exchange (DEX) that allows users to:

* Create liquidity pools for SOL/SPL token pairs
* Add liquidity to pools
* Swap between SOL and SPL tokens
* Remove liquidity from pools

## Visual Flow Diagrams

### 1. High-Level Program Flow

```mermaid
graph TD
    A[User] -->|1. Initialize Pool| B[Create Pool]
    A -->|2. Add Liquidity| C[Provide Liquidity]
    A -->|3. Swap Tokens| D[Execute Swap]
    A -->|4. Remove Liquidity| E[Withdraw Liquidity]
    
    B -->|Creates| F[Pool State Account]
    B -->|Mints| G[LP Tokens]
    
    C -->|Updates| F
    C -->|Receives| G
    
    D -->|Updates| F
    D -->|Transfers| H[Tokens/SOL]
    
    E -->|Updates| F
    E -->|Burns| G
    E -->|Returns| H
```

### 2. Pool Initialization Flow

```mermaid
sequenceDiagram
    participant U as User
    participant P as Program
    participant PS as Pool State
    participant LP as LP Token Mint
    participant SPL as SPL Token

    U->>P: Initialize Pool Request
    P->>PS: Create Pool Account
    P->>LP: Initialize LP Token Mint
    P->>PS: Store Initial State
    PS-->>U: Return Pool Info
```

### 3. Swap Operation Flow

```mermaid
graph LR
    A[User Input] -->|Amount In| B[Calculate Output]
    B -->|Apply Fee| C[Fee Calculation]
    C -->|Check Slippage| D[Slippage Check]
    D -->|Execute| E[Token Transfer]
    E -->|Update| F[Pool State]
```

### 4. Liquidity Management Flow

```mermaid
graph TD
    subgraph Add Liquidity
        A1[User Deposits] -->|SOL + Tokens| B1[Calculate Share]
        B1 -->|Mint| C1[LP Tokens]
        C1 -->|Update| D1[Pool Reserves]
    end
    
    subgraph Remove Liquidity
        A2[User Burns LP] -->|LP Tokens| B2[Calculate Share]
        B2 -->|Return| C2[SOL + Tokens]
        C2 -->|Update| D2[Pool Reserves]
    end
```

### 5. Account Structure and Data Flow

```mermaid
classDiagram
    class PoolState {
        +Pubkey authority
        +u64 sol_reserve
        +u64 token_reserve
        +Pubkey lp_mint
        +u64 fee_rate
        +bool initialized
    }
    
    class UserPosition {
        +Pubkey owner
        +u64 lp_tokens
    }

    class Operations {
        +Initialize()
        +Swap()
        +AddLiquidity()
        +RemoveLiquidity()
    }

    Operations --> PoolState : Updates
    Operations --> UserPosition : Manages
```

### 6. Program State Transitions

```mermaid
stateDiagram-v2
    [*] --> Uninitialized
    Uninitialized --> Initialized: Initialize Pool
    Initialized --> Active: Add Initial Liquidity
    Active --> Active: Swap/Add/Remove Liquidity
    Active --> Empty: Remove All Liquidity
    Empty --> Active: Add Liquidity
```

## Program Architecture

### 1. Account Structure

PoolState Account:

```
+------------------+
|    PoolState     |
+------------------+
| - SOL reserve    |
| - Token reserve  |
| - LP token mint  |
| - Fee rate      |
| - Authority     |
| - Status        |
+------------------+
```

UserPosition Account:

```
+------------------+
|  UserPosition    |
+------------------+
| - Owner         |
| - LP tokens     |
+------------------+
```

### 2. Instructions

#### Initialize Pool

```rust
InitializePool {
    sol_amount: u64,    // Initial SOL amount
    token_amount: u64,  // Initial token amount
    fee_rate: u64,     // Fee rate in basis points (e.g., 30 = 0.3%)
}
```

Required accounts:

1. Signer (pool creator)
2. Pool state account (PDA)
3. Token mint
4. LP token mint
5. System program

#### Swap

```rust
Swap {
    amount_in: u64,           // Amount to swap
    minimum_amount_out: u64,  // Minimum amount to receive
    is_sol_input: bool,      // Whether SOL is input
}
```

Required accounts:

1. Signer (user)
2. Pool state account
3. User's SOL account
4. User's token account
5. Pool's token account
6. Token program

## Development Workflow

### 1. Build and Deploy

```bash
# Build the program
cargo build-sbf

# Deploy to local validator
solana program deploy target/deploy/simple_token_exchange.so
```

### 2. Test Setup

```bash
# Create test token
spl-token create-token

# Create token account
spl-token create-account <TOKEN_ADDRESS>

# Mint test tokens
spl-token mint <TOKEN_ADDRESS> <AMOUNT>
```

## Price Calculation

The program uses the constant product formula (x * y = k):

* x = SOL reserve
* y = Token reserve
* k = Constant product

For swaps:

* Output amount = (y * dx) / (x + dx)
* Where dx is input amount
* Fee is deducted from input amount before calculation

## Security Considerations

1. Overflow Protection
   * All mathematical operations use checked arithmetic
   * Large numbers are handled using u128 for intermediate calculations

2. Access Control
   * Account ownership verification
   * Signer verification for all state-changing operations

3. Slippage Protection
   * Minimum output amount check for swaps
   * Minimum LP token amount check for liquidity provision

## Testing

1. Local Testing

   ```bash
   # Start local validator
   solana-test-validator

   # Run test script
   npx ts-node tests/test-exchange.ts
   ```

2. Monitoring

   ```bash
   # Check program logs
   solana logs <PROGRAM_ID>

   # Check account info
   solana account <ACCOUNT_ADDRESS>
   ```
