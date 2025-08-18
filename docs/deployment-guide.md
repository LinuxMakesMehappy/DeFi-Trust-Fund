# Deployment Guide: DeFi Trust Fund

## Prerequisites

### System Requirements
- **Operating System**: Linux, macOS, or Windows
- **RAM**: Minimum 8GB, Recommended 16GB
- **Storage**: Minimum 50GB free space
- **Network**: Stable internet connection

### Software Requirements
- **Rust**: 1.70+ ([Installation Guide](https://rustup.rs/))
- **Solana CLI**: 1.16+ ([Installation Guide](https://docs.solana.com/cli/install-solana-cli-tools))
- **Anchor CLI**: 0.29+ ([Installation Guide](https://book.anchor-lang.com/getting_started/installation.html))
- **Node.js**: 16+ ([Installation Guide](https://nodejs.org/))
- **Git**: Latest version

### Wallet Setup
- **Solana Wallet**: Phantom, Solflare, or other compatible wallet
- **SOL Balance**: Minimum 5 SOL for deployment and testing
- **RPC Endpoint**: Access to Solana RPC (Mainnet/Devnet/Testnet)

## Step 1: Environment Setup

### 1.1 Clone Repository
```bash
git clone https://github.com/your-org/jupiter-kamino-stake.git
cd jupiter-kamino-stake
```

### 1.2 Install Dependencies
```bash
# Install Rust dependencies
cargo build

# Install Node.js dependencies
npm install
```

### 1.3 Configure Solana CLI
```bash
# Set Solana cluster (choose one)
solana config set --url devnet    # For development
solana config set --url testnet   # For testing
solana config set --url mainnet   # For production

# Verify configuration
solana config get
```

### 1.4 Create Wallet (if needed)
```bash
# Generate new keypair
solana-keygen new --outfile ~/.config/solana/id.json

# Set as default
solana config set --keypair ~/.config/solana/id.json

# Check balance
solana balance
```

## Step 2: Program Configuration

### 2.1 Update Program ID
```bash
# Generate new program ID
solana-keygen new --outfile target/deploy/jupiter_kamino_stake-keypair.json

# Get program ID
solana-keygen pubkey target/deploy/jupiter_kamino_stake-keypair.json
```

### 2.2 Update Configuration Files

#### Update `lib.rs`
```rust
declare_id!("YOUR_GENERATED_PROGRAM_ID_HERE");
```

#### Update `Anchor.toml`
```toml
[programs.devnet]
jupiter_kamino_stake = "YOUR_GENERATED_PROGRAM_ID_HERE"

[programs.testnet]
jupiter_kamino_stake = "YOUR_GENERATED_PROGRAM_ID_HERE"

[programs.mainnet]
jupiter_kamino_stake = "YOUR_GENERATED_PROGRAM_ID_HERE"
```

### 2.3 Configure External Dependencies

#### Jupiter API Configuration
```typescript
// Add to your client configuration
const JUPITER_API_URL = "https://quote-api.jup.ag/v6";
const JUPITER_PROGRAM_ID = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
```

#### Kamino Configuration
```typescript
// Add to your client configuration
const KAMINO_PROGRAM_ID = "KLend2g3c5MGDmXenSmC16qBkmga6DhbVrJmzexvef";
const KAMINO_VAULT_ADDRESS = "YOUR_KAMINO_VAULT_ADDRESS";
```

## Step 3: Build and Test

### 3.1 Build Program
```bash
# Build the program
anchor build

# Verify build
ls target/deploy/
```

### 3.2 Run Tests
```bash
# Run all tests
anchor test

# Run specific test
anchor test --skip-local-validator

# Run with verbose output
anchor test -- --nocapture
```

### 3.3 Verify Build Artifacts
```bash
# Check program binary
ls -la target/deploy/jupiter_kamino_stake.so

# Verify IDL
cat target/idl/jupiter_kamino_stake.json
```

## Step 4: Deployment

### 4.1 Deploy to Devnet (Recommended First)
```bash
# Deploy program
anchor deploy --provider.cluster devnet

# Verify deployment
solana program show YOUR_PROGRAM_ID --url devnet
```

### 4.2 Deploy to Testnet
```bash
# Switch to testnet
solana config set --url testnet

# Deploy program
anchor deploy --provider.cluster testnet

# Verify deployment
solana program show YOUR_PROGRAM_ID --url testnet
```

### 4.3 Deploy to Mainnet
```bash
# Switch to mainnet
solana config set --url mainnet

# Deploy program (BE CAREFUL!)
anchor deploy --provider.cluster mainnet

# Verify deployment
solana program show YOUR_PROGRAM_ID --url mainnet
```

## Step 5: Post-Deployment Setup

### 5.1 Initialize Fund Manager
```typescript
import { Program, AnchorProvider, web3 } from "@coral-xyz/anchor";
import { JupiterKaminoStake } from "../target/types/jupiter_kamino_stake";

const provider = AnchorProvider.env();
const program = new Program<JupiterKaminoStake>(IDL, PROGRAM_ID, provider);

// Initialize fund manager
const fundManagerPda = web3.PublicKey.findProgramAddressSync(
  [Buffer.from("fund_manager")],
  program.programId
)[0];

await program.methods
  .initializeFund(new web3.BN(0))
  .accounts({
    admin: provider.wallet.publicKey,
    fundManager: fundManagerPda,
    // ... other accounts
  })
  .rpc();
```

### 5.2 Create First Fund
```typescript
// Create fund with index 0
const fundPda = web3.PublicKey.findProgramAddressSync(
  [Buffer.from("fund"), new web3.BN(0).toArrayLike(Buffer, "le", 8)],
  program.programId
)[0];

await program.methods
  .initializeFund(new web3.BN(0))
  .accounts({
    admin: provider.wallet.publicKey,
    fundManager: fundManagerPda,
    fund: fundPda,
    // ... other accounts
  })
  .rpc();
```

### 5.3 Verify Setup
```typescript
// Check fund manager
const fundManager = await program.account.fundManager.fetch(fundManagerPda);
console.log("Fund count:", fundManager.fundCount.toNumber());

// Check fund
const fund = await program.account.fund.fetch(fundPda);
console.log("Fund total deposits:", fund.totalDeposits.toNumber());
console.log("Fund user count:", fund.userCount);
```

## Step 6: Integration Testing

### 6.1 Test Deposit Functionality
```typescript
// Test user deposit
const user = web3.Keypair.generate();
const depositAmount = new web3.BN(1 * web3.LAMPORTS_PER_SOL);

await program.methods
  .deposit(new web3.BN(0), depositAmount)
  .accounts({
    user: user.publicKey,
    fund: fundPda,
    // ... other accounts
  })
  .signers([user])
  .rpc();
```

### 6.2 Test Yield Claiming
```typescript
// Test yield claiming
await program.methods
  .claimYields(new web3.BN(0))
  .accounts({
    user: user.publicKey,
    fund: fundPda,
    // ... other accounts
  })
  .signers([user])
  .rpc();
```

### 6.3 Test Rebalancing
```typescript
// Test rebalancing trigger
await program.methods
  .triggerRebalance(new web3.BN(0))
  .accounts({
    caller: provider.wallet.publicKey,
    fund: fundPda,
  })
  .rpc();
```

## Step 7: Production Configuration

### 7.1 Environment Variables
```bash
# Create .env file
cat > .env << EOF
SOLANA_CLUSTER=mainnet
PROGRAM_ID=YOUR_PROGRAM_ID
JUPITER_API_URL=https://quote-api.jup.ag/v6
KAMINO_PROGRAM_ID=KLend2g3c5MGDmXenSmC16qBkmga6DhbVrJmzexvef
PYTH_PROGRAM_ID=HZRCwxP2Vq9PCpPXooayhJ2bxTpo5xfpQrwB1svh332p
EOF
```

### 7.2 Monitoring Setup
```typescript
// Add monitoring for events
program.addEventListener('TierRebalanceEvent', (event) => {
  console.log('Tier rebalance:', event);
});

program.addEventListener('FeeReinvestmentEvent', (event) => {
  console.log('Fee reinvestment:', event);
});

program.addEventListener('YieldReinvestmentEvent', (event) => {
  console.log('Yield reinvestment:', event);
});
```

### 7.3 Security Considerations
- **Multi-sig Wallet**: Use multi-signature wallet for admin operations
- **Rate Limiting**: Implement rate limiting for deposit/claim operations
- **Emergency Pause**: Add emergency pause functionality
- **Audit**: Conduct security audit before mainnet deployment

## Step 8: Maintenance

### 8.1 Regular Monitoring
```bash
# Monitor program logs
solana logs YOUR_PROGRAM_ID

# Check program balance
solana balance YOUR_PROGRAM_ID

# Monitor account sizes
solana account YOUR_PROGRAM_ID
```

### 8.2 Backup Procedures
```bash
# Backup program binary
cp target/deploy/jupiter_kamino_stake.so backup/

# Backup IDL
cp target/idl/jupiter_kamino_stake.json backup/

# Backup keypair (SECURE!)
cp target/deploy/jupiter_kamino_stake-keypair.json backup/
```

### 8.3 Update Procedures
```bash
# Build new version
anchor build

# Deploy update
anchor deploy --program-id YOUR_PROGRAM_ID

# Verify update
solana program show YOUR_PROGRAM_ID
```

## Troubleshooting

### Common Issues

#### Build Errors
```bash
# Clean and rebuild
anchor clean
anchor build

# Update dependencies
cargo update
```

#### Deployment Errors
```bash
# Check balance
solana balance

# Check cluster
solana config get

# Verify program ID
solana-keygen pubkey target/deploy/jupiter_kamino_stake-keypair.json
```

#### Test Failures
```bash
# Run with local validator
anchor test --skip-local-validator

# Check logs
anchor test -- --nocapture
```

### Support Resources
- [Anchor Documentation](https://book.anchor-lang.com/)
- [Solana Documentation](https://docs.solana.com/)
- [Jupiter API Documentation](https://station.jup.ag/docs/apis/swap-api)
- [Kamino Documentation](https://docs.kamino.finance/)

## Conclusion

This deployment guide provides a comprehensive approach to deploying the Jupiter-Kamino Staking Protocol. Always test thoroughly on devnet and testnet before deploying to mainnet, and ensure proper security measures are in place.

For additional support, please refer to the project documentation or create an issue on the GitHub repository.

