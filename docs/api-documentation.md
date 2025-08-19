# DeFi Trust Fund - API Documentation

**Version**: 1.0  
**Framework**: Anchor 0.29.0  
**Network**: Solana

---

## Program Interface

### Program ID
```
Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS
```

### IDL Location
```
target/idl/defi_trust_fund.json
```

---

## Core Instructions

### 1. Initialize Pool

Initialize the staking pool with security parameters and oracle integration.

#### Method Signature
```rust
pub fn initialize_pool(
    ctx: Context<InitializePool>,
    max_apy: u64,
    min_commitment_days: u64,
    max_commitment_days: u64,
    sol_price_feed: Pubkey,
) -> Result<()>
```

#### Parameters
- `max_apy`: Maximum APY allowed (in basis points, e.g., 5000 = 50%)
- `min_commitment_days`: Minimum commitment period (minimum: 1)
- `max_commitment_days`: Maximum commitment period (maximum: 365)
- `sol_price_feed`: Pyth SOL/USD price feed account

#### Accounts
```typescript
interface InitializePool {
  admin: Signer;           // Pool administrator
  pool: Account<Pool>;     // Pool state account
  poolVault: SystemAccount; // Pool treasury
  systemProgram: Program<System>;
}
```

#### Example Usage
```typescript
const maxApy = 5000; // 50%
const minDays = 1;
const maxDays = 365;
const priceFeed = new PublicKey("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");

await program.methods
  .initializePool(maxApy, minDays, maxDays, priceFeed)
  .accounts({
    admin: adminKeypair.publicKey,
    pool: poolKeypair.publicKey,
    poolVault: poolVaultKeypair.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([adminKeypair, poolKeypair, poolVaultKeypair])
  .rpc();
```

### 2. Stake

Deposit SOL with commitment period and slippage protection.

#### Method Signature
```rust
pub fn stake(
    ctx: Context<Stake>,
    amount: u64,
    committed_days: u64,
    min_expected_amount: u64,
    deadline: i64,
) -> Result<()>
```

#### Parameters
- `amount`: Amount to stake in lamports
- `committed_days`: Commitment period (1-365 days)
- `min_expected_amount`: Minimum expected amount after fees (slippage protection)
- `deadline`: Transaction deadline timestamp

#### Accounts
```typescript
interface Stake {
  user: Signer;                    // User making deposit
  pool: Account<Pool>;             // Pool state
  userStake: Account<UserStake>;   // User stake account
  poolVault: SystemAccount;        // Pool treasury
  systemProgram: Program<System>;
}
```

#### Example Usage
```typescript
const stakeAmount = 5 * LAMPORTS_PER_SOL; // 5 SOL
const commitmentDays = 30;
const fee = stakeAmount * 0.005; // 0.5% fee
const minExpected = stakeAmount - fee - (stakeAmount * 0.01); // 1% slippage
const deadline = Math.floor(Date.now() / 1000) + 300; // 5 minutes

await program.methods
  .stake(
    new anchor.BN(stakeAmount),
    commitmentDays,
    new anchor.BN(minExpected),
    new anchor.BN(deadline)
  )
  .accounts({
    user: userKeypair.publicKey,
    pool: poolPublicKey,
    userStake: userStakePublicKey,
    poolVault: poolVaultPublicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([userKeypair])
  .rpc();
```

### 3. Claim Yields

Claim accumulated yields after commitment period completion.

#### Method Signature
```rust
pub fn claim_yields(ctx: Context<ClaimYields>) -> Result<()>
```

#### Accounts
```typescript
interface ClaimYields {
  user: Signer;                    // User claiming yields
  pool: Account<Pool>;             // Pool state
  userStake: Account<UserStake>;   // User stake account
  poolVault: SystemAccount;        // Pool treasury
  systemProgram: Program<System>;
}
```

#### Requirements
- Commitment period must be completed
- User must have positive stake amount
- Pool must have sufficient funds

#### Example Usage
```typescript
await program.methods
  .claimYields()
  .accounts({
    user: userKeypair.publicKey,
    pool: poolPublicKey,
    userStake: userStakePublicKey,
    poolVault: poolVaultPublicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([userKeypair])
  .rpc();
```

### 4. Unstake

Withdraw principal and yields (with penalties for early exit).

