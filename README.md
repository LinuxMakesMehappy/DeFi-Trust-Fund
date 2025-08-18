# DeFi Trust Fund

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solana](https://img.shields.io/badge/Solana-1.16+-blue.svg)](https://solana.com/)
[![Anchor](https://img.shields.io/badge/Anchor-0.29+-purple.svg)](https://www.anchor-lang.com/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)

A revolutionary self-perpetuating proof of loyalty and liquidity DeFi protocol built on Solana using Anchor framework.

## 📋 Table of Contents

- [Overview](#-overview)
- [Key Features](#-key-features)
- [Smart Contract Architecture](#️-smart-contract-architecture)
- [Mathematical Models](#-mathematical-models)
- [Installation & Setup](#-installation--setup)
- [Testing](#-testing)
- [Documentation](#-documentation)
- [Security Features](#-security-features)
- [Economic Model](#-economic-model)
- [Contributing](#-contributing)
- [License](#-license)
- [Support](#-support)

## 🎯 Overview

The DeFi Trust Fund combines Jupiter's JupSOL liquid staking (~10-11% APY) with Kamino's leveraged yield farming (~16% APY at 4.6x leverage) in a unified smart contract. The protocol features hard-locked stakes, NFT-based authorization, tiered yields, and self-perpetuating capital growth through fee reinvestment.

## 🚀 Key Features

### 🏆 **Tiered Loyalty System**
- **Tier 1**: ~11.64% APY (Base tier)
- **Tier 2**: ~17.45% APY (1.5x multiplier)
- **Tier 3**: ~23.27% APY (2.0x multiplier, exempt from yield reinvestment)

### 🎯 **Proof of Loyalty & Liquidity**
- **Score Formula**: `5 * deposit_amount + 5 * days_staked`
- **Loyalty Multiplier**: `1.0 + (years_staked * 0.2)` (capped at 2x after 5 years)
- **Tier Assignment**: 
  - Tier 3: Top 10 users by score
  - Tier 2: Next 20 users by score
  - Tier 1: Remaining users

### 💎 **NFT-Based Authorization**
- **Stake Pass NFT**: Required for fund membership
- **Tier Badge NFT**: Represents loyalty/liquidity tier
- **Referrer Badge NFT**: Rewards for successful referrals
- **Soulbound**: Non-transferable, burned on exit

### 🔄 **Self-Perpetuating Mechanisms**
- **Fee Structure**: 0.5% deposit fee reinvested into fund TVL
- **APY Reinvestment**: 20% of yields reinvested (Tier 3 exempt)
- **Auto-Reinvest Options**: User-controlled yield reinvestment (0-100%)
- **Referral Bonuses**: 10% yield multiplier for referred users (3 months)
- **Permanent Capital**: Fees and reinvestments locked forever

### 📊 **Monthly Rebalancing**
- Automatic tier recalculation every 30 days
- Batch processing (25 users per transaction)
- On-chain sorting and tier assignment
- NFT metadata updates with new tier info
- Inactivity penalties for low-scoring users

### 🌟 **Enhanced Features**
- **Loyalty Multipliers**: Exponential rewards for long-term staking
- **Referral System**: Network effects through referral bonuses
- **Auto-Reinvest**: User-controlled yield reinvestment strategies
- **Yield Reserve Management**: Centralized fee and reinvestment handling
- **Early Adopter Bonuses**: Special rewards for the first 10 users

## 🏗️ Smart Contract Architecture

### Core Instructions

1. **`initialize_fund`**: Creates new fund with NFT mints and metadata
2. **`deposit`**: Stakes SOL with fee collection, NFT minting, and referral tracking
3. **`claim_yields`**: Claims yields with tier-based multipliers and auto-reinvest
4. **`trigger_rebalance`**: Permissionless monthly rebalance trigger
5. **`rebalance_tiers_batch`**: Batch tier recalculation and NFT updates
6. **`finalize_rebalance`**: On-chain sorting and tier assignment
7. **`burn_nft`**: Burns NFTs on user exit
8. **`set_auto_reinvest`**: User-controlled reinvestment percentage
9. **`deposit_yield_reserve`**: Manual yield reserve deployment
10. **`find_open_fund`**: Returns available fund index
11. **`get_user_info`**: Returns user stake data

### Account Structure

```rust
FundManager {
    fund_count: u64,
}

Fund {
    stake_nft_mint: Pubkey,
    tier_nft_mint: Pubkey,
    referrer_nft_mint: Pubkey,
    total_deposits: u64,
    user_count: u64,
    early_adopters: u8,
    last_rebalance_timestamp: u64,
}

UserStake {
    deposit_amount: u64,
    stake_timestamp: u64,
    tier: u8,
    referrer: Pubkey,
    referral_expiry: u64,
    auto_reinvest_percentage: u8,
}

TempScores {
    scores: Vec<TempScore>,
}

TempScore {
    user: Pubkey,
    score: u64,
}
```

## 📈 Mathematical Models

### Tier Score Calculation
```
base_score = 5 * deposit_amount + 5 * days_staked
loyalty_multiplier = 1.0 + (years_staked * 0.2).min(1.0)
final_score = base_score * loyalty_multiplier
```

### Fee for Permanent Capital
```
fee_amount = deposit_amount * 0.005
fund_tvl_new = fund_tvl + fee_amount
```

### APY Reinvestment
```
protocol_reinvestment = fund_growth * 0.20  // Tier 3 exempt
user_reinvestment = fund_growth * (auto_reinvest_percentage / 100)
total_reinvestment = protocol_reinvestment + user_reinvestment
user_yield = fund_growth - total_reinvestment
```

### Referral Bonus
```
referral_multiplier = 1.1  // 10% bonus for 3 months
total_weight = base_weight * referral_multiplier * early_adopter_multiplier
```

## 🔧 Installation & Setup

### Prerequisites
- Rust 1.70+
- Solana CLI 1.16+
- Anchor CLI 0.29+
- Node.js 18+

### Quick Start
```bash
# Clone the repository
git clone https://github.com/LinuxMakesMehappy/DeFi-Trust-Fund.git
cd DeFi-Trust-Fund

# Install dependencies
npm install

# Build the program
anchor build

# Run tests
anchor test

# Deploy to localnet
anchor deploy
```

## 🧪 Testing

The test suite covers all major functionality:

```bash
# Run all tests
anchor test

# Run specific test file
anchor test tests/defi-trust-fund.ts
```

## 📚 Documentation

- [Mathematical Analysis](./docs/mathematical-analysis.md)
- [Deployment Guide](./docs/deployment-guide.md)

## 🔒 Security Features

- **Checked Arithmetic**: All calculations use safe math operations
- **Access Control**: NFT-based authorization for all operations
- **Batch Processing**: Efficient on-chain operations within compute limits
- **Emergency Exit**: NFT burning mechanism for user exits
- **Transparency**: All operations emit on-chain events

## 🌐 Economic Model

### Self-Perpetuation Formula
```
TVL_t+1 = TVL_t + (TVL_t * APY * Reinvestment_Rate) + Fees + Referral_Growth
```

Where:
- **Reinvestment_Rate** = 20% (protocol) + user_auto_reinvest
- **Fees** = 0.5% of all deposits
- **Referral_Growth** = Additional TVL from referred users

### Growth Phases
1. **Initial Phase**: Relies on deposits and fees
2. **Growth Phase**: Reinvested yields compound TVL
3. **Maturity Phase**: Protocol generates sufficient yields to sustain growth

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🚨 Disclaimer

This software is for educational purposes. Use at your own risk. The authors are not responsible for any financial losses.

## 📞 Support

- **GitHub Issues**: [Report bugs or request features](https://github.com/yourusername/DeFi-Trust-Fund/issues)
- **Documentation**: Check the [docs](./docs/) folder for detailed guides
- **Discord**: Join our community for discussions and support

---

**DeFi Trust Fund** - Building the future of decentralized finance through loyalty and liquidity. 🚀

## 📊 Repository Statistics

![GitHub stars](https://img.shields.io/github/stars/yourusername/DeFi-Trust-Fund?style=social)
![GitHub forks](https://img.shields.io/github/forks/yourusername/DeFi-Trust-Fund?style=social)
![GitHub issues](https://img.shields.io/github/issues/yourusername/DeFi-Trust-Fund)
![GitHub pull requests](https://img.shields.io/github/issues-pr/yourusername/DeFi-Trust-Fund)
![GitHub contributors](https://img.shields.io/github/contributors/yourusername/DeFi-Trust-Fund)
![GitHub last commit](https://img.shields.io/github/last-commit/yourusername/DeFi-Trust-Fund)

---

⭐ **Star this repository if you find it helpful!**

