use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Burn, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata};
use mpl_token_metadata::instruction as mpl_instruction;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod defi_trust_fund {
    use super::*;

    // ===== EVENTS =====
    #[event]
    pub struct PoolInitializedEvent {
        pub admin: Pubkey,
        pub pool: Pubkey,
        pub timestamp: i64,
    }

    #[event]
    pub struct StakeEvent {
        pub user: Pubkey,
        pub amount: u64,
        pub committed_days: u64,
        pub fee_amount: u64,
        pub timestamp: i64,
    }

    #[event]
    pub struct UnstakeEvent {
        pub user: Pubkey,
        pub amount: u64,
        pub yields: u64,
        pub penalty: u64,
        pub timestamp: i64,
    }

    #[event]
    pub struct ClaimEvent {
        pub user: Pubkey,
        pub yields: u64,
        pub timestamp: i64,
    }

    #[event]
    pub struct EmergencyPauseEvent {
        pub admin: Pubkey,
        pub reason: String,
        pub timestamp: i64,
    }

    #[event]
    pub struct EmergencyUnpauseEvent {
        pub admin: Pubkey,
        pub timestamp: i64,
    }

    #[event]
    pub struct ParameterUpdateEvent {
        pub admin: Pubkey,
        pub parameter: String,
        pub old_value: u64,
        pub new_value: u64,
        pub timestamp: i64,
    }

    // ===== CORE FUNCTIONS =====

    /// Initialize the staking pool with enhanced security
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        max_apy: u64,
        min_commitment_days: u64,
        max_commitment_days: u64,
    ) -> Result<()> {
        // Validate input parameters
        require!(max_apy <= 5000, ErrorCode::InvalidApy); // Max 50% APY
        require!(min_commitment_days >= 1, ErrorCode::InvalidCommitment);
        require!(max_commitment_days <= 365, ErrorCode::InvalidCommitment);
        require!(min_commitment_days <= max_commitment_days, ErrorCode::InvalidCommitment);

        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;

        // Initialize pool with security parameters
        pool.admin = ctx.accounts.admin.key();
        pool.total_staked = 0;
        pool.total_users = 0;
        pool.apy = 1200; // 12% APY in basis points
        pool.deposit_fee = 50; // 0.5% fee in basis points
        pool.max_apy = max_apy;
        pool.min_commitment_days = min_commitment_days;
        pool.max_commitment_days = max_commitment_days;
        pool.is_active = true;
        pool.is_paused = false;
        pool.emergency_pause_reason = "".to_string();
        pool.total_fees_collected = 0;
        pool.total_yields_paid = 0;
        pool.last_rebalance_timestamp = clock.unix_timestamp;
        pool.created_at = clock.unix_timestamp;
        pool.updated_at = clock.unix_timestamp;

        // Initialize security parameters
        pool.max_deposit_per_user = 1000 * LAMPORTS_PER_SOL; // 1000 SOL max per user
        pool.max_total_staked = 100000 * LAMPORTS_PER_SOL; // 100k SOL max total
        pool.min_stake_amount = 0.1 * LAMPORTS_PER_SOL; // 0.1 SOL minimum
        pool.max_stake_amount = 100 * LAMPORTS_PER_SOL; // 100 SOL max per stake
        
        emit!(PoolInitializedEvent {
            admin: ctx.accounts.admin.key(),
            pool: pool.key(),
            timestamp: clock.unix_timestamp,
        });
        
        Ok(())
    }

    /// Stake SOL with comprehensive security checks
    pub fn stake(
        ctx: Context<Stake>,
        amount: u64,
        committed_days: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let user_stake = &mut ctx.accounts.user_stake;
        let clock = Clock::get()?;

        // Security checks
        require!(!pool.is_paused, ErrorCode::PoolPaused);
        require!(pool.is_active, ErrorCode::PoolInactive);
        require!(amount >= pool.min_stake_amount, ErrorCode::AmountTooSmall);
        require!(amount <= pool.max_stake_amount, ErrorCode::AmountTooLarge);
        require!(committed_days >= pool.min_commitment_days, ErrorCode::InvalidCommitment);
        require!(committed_days <= pool.max_commitment_days, ErrorCode::InvalidCommitment);
        
        // Check user limits
        let new_total_user_stake = user_stake.amount.checked_add(amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        require!(new_total_user_stake <= pool.max_deposit_per_user, ErrorCode::UserLimitExceeded);
        
        // Check pool limits
        let new_total_pool_stake = pool.total_staked.checked_add(amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        require!(new_total_pool_stake <= pool.max_total_staked, ErrorCode::PoolLimitExceeded);

        // Calculate fee with overflow protection
        let fee_amount = amount
            .checked_mul(pool.deposit_fee)
            .ok_or(ErrorCode::ArithmeticOverflow)?
            .checked_div(10000)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
        let net_amount = amount.checked_sub(fee_amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;

        let is_new_user = user_stake.amount == 0;
        
        // Update user stake
        user_stake.user = ctx.accounts.user.key();
        user_stake.amount = user_stake.amount.checked_add(net_amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        user_stake.committed_days = committed_days;
        user_stake.stake_timestamp = clock.unix_timestamp;
        user_stake.last_claim_timestamp = clock.unix_timestamp;
        user_stake.total_staked_lifetime = user_stake.total_staked_lifetime
            .checked_add(net_amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        user_stake.total_days_staked = user_stake.total_days_staked
            .checked_add(committed_days)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
        // Update pool
        pool.total_staked = pool.total_staked.checked_add(net_amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        pool.total_fees_collected = pool.total_fees_collected
            .checked_add(fee_amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
        if is_new_user {
            pool.total_users = pool.total_users.checked_add(1)
                .ok_or(ErrorCode::ArithmeticOverflow)?;
        }
        
        pool.updated_at = clock.unix_timestamp;

        // Transfer SOL to pool vault
        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.pool_vault.key(),
            amount,
        );
        
        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.pool_vault.clone(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        emit!(StakeEvent {
            user: ctx.accounts.user.key(),
            amount: net_amount,
            committed_days,
            fee_amount,
            timestamp: clock.unix_timestamp,
        });
        
        Ok(())
    }

    /// Claim yields with security validations
    pub fn claim_yields(ctx: Context<ClaimYields>) -> Result<()> {
        let user_stake = &mut ctx.accounts.user_stake;
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;

        // Security checks
        require!(!pool.is_paused, ErrorCode::PoolPaused);
        require!(pool.is_active, ErrorCode::PoolInactive);
        require!(user_stake.amount > 0, ErrorCode::NoStake);
        
        let current_time = clock.unix_timestamp;
        let time_staked = current_time.checked_sub(user_stake.stake_timestamp)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        let days_staked = time_staked.checked_div(86400)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
        require!(days_staked >= user_stake.committed_days, ErrorCode::CommitmentNotMet);
        
        // Calculate yields with overflow protection
        let apy_decimal = pool.apy as f64 / 10000.0;
        let yields = (user_stake.amount as f64 * apy_decimal * (days_staked as f64 / 365.0)) as u64;
        
        require!(yields > 0, ErrorCode::NoYieldsToClaim);
        
        // Check if pool has sufficient funds
        require!(yields <= ctx.accounts.pool_vault.lamports(), ErrorCode::InsufficientFunds);
        
        // Update user stake
        user_stake.last_claim_timestamp = current_time;
        user_stake.total_yields_claimed = user_stake.total_yields_claimed
            .checked_add(yields)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
        // Update pool
        pool.total_yields_paid = pool.total_yields_paid
            .checked_add(yields)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        pool.updated_at = current_time;
        
        // Transfer yields to user
        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.pool_vault.key(),
            &ctx.accounts.user.key(),
            yields,
        );
        
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_ix,
            &[
                ctx.accounts.pool_vault.clone(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[b"pool_vault", &[ctx.bumps.pool_vault]]],
        )?;
        
        emit!(ClaimEvent {
            user: ctx.accounts.user.key(),
            yields,
            timestamp: current_time,
        });
        
        Ok(())
    }

    /// Unstake with penalty calculation
    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        let user_stake = &mut ctx.accounts.user_stake;
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;

        // Security checks
        require!(!pool.is_paused, ErrorCode::PoolPaused);
        require!(user_stake.amount > 0, ErrorCode::NoStake);
        
        let current_time = clock.unix_timestamp;
        let time_staked = current_time.checked_sub(user_stake.stake_timestamp)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        let days_staked = time_staked.checked_div(86400)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
        let mut return_amount = user_stake.amount;
        let mut yields = 0;
        let mut penalty = 0;
        
        // Calculate yields and penalties
        if days_staked >= user_stake.committed_days {
            // Full commitment met - calculate yields
            let apy_decimal = pool.apy as f64 / 10000.0;
            yields = (user_stake.amount as f64 * apy_decimal * (days_staked as f64 / 365.0)) as u64;
            return_amount = return_amount.checked_add(yields)
                .ok_or(ErrorCode::ArithmeticOverflow)?;
        } else {
            // Early exit - apply penalty
            penalty = user_stake.amount.checked_mul(500).ok_or(ErrorCode::ArithmeticOverflow)?
                .checked_div(10000).ok_or(ErrorCode::ArithmeticOverflow)?; // 5% penalty
            return_amount = return_amount.checked_sub(penalty)
                .ok_or(ErrorCode::ArithmeticOverflow)?;
        }
        
        // Check if pool has sufficient funds
        require!(return_amount <= ctx.accounts.pool_vault.lamports(), ErrorCode::InsufficientFunds);
        
        // Update pool
        pool.total_staked = pool.total_staked.checked_sub(user_stake.amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        pool.total_users = pool.total_users.checked_sub(1)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        pool.updated_at = current_time;
        
        // Reset user stake
        user_stake.amount = 0;
        user_stake.committed_days = 0;
        user_stake.stake_timestamp = 0;
        user_stake.last_claim_timestamp = 0;
        
        // Transfer funds to user
        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.pool_vault.key(),
            &ctx.accounts.user.key(),
            return_amount,
        );
        
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_ix,
            &[
                ctx.accounts.pool_vault.clone(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[b"pool_vault", &[ctx.bumps.pool_vault]]],
        )?;
        
        emit!(UnstakeEvent {
            user: ctx.accounts.user.key(),
            amount: user_stake.amount,
            yields,
            penalty,
            timestamp: current_time,
        });
        
        Ok(())
    }

    // ===== ADMIN FUNCTIONS =====

    /// Emergency pause function
    pub fn emergency_pause(ctx: Context<AdminOnly>, reason: String) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;
        
        pool.is_paused = true;
        pool.emergency_pause_reason = reason.clone();
        pool.updated_at = clock.unix_timestamp;
        
        emit!(EmergencyPauseEvent {
            admin: ctx.accounts.admin.key(),
            reason,
            timestamp: clock.unix_timestamp,
        });
        
        Ok(())
    }

    /// Emergency unpause function
    pub fn emergency_unpause(ctx: Context<AdminOnly>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;
        
        pool.is_paused = false;
        pool.emergency_pause_reason = "".to_string();
        pool.updated_at = clock.unix_timestamp;
        
        emit!(EmergencyUnpauseEvent {
            admin: ctx.accounts.admin.key(),
            timestamp: clock.unix_timestamp,
        });
        
        Ok(())
    }

    /// Update APY with bounds checking
    pub fn update_apy(ctx: Context<AdminOnly>, new_apy: u64) -> Result<()> {
        require!(new_apy <= ctx.accounts.pool.max_apy, ErrorCode::InvalidApy);
        require!(new_apy >= 100, ErrorCode::InvalidApy); // Min 1% APY
        
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;
        let old_apy = pool.apy;
        
        pool.apy = new_apy;
        pool.updated_at = clock.unix_timestamp;
        
        emit!(ParameterUpdateEvent {
            admin: ctx.accounts.admin.key(),
            parameter: "apy".to_string(),
            old_value: old_apy,
            new_value: new_apy,
            timestamp: clock.unix_timestamp,
        });
        
        Ok(())
    }

    /// Update deposit fee with bounds checking
    pub fn update_deposit_fee(ctx: Context<AdminOnly>, new_fee: u64) -> Result<()> {
        require!(new_fee <= 1000, ErrorCode::InvalidFee); // Max 10% fee
        require!(new_fee >= 10, ErrorCode::InvalidFee);   // Min 0.1% fee
        
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;
        let old_fee = pool.deposit_fee;
        
        pool.deposit_fee = new_fee;
        pool.updated_at = clock.unix_timestamp;
        
        emit!(ParameterUpdateEvent {
            admin: ctx.accounts.admin.key(),
            parameter: "deposit_fee".to_string(),
            old_value: old_fee,
            new_value: new_fee,
            timestamp: clock.unix_timestamp,
        });
        
        Ok(())
    }

    /// Update pool limits
    pub fn update_pool_limits(
        ctx: Context<AdminOnly>,
        max_deposit_per_user: u64,
        max_total_staked: u64,
        min_stake_amount: u64,
        max_stake_amount: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;
        
        // Validate new limits
        require!(max_deposit_per_user > 0, ErrorCode::InvalidLimit);
        require!(max_total_staked > pool.total_staked, ErrorCode::InvalidLimit);
        require!(min_stake_amount > 0, ErrorCode::InvalidLimit);
        require!(max_stake_amount >= min_stake_amount, ErrorCode::InvalidLimit);
        require!(max_deposit_per_user >= max_stake_amount, ErrorCode::InvalidLimit);
        
        pool.max_deposit_per_user = max_deposit_per_user;
        pool.max_total_staked = max_total_staked;
        pool.min_stake_amount = min_stake_amount;
        pool.max_stake_amount = max_stake_amount;
        pool.updated_at = clock.unix_timestamp;
        
        Ok(())
    }

    /// Withdraw fees (admin only)
    pub fn withdraw_fees(ctx: Context<WithdrawFees>, amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;
        
        require!(amount <= pool.total_fees_collected, ErrorCode::InsufficientFunds);
        require!(amount <= ctx.accounts.pool_vault.lamports(), ErrorCode::InsufficientFunds);
        
        pool.total_fees_collected = pool.total_fees_collected
            .checked_sub(amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        pool.updated_at = clock.unix_timestamp;
        
        // Transfer fees to admin
        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.pool_vault.key(),
            &ctx.accounts.admin.key(),
            amount,
        );
        
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_ix,
            &[
                ctx.accounts.pool_vault.clone(),
                ctx.accounts.admin.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[b"pool_vault", &[ctx.bumps.pool_vault]]],
        )?;
        
        Ok(())
    }
}

// ===== ACCOUNT CONTEXTS =====

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        init,
        payer = admin,
        space = 8 + Pool::INIT_SPACE,
        seeds = [b"pool"],
        bump
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        init,
        payer = admin,
        space = 0,
        seeds = [b"pool_vault"],
        bump
    )]
    pub pool_vault: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool"],
        bump,
        constraint = pool.is_active @ ErrorCode::PoolInactive,
        constraint = !pool.is_paused @ ErrorCode::PoolPaused
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserStake::INIT_SPACE,
        seeds = [b"user_stake", user.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,
    
    #[account(
        mut,
        seeds = [b"pool_vault"],
        bump
    )]
    pub pool_vault: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimYields<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool"],
        bump,
        constraint = pool.is_active @ ErrorCode::PoolInactive,
        constraint = !pool.is_paused @ ErrorCode::PoolPaused
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        seeds = [b"user_stake", user.key().as_ref()],
        bump,
        constraint = user_stake.amount > 0 @ ErrorCode::NoStake
    )]
    pub user_stake: Account<'info, UserStake>,
    
    #[account(
        mut,
        seeds = [b"pool_vault"],
        bump
    )]
    pub pool_vault: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool"],
        bump,
        constraint = !pool.is_paused @ ErrorCode::PoolPaused
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        seeds = [b"user_stake", user.key().as_ref()],
        bump,
        constraint = user_stake.amount > 0 @ ErrorCode::NoStake
    )]
    pub user_stake: Account<'info, UserStake>,
    
    #[account(
        mut,
        seeds = [b"pool_vault"],
        bump
    )]
    pub pool_vault: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    #[account(
        constraint = admin.key() == pool.admin @ ErrorCode::Unauthorized
    )]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool"],
        bump
    )]
    pub pool: Account<'info, Pool>,
}

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(
        constraint = admin.key() == pool.admin @ ErrorCode::Unauthorized
    )]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool"],
        bump
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        seeds = [b"pool_vault"],
        bump
    )]
    pub pool_vault: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

