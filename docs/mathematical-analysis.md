# Mathematical Analysis: DeFi Trust Fund

## Overview

This document provides a comprehensive mathematical analysis of the Jupiter-Kamino Staking Protocol's self-perpetuating mechanisms, tiered yield system, and economic models.

## 1. Tier Score Calculation

### Formula
```
score = 5 * deposit_amount + 5 * (current_time - stake_timestamp) / (365 * 24 * 3600)
```

### Parameters
- `deposit_amount`: User's total staked amount in SOL
- `current_time`: Current Unix timestamp
- `stake_timestamp`: User's initial stake timestamp
- `365 * 24 * 3600`: Seconds in a year

### Tier Assignment
- **Tier 3**: Score > 15 (High loyalty/liquidity)
- **Tier 2**: Score > 7 (Medium loyalty/liquidity)
- **Tier 1**: Score ≤ 7 (Base tier)

### Example Calculations

#### User A: 1 SOL staked for 365 days
```
score = 5 * 1 + 5 * 365 / 365 = 5 + 5 = 10
Result: Tier 2 (10 > 7)
```

#### User B: 3 SOL staked for 30 days
```
score = 5 * 3 + 5 * 30 / 365 = 15 + 0.41 = 15.41
Result: Tier 3 (15.41 > 15)
```

#### User C: 0.5 SOL staked for 60 days
```
score = 5 * 0.5 + 5 * 60 / 365 = 2.5 + 0.82 = 3.32
Result: Tier 1 (3.32 ≤ 7)
```

## 2. Fee Structure for Permanent Capital Growth

### Deposit Fee Calculation
```
fee_amount = deposit_amount * 0.005
net_deposit = deposit_amount - fee_amount
```

### Example: 100 SOL Deposit
```
fee_amount = 100 * 0.005 = 0.5 SOL
net_deposit = 100 - 0.5 = 99.5 SOL
fund_tvl_increase = 99.5 + 0.5 = 100 SOL (fee reinvested)
```

### Cumulative Effect
Over 100 deposits of 1 SOL each:
```
total_fees = 100 * 0.005 = 0.5 SOL
permanent_capital_growth = 0.5 SOL
```

## 3. APY Reinvestment for Self-Growth

### Base APY Structure
- **Base APY**: 16% (from Kamino leveraged yield farming)
- **Tier 1**: 11.64% (1.0x multiplier)
- **Tier 2**: 17.45% (1.5x multiplier)
- **Tier 3**: 23.27% (2.0x multiplier)

### Reinvestment Calculation
```
reinvestment_percentage = 0.20  // 20% for Tier 1-2, 0% for Tier 3
user_yield = total_yield * (1 - reinvestment_percentage)
reinvested_amount = total_yield * reinvestment_percentage
```

### Example: User with 10 SOL at Tier 2
```
base_yield = 10 * 0.1745 = 1.745 SOL/year
reinvested = 1.745 * 0.20 = 0.349 SOL
user_payout = 1.745 * 0.80 = 1.396 SOL
```

## 4. Self-Perpetuating TVL Growth Model

### Mathematical Formula
```
tvl_t+1 = tvl_t + (tvl_t * apy * reinvestment_percentage) + fees_t
```

### Parameters
- `tvl_t`: Total Value Locked at time t
- `apy`: Annual percentage yield (16%)
- `reinvestment_percentage`: 20% of yields reinvested
- `fees_t`: Fees collected in period t

### Example: 5-Year Projection

#### Initial Conditions
- Starting TVL: 1000 SOL
- Annual fees: 50 SOL (from deposits)
- APY: 16%
- Reinvestment: 20%

#### Year-by-Year Growth
```
Year 0: 1000 SOL
Year 1: 1000 + (1000 * 0.16 * 0.20) + 50 = 1000 + 32 + 50 = 1082 SOL
Year 2: 1082 + (1082 * 0.16 * 0.20) + 50 = 1082 + 34.6 + 50 = 1166.6 SOL
Year 3: 1166.6 + (1166.6 * 0.16 * 0.20) + 50 = 1166.6 + 37.3 + 50 = 1253.9 SOL
Year 4: 1253.9 + (1253.9 * 0.16 * 0.20) + 50 = 1253.9 + 40.1 + 50 = 1344.0 SOL
Year 5: 1344.0 + (1344.0 * 0.16 * 0.20) + 50 = 1344.0 + 43.0 + 50 = 1437.0 SOL
```

