# Token Exchange Program Workflow

## Program Overview

This Solana program implements a simple token exchange (DEX) that allows users to:

- Create liquidity pools for SOL/SPL token pairs
- Add liquidity to pools
- Swap between SOL and SPL tokens
- Remove liquidity from pools

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

## Program Architecture

### 1. Account Structure

- **PoolState Account**: Stores pool information
  - SOL reserve
  - Token reserve
  - LP token mint
  - Fee rate
  - Authority
  - Initialization status

- **User Position Account**: Tracks user's LP tokens
  - Owner
  - LP token amount

### 2. Data Flow Diagram

```mermaid
graph TD
    subgraph Accounts
        A[PoolState Account] -->|Stores| B[Pool Data]
        C[User Position Account] -->|Tracks| D[LP Tokens]
    end
    
    subgraph Operations
        E[Initialize] -->|Creates| A
        F[Swap] -->|Updates| A
        G[Add Liquidity] -->|Updates| A
        G -->|Updates| C
        H[Remove Liquidity] -->|Updates| A
        H -->|Updates| C
    end
```

### 2. Instructions

#### Initialize Pool

```