#### Method Signature
```rust
pub fn unstake(ctx: Context<Unstake>) -> Result<()>
```

#### Accounts
```typescript
interface Unstake {
  user: Signer;                    // User unstaking
  pool: Account<Pool>;             // Pool state
  userStake: Account<UserStake>;   // User stake account
  poolVault: SystemAccount;        // Pool treasury
  systemProgram: Program<System>;
}
```

#### Penalty Structure
- **Complete Commitment**: Full principal + yields
- **Early Exit**: Principal - 5% penalty, no yields

#### Example Usage
```typescript
await program.methods
  .unstake()
  .accounts({
    user: userKeypair.publicKey,
    pool: poolPublicKey,
    userStake: userStakePublicKey,
    poolVault: poolVaultPublicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([userKeypair])
  .rpc();
```

---

## Governance Instructions

### 1. Propose Admin Action

Propose an administrative action requiring multi-signature approval.

#### Method Signature
```rust
pub fn propose_admin_action(
    ctx: Context<ProposeAction>,
    action_type: ActionType,
    parameters: ActionParameters,
) -> Result<()>
```

#### Action Types
```rust
pub enum ActionType {
    UpdateApy,
    UpdateFee,
    EmergencyPause,
    EmergencyUnpause,
    WithdrawFees,
    UpdateLimits,
}
```

#### Example Usage
```typescript
await program.methods
  .proposeAdminAction(
    { updateApy: {} },
    {
      newApy: 1500, // 15%
      newFee: null,
      pauseReason: null,
      newUserLimit: null,
      newPoolLimit: null,
      withdrawalAmount: null,
    }
  )
  .accounts({
    proposer: proposerKeypair.publicKey,
    pool: poolPublicKey,
  })
  .signers([proposerKeypair])
  .rpc();
```

### 2. Sign Admin Action

Sign a pending administrative action.

#### Method Signature
```rust
pub fn sign_admin_action(ctx: Context<SignAction>) -> Result<()>
```

#### Example Usage
```typescript
await program.methods
  .signAdminAction()
  .accounts({
    signer: signerKeypair.publicKey,
    pool: poolPublicKey,
  })
  .signers([signerKeypair])
  .rpc();
```

### 3. Execute Admin Action

Execute a pending action after timelock and sufficient signatures.

#### Method Signature
```rust
pub fn execute_admin_action(ctx: Context<ExecuteAction>) -> Result<()>
```

#### Requirements
- Timelock period expired (24 hours)
- Sufficient signatures (3-of-5 minimum)
- Valid action parameters

#### Example Usage
```typescript
await program.methods
  .executeAdminAction()
  .accounts({
    executor: executorKeypair.publicKey,
    pool: poolPublicKey,
  })
  .signers([executorKeypair])
  .rpc();
```

---

## Oracle Instructions

### Update SOL Price

Update the SOL price from Pyth oracle.

#### Method Signature
```rust
pub fn update_sol_price(ctx: Context<UpdatePrice>) -> Result<()>
```

#### Accounts
```typescript
interface UpdatePrice {
  admin: Signer;                   // Admin account
  pool: Account<Pool>;             // Pool state
  priceFeed: AccountInfo;          // Pyth price feed
}
```

#### Example Usage
```typescript
await program.methods
  .updateSolPrice()
  .accounts({
    admin: adminKeypair.publicKey,
    pool: poolPublicKey,
    priceFeed: pythPriceFeedPublicKey,
  })
  .signers([adminKeypair])
  .rpc();
```

---

## Account Structures

### Pool Account