#### Growth Analysis
- **Total Growth**: 43.7% over 5 years
- **Compound Growth Rate**: ~7.5% annually
- **Self-Perpetuating Component**: 32-43 SOL/year from reinvestment
- **Fee Component**: 50 SOL/year from deposits

## 5. Early Adopter Bonus

### Bonus Calculation
```
early_adopter_bonus = 1.1  // 10% bonus for first 10 users
effective_apy = base_apy * early_adopter_bonus
```

### Example: Tier 2 Early Adopter
```
base_apy = 17.45%
effective_apy = 17.45% * 1.1 = 19.20%
```

## 6. Loyalty Multiplier Analysis

### Time-Based Loyalty
```
loyalty_multiplier = 1 + (time_staked_years / 5)
effective_weight = base_weight * loyalty_multiplier
```

### Example: 2-Year Stake at Tier 3
```
loyalty_multiplier = 1 + (2 / 5) = 1.4
effective_weight = 2.0 * 1.4 = 2.8
effective_apy = 23.27% * 2.8 = 65.16%
```

## 7. Risk Mitigation Models

### Fee Caps
```
max_fee_percentage = 0.005  // 0.5% maximum
min_effective_apy = base_apy * (1 - max_fee_percentage)
```

### Example: Tier 1 User
```
min_effective_apy = 11.64% * 0.995 = 11.58%
```

### Inactivity Penalties
```
inactivity_threshold = 6  // Score < 6 triggers NFT burn
minimum_stake_time = 2 months  // For small deposits
```

## 8. Economic Equilibrium Analysis

### Supply-Demand Balance
```
total_supply = initial_tvl + cumulative_fees + cumulative_reinvestment
user_demand = function(apy, risk, liquidity_needs)
equilibrium_apy = where(supply = demand)
```

### Sustainability Conditions
1. **Fee Revenue > Operating Costs**
2. **Reinvestment Growth > Inflation**
3. **User Retention > Churn Rate**
4. **TVL Growth > Market Competition**

## 9. Monte Carlo Simulation Parameters

### Input Variables
- Initial TVL: 1000-10000 SOL
- Deposit Rate: 10-100 SOL/day
- APY Volatility: ±2%
- Fee Rate: 0.3-0.7%
- Reinvestment Rate: 15-25%

### Output Metrics
- 95% Confidence Interval for TVL
- Expected APY Range
- Probability of Fund Sustainability
- Optimal Parameter Combinations

## 10. Comparative Analysis

### vs Traditional Staking
```
Traditional: APY = 5-7%, No Lock, No Fees
Jupiter-Kamino: APY = 11-23%, Hard Lock, 0.5% Fee
Net Advantage: 6-16% higher APY for loyalty
```

### vs Other DeFi Protocols
```
Compound: APY = 2-8%, Liquid, No Lock
Aave: APY = 3-12%, Liquid, No Lock
Jupiter-Kamino: APY = 11-23%, Locked, Loyalty Rewards
```

## 11. Conclusion

The Jupiter-Kamino Staking Protocol's mathematical models demonstrate:

1. **Self-Perpetuating Growth**: 7.5% annual compound growth from reinvestment
2. **Loyalty Incentives**: Up to 2.8x multiplier for long-term stakers
3. **Risk Management**: Fee caps and inactivity penalties maintain stability
4. **Economic Sustainability**: Fee revenue covers operational costs
5. **Competitive Advantage**: 6-16% higher APY than traditional staking

The protocol creates a positive feedback loop where:
- Higher TVL → More yield generation
- More yields → More reinvestment
- More reinvestment → Higher TVL
- Higher TVL → Better APY for users

This creates a "perpetual machine" that grows organically while rewarding loyal participants.

