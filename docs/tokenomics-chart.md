# DeFi Trust Fund - Tokenomics Flow Chart

## Fund Flow Diagram

```mermaid
graph TD
    A[User Deposits] -->|0.5% Fee| B[Protocol Treasury]
    A -->|99.5% Net Amount| C[Staking Pool]
    
    C --> D{Commitment Period}
    D -->|Complete| E[Yield Distribution]
    D -->|Early Exit| F[5% Penalty]
    
    F --> B
    E --> G{User Tier}
    
    G -->|Bronze 11.64% APY| H[70% to User]
    G -->|Silver 17.45% APY| I[70% to User]
    G -->|Gold 23.27% APY| J[70% to User]
    
    H -->|20%| B
    I -->|20%| B
    
    B --> K[Fund Allocation]
    K -->|70%| L[Yield Reserve]
    K -->|20%| M[Development & Operations]
    K -->|10%| N[Emergency Reserve]
    
    L --> O[Reinvested into Pool]
    O --> C
    
    P[Loyalty Multiplier] -->|Up to 2x| E
    Q[Auto-Reinvest Option] --> C
    
    style A fill:#e1f5fe
    style B fill:#f3e5f5
    style C fill:#e8f5e8
    style E fill:#fff3e0
    style F fill:#ffebee
```

## Tier System Flow

```mermaid
graph LR
    A[New User] --> B[Bronze Tier<br/>11.64% APY]
    B --> C{Score Calculation}
    C -->|Top 30 Users| D[Silver Tier<br/>17.45% APY]
    C -->|Top 10 Users| E[Gold Tier<br/>23.27% APY]
    
    F[Score Formula:<br/>5×Deposit + 5×Days<br/>×Loyalty Multiplier] --> C
    
    G[Monthly Rebalancing] --> C
    
    style A fill:#f5f5f5
    style B fill:#cd7f32
    style D fill:#c0c0c0
    style E fill:#ffd700
```

## Loyalty Multiplier System

```mermaid
graph TD
    A[User Stakes] --> B[Track Cumulative Days]
    B --> C{Total Days Staked}
    
    C -->|0-91 Days| D[1.0x - 1.05x Multiplier]
    C -->|92-182 Days| E[1.05x - 1.1x Multiplier]
    C -->|183-274 Days| F[1.1x - 1.15x Multiplier]
    C -->|275-365 Days| G[1.15x - 1.2x Multiplier]
    C -->|365+ Days| H[2.0x Max Multiplier]
    
    I[Formula:<br/>1 + Total Days/365 × 0.2<br/>Max: 2.0x] --> C
    
    style H fill:#ffd700
    style G fill:#ffeb3b
    style F fill:#ffcc02
    style E fill:#ff9800
    style D fill:#ff5722
```

## Revenue & Cost Structure

```mermaid
pie title Revenue Sources
    "Deposit Fees (0.5%)" : 40
    "Early Exit Penalties (5%)" : 35
    "Protocol Reinvestment (20%)" : 25

```

```mermaid
pie title Fund Allocation
    "User Yield Distribution" : 70
    "Development & Operations" : 20
    "Emergency Reserve" : 10
```

## User Journey Flow

```mermaid
sequenceDiagram
    participant U as User
    participant F as Frontend
    participant SC as Smart Contract
    participant O as Oracle
    participant T as Treasury
    
    U->>F: Connect Wallet
    F->>F: Security Validation
    U->>F: Enter Deposit Amount & Days
    F->>F: Input Validation & Slippage Calc
    F->>SC: Submit Deposit Transaction
    SC->>O: Validate Price (if needed)
    SC->>SC: Apply Rate Limiting
    SC->>SC: Calculate Fees & Net Amount
    SC->>T: Transfer Fee (0.5%)
    SC->>SC: Update User Stake
    SC->>SC: Emit Stake Event
    SC->>U: Return Transaction Success
    
    Note over U,T: Commitment Period Active
    
    U->>F: Request Yield Claim
    F->>SC: Submit Claim Transaction
    SC->>SC: Validate Commitment Period
    SC->>SC: Calculate Yields with Fixed-Point Math
    SC->>SC: Apply Loyalty Multiplier
    SC->>U: Transfer Yields
    SC->>SC: Emit Claim Event
```

## Security Architecture

```mermaid
graph TB
    A[User Input] --> B[Frontend Validation]
    B --> C[Smart Contract Validation]
    C --> D[Rate Limiting Check]
    D --> E[Reentrancy Guard]
    E --> F[Oracle Price Validation]
    F --> G[Slippage Protection]
    G --> H[MEV Protection]
    H --> I[Multi-Sig Authorization]
    I --> J[Timelock Delay]
    J --> K[Execute Transaction]
    
    L[Emergency Controls] --> M[Circuit Breakers]
    M --> N[Pause Mechanism]
    N --> O[Admin Override]
    
    style A fill:#e3f2fd
    style K fill:#e8f5e8
    style L fill:#fff3e0
    style O fill:#ffebee
```

## Growth Projection Model

```mermaid
graph LR
    A[Launch: $1M TVL] --> B[Month 3: $5M TVL]
    B --> C[Month 6: $15M TVL]
    C --> D[Month 12: $50M TVL]
    D --> E[Year 2: $100M+ TVL]
    
    F[Network Effects] --> G[More Users]
    G --> H[Higher TVL]
    H --> I[Better Yields]
    I --> F
    
    J[Loyalty Rewards] --> K[User Retention]
    K --> L[Stable TVL]
    L --> M[Predictable Returns]
    M --> J
    
    style A fill:#ffcdd2
    style B fill:#f8bbd9
    style C fill:#e1bee7
    style D fill:#c5cae9
    style E fill:#bbdefb
```

## Risk Management Framework

```mermaid
graph TD
    A[Risk Assessment] --> B{Risk Type}
    
    B -->|Technical| C[Smart Contract Audits]
    B -->|Economic| D[Stress Testing]
    B -->|Operational| E[Multi-Sig Controls]
    B -->|Regulatory| F[Compliance Framework]
    
    C --> G[Continuous Monitoring]
    D --> G
    E --> G
    F --> G
    
    G --> H{Risk Level}
    H -->|Low| I[Normal Operations]
    H -->|Medium| J[Enhanced Monitoring]
    H -->|High| K[Circuit Breakers]
    H -->|Critical| L[Emergency Pause]
    
    style L fill:#ffebee
    style K fill:#fff3e0
    style J fill:#f3e5f5
    style I fill:#e8f5e8
```