```rust
pub struct Pool {
    pub admin: Pubkey,                    // Pool administrator
    pub total_staked: u64,                // Total SOL staked
    pub total_users: u64,                 // Number of users
    pub apy: u64,                         // Current APY (basis points)
    pub deposit_fee: u64,                 // Deposit fee (basis points)
    pub max_apy: u64,                     // Maximum allowed APY
    pub min_commitment_days: u64,         // Minimum commitment
    pub max_commitment_days: u64,         // Maximum commitment
    pub is_active: bool,                  // Pool status
    pub is_paused: bool,                  // Emergency pause
    pub emergency_pause_reason: String,   // Pause reason
    pub total_fees_collected: u64,        // Total fees
    pub total_yields_paid: u64,           // Total yields paid
    pub last_rebalance_timestamp: i64,    // Last tier rebalance
    pub max_deposit_per_user: u64,        // User deposit limit
    pub max_total_staked: u64,            // Pool TVL limit
    pub min_stake_amount: u64,            // Minimum stake
    pub max_stake_amount: u64,            // Maximum stake
    pub created_at: i64,                  // Creation time
    pub updated_at: i64,                  // Last update
    // Oracle fields
    pub sol_price_feed: Pubkey,           // Price feed account
    pub price_staleness_threshold: u64,   // Max staleness (seconds)
    pub max_price_deviation: u64,         // Max deviation (bps)
    pub circuit_breaker_threshold: u64,   // Circuit breaker (bps)
    pub last_price_update: i64,           // Last price update
    // Multi-signature fields
    pub multisig_threshold: u8,           // Required signatures
    pub multisig_signers: Vec<Pubkey>,    // Authorized signers
    pub pending_admin_action: Option<PendingAction>, // Pending action
    pub action_timelock_delay: u64,       // Timelock delay
    // Security fields
    pub reentrancy_guard: bool,           // Reentrancy protection
    pub max_slippage_bps: u64,            // Max slippage (bps)
    pub transaction_deadline: u64,        // TX deadline (seconds)
    pub min_block_delay: u64,             // MEV protection blocks
    pub last_large_operation_slot: u64,   // Last large op slot
}
```

### UserStake Account

```rust
pub struct UserStake {
    pub user: Pubkey,                     // User public key
    pub amount: u64,                      // Staked amount
    pub committed_days: u64,              // Commitment period
    pub stake_timestamp: i64,             // Stake time
    pub last_claim_timestamp: i64,        // Last claim
    pub total_staked_lifetime: u64,       // Lifetime staked
    pub total_days_staked: u64,           // Lifetime days
    pub total_yields_claimed: u64,        // Lifetime yields
    // Rate limiting fields
    pub last_claim_attempt: i64,          // Last claim attempt
    pub claim_attempts_count: u64,        // Claim attempts
    pub last_stake_attempt: i64,          // Last stake attempt
    pub stake_attempts_count: u64,        // Stake attempts
}
```

---

## Events

### StakeEvent
```rust
pub struct StakeEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub committed_days: u64,
    pub fee_amount: u64,
    pub timestamp: i64,
}
```

### ClaimEvent
```rust
pub struct ClaimEvent {
    pub user: Pubkey,
    pub yields: u64,
    pub timestamp: i64,
}
```

### UnstakeEvent
```rust
pub struct UnstakeEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub yields: u64,
    pub penalty: u64,
    pub timestamp: i64,
}
```

### EmergencyPauseEvent
```rust
pub struct EmergencyPauseEvent {
    pub admin: Pubkey,
    pub reason: String,
    pub timestamp: i64,
}
```

---

## Error Codes

```rust
pub enum ErrorCode {
    ZeroAmount = 6000,
    AmountTooSmall = 6001,
    AmountTooLarge = 6002,
    InvalidCommitment = 6003,
    PoolInactive = 6004,
    PoolPaused = 6005,
    ArithmeticOverflow = 6006,
    NoYieldsToClaim = 6007,
    CommitmentNotMet = 6008,
    NoStake = 6009,
    Unauthorized = 6010,
    InvalidApy = 6011,
    InvalidFee = 6012,
    InvalidLimit = 6013,
    InsufficientFunds = 6014,
    UserLimitExceeded = 6015,
    PoolLimitExceeded = 6016,
    StalePriceData = 6017,
    PriceDeviationTooHigh = 6018,
    CircuitBreakerTriggered = 6019,
    InvalidOracle = 6020,
    PendingActionExists = 6021,
    NoPendingAction = 6022,
    AlreadySigned = 6023,
    TimelockNotExpired = 6024,
    InsufficientSignatures = 6025,
    InvalidAction = 6026,
    TooManySigners = 6027,
    SignerAlreadyExists = 6028,
    InvalidThreshold = 6029,
    ReentrancyDetected = 6030,
    RateLimitExceeded = 6031,
    SlippageExceeded = 6032,
    TransactionExpired = 6033,
    MevProtectionActive = 6034,
}
```

---

## Rate Limits

