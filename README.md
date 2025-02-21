# Simple Token Exchange Program

A Solana program that implements a basic token exchange (DEX) allowing users to swap between SOL and SPL tokens using the constant product formula (x * y = k).

## Features

- Create liquidity pools (SOL + SPL token)
- Swap between SOL and SPL tokens
- Add liquidity to pools
- Remove liquidity from pools
- Constant product formula implementation
- Fee collection system (configurable fee rate)
- Slippage protection

## Prerequisites

- Rust 1.85.0 or later
- Solana CLI 1.18.1 or later
- SPL Token CLI

## Building

```bash
cargo build-bpf
```

## Testing

```bash
cargo test-bpf
```

## Program Instructions

### 1. Initialize Pool

Creates a new liquidity pool with initial SOL and token liquidity.

Required accounts:

- Signer (pool creator)
- Pool state account (PDA)
- Token mint
- LP token mint
- System program

### 2. Swap

Swap between SOL and SPL tokens.

Required accounts:

- Signer (user)
- Pool state account
- User's SOL account
- User's token account
- Pool's token account
- Token program

### 3. Add Liquidity

Add liquidity to the pool.

Required accounts:

- Signer (liquidity provider)
- Pool state account
- Provider's SOL account
- Provider's token account
- Pool's token account
- Provider's LP token account
- LP token mint
- Token program

### 4. Remove Liquidity

Remove liquidity from the pool.

Required accounts:

- Signer (liquidity provider)
- Pool state account
- Provider's SOL account
- Provider's token account
- Pool's token account
- Provider's LP token account
- LP token mint
- Token program

## Security Features

- Integer overflow/underflow protection
- Account ownership verification
- Signer verification
- Slippage protection
- Parameter validation

## Mathematical Implementation

The program uses the constant product formula (x * y = k) for price determination:

- x = SOL reserve
- y = Token reserve
- k = Constant product

For swaps:

- Output amount = (y * dx) / (x + dx)
- Where dx is the input amount

## License

MIT