// ===== ACCOUNT STRUCTS =====

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub admin: Pubkey,
    pub total_staked: u64,
    pub total_users: u64,
    pub apy: u64,                    // Basis points (1200 = 12%)
    pub deposit_fee: u64,            // Basis points (50 = 0.5%)
    pub max_apy: u64,                // Maximum allowed APY
    pub min_commitment_days: u64,    // Minimum commitment period
    pub max_commitment_days: u64,    // Maximum commitment period
    pub is_active: bool,             // Pool active status
    pub is_paused: bool,             // Emergency pause status
    pub emergency_pause_reason: String, // Reason for emergency pause
    pub total_fees_collected: u64,   // Total fees collected
    pub total_yields_paid: u64,      // Total yields paid out
    pub last_rebalance_timestamp: i64, // Last rebalance timestamp
    pub max_deposit_per_user: u64,   // Maximum deposit per user
    pub max_total_staked: u64,       // Maximum total staked
    pub min_stake_amount: u64,       // Minimum stake amount
    pub max_stake_amount: u64,       // Maximum stake amount
    pub created_at: i64,             // Creation timestamp
    pub updated_at: i64,             // Last update timestamp
}

#[account]
#[derive(InitSpace)]
pub struct UserStake {
    pub user: Pubkey,
    pub amount: u64,
    pub committed_days: u64,
    pub stake_timestamp: i64,
    pub last_claim_timestamp: i64,
    pub total_staked_lifetime: u64,  // Total amount staked over lifetime
    pub total_days_staked: u64,      // Total days staked over lifetime
    pub total_yields_claimed: u64,   // Total yields claimed over lifetime
}

// ===== ERROR CODES =====

#[error_code]
pub enum ErrorCode {
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    
    #[msg("Amount is too small")]
    AmountTooSmall,
    
    #[msg("Amount is too large")]
    AmountTooLarge,
    
    #[msg("Invalid commitment period")]
    InvalidCommitment,
    
    #[msg("Pool is not active")]
    PoolInactive,
    
    #[msg("Pool is paused")]
    PoolPaused,
    
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    
    #[msg("No yields to claim")]
    NoYieldsToClaim,
    
    #[msg("Commitment period not met")]
    CommitmentNotMet,
    
    #[msg("No stake found")]
    NoStake,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Invalid APY")]
    InvalidApy,
    
    #[msg("Invalid fee")]
    InvalidFee,
    
    #[msg("Invalid limit")]
    InvalidLimit,
    
    #[msg("Insufficient funds")]
    InsufficientFunds,
    
    #[msg("User limit exceeded")]
    UserLimitExceeded,
    
    #[msg("Pool limit exceeded")]
    PoolLimitExceeded,
}

