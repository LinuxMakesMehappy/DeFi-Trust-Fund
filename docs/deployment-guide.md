# DeFi Trust Fund - Deployment Guide

**Version**: 1.0  
**Target**: Solana Devnet/Testnet/Mainnet  
**Framework**: Anchor 0.29.0

---

## Prerequisites

### Development Environment

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.16.0/install)"

# Install Anchor
npm install -g @coral-xyz/anchor-cli@0.29.0

# Verify installations
solana --version
anchor --version
cargo --version
```

### Required Dependencies

```bash
# Install Node.js dependencies
npm install

# Install Rust dependencies
cargo build

# Install frontend dependencies
cd frontend && npm install
```

---

## Configuration

### 1. Network Configuration

#### Devnet Deployment
```bash
# Set Solana to devnet
solana config set --url https://api.devnet.solana.com

# Create/import wallet
solana-keygen new --outfile ~/.config/solana/id.json
# OR import existing wallet
solana-keygen recover --outfile ~/.config/solana/id.json

# Fund wallet for deployment
solana airdrop 10
```

#### Testnet/Mainnet Configuration
```bash
# For testnet
solana config set --url https://api.testnet.solana.com

# For mainnet
solana config set --url https://api.mainnet-beta.solana.com
```

### 2. Environment Variables

Create `.env` file in project root:

```bash
# Network Configuration
SOLANA_RPC_URL=https://api.devnet.solana.com
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com

# Program Configuration
DEFI_TRUST_FUND_PROGRAM_ID=Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS

# Oracle Configuration
SOL_USD_PRICE_FEED=H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG  # Pyth SOL/USD Devnet

# Security Configuration
MULTISIG_THRESHOLD=3
MULTISIG_SIGNERS=["pubkey1", "pubkey2", "pubkey3", "pubkey4", "pubkey5"]

# Frontend Configuration
REACT_APP_SOLANA_NETWORK=devnet
REACT_APP_PROGRAM_ID=Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS
```

### 3. Multi-Signature Setup

#### Generate Signer Keypairs
```bash
# Generate 5 keypairs for multi-sig
for i in {1..5}; do
  solana-keygen new --outfile ./keys/signer$i.json --no-bip39-passphrase
done

# Extract public keys
for i in {1..5}; do
  echo "Signer $i: $(solana-keygen pubkey ./keys/signer$i.json)"
done
```

#### Update Anchor.toml
```toml
[features]
seeds = false
skip-lint = false

[programs.devnet]
defi_trust_fund = "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
```

---

## Deployment Process

### 1. Smart Contract Deployment

#### Build and Test
```bash
# Clean previous builds
anchor clean

# Build the program
anchor build

# Run comprehensive tests
anchor test

# Run security-specific tests
npm run test:security
```

#### Deploy to Network
```bash
# Deploy to devnet
anchor deploy --provider.cluster devnet

# Verify deployment
solana program show Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS
```

### 2. Initialize Protocol

#### Create Initialization Script
```typescript
// scripts/initialize.ts
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DeFiTrustFund } from "../target/types/defi_trust_fund";

async function initialize() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  
  const program = anchor.workspace.DeFiTrustFund as Program<DeFiTrustFund>;
  
  // Initialize pool with security parameters
  const maxApy = 5000; // 50%
  const minCommitmentDays = 1;
  const maxCommitmentDays = 365;
  const solPriceFeed = new anchor.web3.PublicKey("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");
  
  await program.methods
    .initializePool(maxApy, minCommitmentDays, maxCommitmentDays, solPriceFeed)
    .rpc();
    
  console.log("Protocol initialized successfully!");
}

initialize().catch(console.error);
```

#### Run Initialization
```bash
npx ts-node scripts/initialize.ts
```

### 3. Multi-Signature Configuration

#### Setup Multi-Sig Governance
```typescript
// scripts/setup-multisig.ts
import * as anchor from "@coral-xyz/anchor";

async function setupMultisig() {
  // Add additional signers
  const signers = [
    new anchor.web3.PublicKey("signer1_pubkey"),
    new anchor.web3.PublicKey("signer2_pubkey"),
    new anchor.web3.PublicKey("signer3_pubkey"),
    new anchor.web3.PublicKey("signer4_pubkey"),
    new anchor.web3.PublicKey("signer5_pubkey"),
  ];
  
  for (const signer of signers.slice(1)) { // Skip first (already added)
    await program.methods
      .addMultisigSigner(signer)
      .rpc();
  }
  
  // Set threshold to 3-of-5
  await program.methods
    .updateMultisigThreshold(3)
    .rpc();
    
  console.log("Multi-signature governance configured!");
}
```

### 4. Frontend Deployment

#### Build Frontend
```bash
cd frontend

# Install dependencies
npm install

# Build for production
npm run build

# Test locally
npm start
```

#### Deploy to Hosting
```bash
# Example: Vercel deployment
npm install -g vercel
vercel --prod