### Constants
```rust
const RATE_LIMIT_WINDOW: i64 = 3600;    // 1 hour
const MAX_CLAIMS_PER_HOUR: u64 = 10;    // Max claims
const MAX_STAKES_PER_HOUR: u64 = 5;     // Max stakes
const COOLDOWN_PERIOD: i64 = 300;       // 5 minutes
```

### Limits
- **Claims**: Maximum 10 per hour per user
- **Stakes**: Maximum 5 per hour per user
- **Cooldown**: 5 minutes between operations
- **Window Reset**: Automatic after 1 hour

---

## Integration Examples

### TypeScript SDK Integration

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DeFiTrustFund } from "./types/defi_trust_fund";

class DeFiTrustFundSDK {
  constructor(
    private program: Program<DeFiTrustFund>,
    private poolPublicKey: anchor.web3.PublicKey
  ) {}

  async stake(
    amount: number,
    days: number,
    userKeypair: anchor.web3.Keypair
  ): Promise<string> {
    const amountLamports = amount * anchor.web3.LAMPORTS_PER_SOL;
    const fee = amountLamports * 0.005;
    const minExpected = amountLamports - fee - (amountLamports * 0.01);
    const deadline = Math.floor(Date.now() / 1000) + 300;

    return await this.program.methods
      .stake(
        new anchor.BN(amountLamports),
        days,
        new anchor.BN(minExpected),
        new anchor.BN(deadline)
      )
      .accounts({
        user: userKeypair.publicKey,
        pool: this.poolPublicKey,
        userStake: this.getUserStakeAddress(userKeypair.publicKey),
        poolVault: this.getPoolVaultAddress(),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([userKeypair])
      .rpc();
  }

  async claimYields(userKeypair: anchor.web3.Keypair): Promise<string> {
    return await this.program.methods
      .claimYields()
      .accounts({
        user: userKeypair.publicKey,
        pool: this.poolPublicKey,
        userStake: this.getUserStakeAddress(userKeypair.publicKey),
        poolVault: this.getPoolVaultAddress(),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([userKeypair])
      .rpc();
  }

  private getUserStakeAddress(user: anchor.web3.PublicKey): anchor.web3.PublicKey {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_stake"), user.toBuffer()],
      this.program.programId
    )[0];
  }

  private getPoolVaultAddress(): anchor.web3.PublicKey {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("pool_vault")],
      this.program.programId
    )[0];
  }
}
```

### React Hook Integration

```typescript
import { useProgram, useAnchorWallet } from "@solana/wallet-adapter-react";
import { DeFiTrustFund } from "./types/defi_trust_fund";

export function useDeFiTrustFund() {
  const wallet = useAnchorWallet();
  const program = useProgram<DeFiTrustFund>();
  
  const stake = async (amount: number, days: number) => {
    if (!wallet || !program) throw new Error("Wallet not connected");
    
    const sdk = new DeFiTrustFundSDK(program, POOL_PUBLIC_KEY);
    return await sdk.stake(amount, days, wallet);
  };
  
  const claimYields = async () => {
    if (!wallet || !program) throw new Error("Wallet not connected");
    
    const sdk = new DeFiTrustFundSDK(program, POOL_PUBLIC_KEY);
    return await sdk.claimYields(wallet);
  };
  
  return { stake, claimYields };
}
```

---

## Testing

### Unit Tests

```typescript
describe("DeFi Trust Fund", () => {
  it("should stake successfully", async () => {
    const amount = 5 * LAMPORTS_PER_SOL;
    const days = 30;
    
    const tx = await program.methods
      .stake(new BN(amount), days, new BN(minExpected), new BN(deadline))
      .accounts({ /* ... */ })
      .rpc();
      
    const userStake = await program.account.userStake.fetch(userStakeAddress);
    expect(userStake.amount.toNumber()).to.be.greaterThan(0);
  });
});
```

### Integration Tests

```typescript
describe("Integration Tests", () => {
  it("should complete full stake-claim-unstake cycle", async () => {
    // Stake
    await stake(5, 1); // 5 SOL for 1 day
    
    // Wait for commitment period
    await sleep(86400 * 1000); // 1 day
    
    // Claim yields
    await claimYields();
    
    // Unstake
    await unstake();
  });
});
```

---

**Support**: For API questions, contact dev@defitrustfund.com