# Example: Netlify deployment
npm install -g netlify-cli
netlify deploy --prod --dir=build
```

---

## Security Checklist

### Pre-Deployment Security ✅

- [ ] **Smart Contract Audit**: Comprehensive security review completed
- [ ] **Test Coverage**: 95%+ coverage with security tests
- [ ] **Dependency Audit**: All dependencies scanned for vulnerabilities
- [ ] **Multi-Signature Setup**: Proper governance configuration
- [ ] **Oracle Configuration**: Price feeds validated and tested
- [ ] **Emergency Controls**: Pause mechanisms tested
- [ ] **Rate Limiting**: Spam protection verified
- [ ] **Access Control**: Authorization properly configured

### Post-Deployment Verification ✅

- [ ] **Program Deployment**: Contract deployed to correct address
- [ ] **Initialization**: Pool properly initialized with correct parameters
- [ ] **Multi-Sig Active**: Governance controls operational
- [ ] **Oracle Integration**: Price feeds functioning correctly
- [ ] **Frontend Connection**: UI properly connected to contract
- [ ] **Security Monitoring**: Alerting systems operational
- [ ] **Emergency Procedures**: Response protocols documented

---

## Monitoring and Maintenance

### 1. Real-Time Monitoring

#### Set up monitoring dashboard
```typescript
// monitoring/dashboard.ts
import { Connection, PublicKey } from "@solana/web3.js";

const connection = new Connection("https://api.devnet.solana.com");
const programId = new PublicKey("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// Monitor program account changes
connection.onAccountChange(programId, (accountInfo) => {
  console.log("Program account updated:", accountInfo);
  // Send alerts for unusual activity
});
```

#### Key Metrics to Monitor
- Total Value Locked (TVL)
- Number of active users
- Transaction volume and frequency
- Error rates and failed transactions
- Oracle price feed status
- Multi-signature proposal activity

### 2. Security Monitoring

#### Automated Alerts
```bash
# Set up automated monitoring
npm install @solana/web3.js @solana/spl-token

# Create monitoring scripts for:
# - Large transactions (>10 SOL)
# - Failed transactions
# - Oracle price deviations
# - Emergency pause triggers
```

#### Manual Checks
- Weekly security review of transactions
- Monthly governance activity audit
- Quarterly dependency vulnerability scan
- Semi-annual security posture assessment

### 3. Maintenance Procedures

#### Regular Updates
```bash
# Update dependencies monthly
npm audit fix
cargo update

# Rebuild and test
anchor build
anchor test
```

#### Emergency Procedures
1. **Immediate Response**: Pause protocol if critical issue detected
2. **Assessment**: Evaluate scope and impact of issue
3. **Communication**: Notify users and community
4. **Resolution**: Implement fix and resume operations
5. **Post-Mortem**: Document incident and improve procedures

---

## Troubleshooting

### Common Deployment Issues

#### 1. Insufficient SOL for Deployment
```bash
# Check balance
solana balance

# Request more SOL (devnet/testnet only)
solana airdrop 10
```

#### 2. Program Build Failures
```bash
# Clean and rebuild
anchor clean
cargo clean
anchor build
```

#### 3. Test Failures
```bash
# Run specific test suite
anchor test tests/security-tests.ts

# Debug specific test
anchor test --skip-deploy tests/specific-test.ts
```

#### 4. RPC Connection Issues
```bash
# Check RPC status
solana cluster-version

# Switch RPC endpoint
solana config set --url https://api.devnet.solana.com
```

### Support Resources

- **Documentation**: [Anchor Docs](https://www.anchor-lang.com/)
- **Community**: [Solana Discord](https://discord.gg/solana)
- **Issues**: Create GitHub issue in project repository
- **Security**: Contact security@defitrustfund.com for security issues

---

## Production Deployment Checklist

### Mainnet Preparation

#### 1. Security Requirements ✅
- [ ] Third-party security audit completed
- [ ] Bug bounty program active for 30+ days
- [ ] Extended testnet operation (6+ weeks)
- [ ] Incident response procedures documented
- [ ] Emergency contact list prepared

#### 2. Operational Requirements ✅
- [ ] Multi-signature wallet setup with trusted parties
- [ ] Monitoring and alerting systems operational
- [ ] Legal compliance review completed
- [ ] Insurance coverage evaluated
- [ ] Community governance transition plan ready

#### 3. Technical Requirements ✅
- [ ] Mainnet RPC endpoints configured
- [ ] Production oracle feeds configured
- [ ] Frontend optimized and tested
- [ ] Backup procedures established
- [ ] Performance benchmarks validated

### Go-Live Process

1. **Final Security Review**: Complete pre-deployment checklist
2. **Mainnet Deployment**: Deploy contracts with limited TVL cap
3. **Community Testing**: Allow limited community access
4. **Gradual Scale-Up**: Increase TVL limits based on performance
5. **Full Launch**: Remove limits and announce public availability

---

**Deployment Support**: For deployment assistance, contact dev@defitrustfund.com