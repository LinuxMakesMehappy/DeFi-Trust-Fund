use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod defi_trust_fund {
    use super::*;

    // Events
    #[event]
    pub struct PoolInitializedEvent {
        pub admin: Pubkey,
        pub pool: Pubkey,
        pub max_apy: u64,
        pub min_commitment_days: u64,
        pub max_commitment_days: u64,
        pub timestamp: i64,
    }

    #[event]
    pub struct StakeEvent {
        pub user: Pubkey,
        pub amount: u64,
        pub committed_days: u64,
        pub timestamp: i64,
    }

    #[event]
    pub struct UnstakeEvent {
        pub user: Pubkey,
        pub amount: u64,
        pub penalty: u64,
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

    // Initialize the pool
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        max_apy: u64,
        min_commitment_days: u64,
        max_commitment_days: u64,
    ) -> Result<()> {
        // Validate parameters
        require!(max_apy > 0 && max_apy <= 10000, ErrorCode::InvalidApy); // Max 100% APY
        require!(min_commitment_days > 0, ErrorCode::InvalidCommitmentDays);
        require!(max_commitment_days >= min_commitment_days, ErrorCode::InvalidCommitmentDays);
        require!(max_commitment_days <= 365, ErrorCode::InvalidCommitmentDays);

        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;

        // Initialize pool state
        pool.admin = ctx.accounts.admin.key();
        pool.max_apy = max_apy;
        pool.min_commitment_days = min_commitment_days;
        pool.max_commitment_days = max_commitment_days;
        pool.min_stake_amount = 100_000_000; // 0.1 SOL minimum
        pool.max_stake_amount = 1_000_000_000_000; // 1000 SOL maximum
        pool.total_staked = 0;
        pool.total_users = 0;
        pool.total_fees_collected = 0;
        pool.deposit_fee_bps = 50; // 0.5% fee
        pool.is_paused = false;
        pool.created_at = clock.unix_timestamp;
        pool.last_update = clock.unix_timestamp;

        emit!(PoolInitializedEvent {
            admin: ctx.accounts.admin.key(),
            pool: ctx.accounts.pool.key(),
            max_apy,
            min_commitment_days,
            max_commitment_days,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    // Stake function
    pub fn stake(ctx: Context<Stake>, amount: u64, committed_days: u64) -> Result<()> {
        // Security checks
        require!(!ctx.accounts.pool.is_paused, ErrorCode::PoolPaused);
        require!(amount >= ctx.accounts.pool.min_stake_amount, ErrorCode::AmountTooSmall);
        require!(amount <= ctx.accounts.pool.max_stake_amount, ErrorCode::AmountTooLarge);
        require!(committed_days >= ctx.accounts.pool.min_commitment_days, ErrorCode::InvalidCommitmentDays);
        require!(committed_days <= ctx.accounts.pool.max_commitment_days, ErrorCode::InvalidCommitmentDays);

        let pool = &mut ctx.accounts.pool;
        let user_stake = &mut ctx.accounts.user_stake;
        let clock = Clock::get()?;

        // Calculate fee
        let fee_amount = amount.checked_mul(pool.deposit_fee_bps).unwrap().checked_div(10000).unwrap();
        let net_amount = amount.checked_sub(fee_amount).unwrap();

        // Transfer SOL from user to pool vault
        let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.pool_vault.key(),
            amount,
        );

        anchor_lang::solana_program::program::invoke(
            &transfer_instruction,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.pool_vault.to_account_info(),
            ],
        )?;

        // Update user stake
        user_stake.user = ctx.accounts.user.key();
        user_stake.amount = net_amount;
        user_stake.committed_days = committed_days;
        user_stake.stake_timestamp = clock.unix_timestamp;
        user_stake.last_claim_timestamp = clock.unix_timestamp;
        user_stake.total_claimed = 0;

        // Update pool state
        pool.total_staked = pool.total_staked.checked_add(net_amount).unwrap();
        pool.total_users = pool.total_users.checked_add(1).unwrap();
        pool.total_fees_collected = pool.total_fees_collected.checked_add(fee_amount).unwrap();
        pool.last_update = clock.unix_timestamp;

        emit!(StakeEvent {
            user: ctx.accounts.user.key(),
            amount: net_amount,
            committed_days,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    // Claim yields
    pub fn claim_yields(ctx: Context<ClaimYields>) -> Result<()> {
        require!(!ctx.accounts.pool.is_paused, ErrorCode::PoolPaused);
        require!(ctx.accounts.user_stake.amount > 0, ErrorCode::NoStake);

        let pool = &mut ctx.accounts.pool;
        let user_stake = &mut ctx.accounts.user_stake;
        let clock = Clock::get()?;

        // Calculate time since last claim
        let time_since_last_claim = clock.unix_timestamp.checked_sub(user_stake.last_claim_timestamp).unwrap();
        require!(time_since_last_claim > 0, ErrorCode::NoYieldToClaim);

        // Calculate yield (simplified calculation)
        let days_staked = time_since_last_claim.checked_div(86400).unwrap(); // Convert seconds to days
        let apy_rate = pool.max_apy.checked_div(10000).unwrap(); // Convert basis points to decimal
        let daily_rate = apy_rate.checked_div(365).unwrap();
        
        let yield_amount = user_stake.amount
            .checked_mul(daily_rate).unwrap()
            .checked_mul(days_staked.try_into().unwrap()).unwrap()
            .checked_div(10000).unwrap();

        require!(yield_amount > 0, ErrorCode::NoYieldToClaim);

        // Check if pool has sufficient funds
        let pool_balance = ctx.accounts.pool_vault.lamports();
        require!(pool_balance >= yield_amount, ErrorCode::InsufficientFunds);

        // Transfer yield to user
        **ctx.accounts.pool_vault.try_borrow_mut_lamports()? -= yield_amount;
        **ctx.accounts.user.try_borrow_mut_lamports()? += yield_amount;

        // Update user stake
        user_stake.last_claim_timestamp = clock.unix_timestamp;
        user_stake.total_claimed = user_stake.total_claimed.checked_add(yield_amount).unwrap();

        // Update pool state
        pool.total_staked = pool.total_staked.checked_sub(yield_amount).unwrap();
        pool.last_update = clock.unix_timestamp;

        Ok(())
    }

    // Unstake function
    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        require!(!ctx.accounts.pool.is_paused, ErrorCode::PoolPaused);
        require!(ctx.accounts.user_stake.amount > 0, ErrorCode::NoStake);

        let pool = &mut ctx.accounts.pool;
        let user_stake = &mut ctx.accounts.user_stake;
        let clock = Clock::get()?;

        // Calculate time staked
        let time_staked = clock.unix_timestamp.checked_sub(user_stake.stake_timestamp).unwrap();
        let days_staked = time_staked.checked_div(86400).unwrap(); // Convert seconds to days

        let unstake_amount = user_stake.amount;
        let mut penalty_amount = 0;

        // Apply penalty for early exit (5% if commitment not met)
        if days_staked < user_stake.committed_days.try_into().unwrap() {
            penalty_amount = unstake_amount.checked_mul(5).unwrap().checked_div(100).unwrap();
        }

        let final_amount = unstake_amount.checked_sub(penalty_amount).unwrap();

        // Transfer funds back to user
        **ctx.accounts.pool_vault.try_borrow_mut_lamports()? -= final_amount;
        **ctx.accounts.user.try_borrow_mut_lamports()? += final_amount;

        // Update pool state
        pool.total_staked = pool.total_staked.checked_sub(unstake_amount).unwrap();
        pool.total_users = pool.total_users.checked_sub(1).unwrap();
        pool.last_update = clock.unix_timestamp;

        // Reset user stake
        user_stake.amount = 0;
        user_stake.committed_days = 0;
        user_stake.stake_timestamp = 0;
        user_stake.last_claim_timestamp = 0;
        user_stake.total_claimed = 0;

        emit!(UnstakeEvent {
            user: ctx.accounts.user.key(),
            amount: final_amount,
            penalty: penalty_amount,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    // Emergency pause (admin only)
    pub fn emergency_pause(ctx: Context<AdminOnly>, reason: String) -> Result<()> {
        require!(ctx.accounts.admin.key() == ctx.accounts.pool.admin, ErrorCode::Unauthorized);
        require!(reason.len() <= 200, ErrorCode::InvalidReason);

        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;

        pool.is_paused = true;
        pool.last_update = clock.unix_timestamp;

        emit!(EmergencyPauseEvent {
            admin: ctx.accounts.admin.key(),
            reason,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    // Emergency unpause (admin only)
    pub fn emergency_unpause(ctx: Context<AdminOnly>) -> Result<()> {
        require!(ctx.accounts.admin.key() == ctx.accounts.pool.admin, ErrorCode::Unauthorized);

        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;

        pool.is_paused = false;
        pool.last_update = clock.unix_timestamp;

        emit!(EmergencyUnpauseEvent {
            admin: ctx.accounts.admin.key(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    // Update APY (admin only)
    pub fn update_apy(ctx: Context<AdminOnly>, new_apy: u64) -> Result<()> {
        require!(ctx.accounts.admin.key() == ctx.accounts.pool.admin, ErrorCode::Unauthorized);
        require!(new_apy > 0 && new_apy <= 10000, ErrorCode::InvalidApy);

        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;
        let old_apy = pool.max_apy;

        pool.max_apy = new_apy;
        pool.last_update = clock.unix_timestamp;

        emit!(ParameterUpdateEvent {
            admin: ctx.accounts.admin.key(),
            parameter: "max_apy".to_string(),
            old_value: old_apy,
            new_value: new_apy,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    // Update deposit fee (admin only)
    pub fn update_deposit_fee(ctx: Context<AdminOnly>, new_fee_bps: u64) -> Result<()> {
        require!(ctx.accounts.admin.key() == ctx.accounts.pool.admin, ErrorCode::Unauthorized);
        require!(new_fee_bps <= 1000, ErrorCode::InvalidFee); // Max 10%

        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;
        let old_fee = pool.deposit_fee_bps;

        pool.deposit_fee_bps = new_fee_bps;
        pool.last_update = clock.unix_timestamp;

        emit!(ParameterUpdateEvent {
            admin: ctx.accounts.admin.key(),
            parameter: "deposit_fee_bps".to_string(),
            old_value: old_fee,
            new_value: new_fee_bps,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    // Update pool limits (admin only)
    pub fn update_pool_limits(
        ctx: Context<AdminOnly>,
        new_min_stake: u64,
        new_max_stake: u64,
    ) -> Result<()> {
        require!(ctx.accounts.admin.key() == ctx.accounts.pool.admin, ErrorCode::Unauthorized);
        require!(new_min_stake > 0, ErrorCode::InvalidAmount);
        require!(new_max_stake > new_min_stake, ErrorCode::InvalidAmount);

        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;

        pool.min_stake_amount = new_min_stake;
        pool.max_stake_amount = new_max_stake;
        pool.last_update = clock.unix_timestamp;

        Ok(())
    }

    // Withdraw fees (admin only)
    pub fn withdraw_fees(ctx: Context<WithdrawFees>, amount: u64) -> Result<()> {
        require!(ctx.accounts.admin.key() == ctx.accounts.pool.admin, ErrorCode::Unauthorized);
        require!(amount > 0, ErrorCode::InvalidAmount);

        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;

        // Check if pool has sufficient fees
        require!(pool.total_fees_collected >= amount, ErrorCode::InsufficientFunds);

        // Transfer fees to admin
        **ctx.accounts.pool_vault.try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.admin.try_borrow_mut_lamports()? += amount;

        pool.total_fees_collected = pool.total_fees_collected.checked_sub(amount).unwrap();
        pool.last_update = clock.unix_timestamp;

        Ok(())
    }
}

// Account contexts
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
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        constraint = !pool.is_paused
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        seeds = [b"pool_vault"],
        bump
    )]
    pub pool_vault: SystemAccount<'info>,
    
    #[account(
        init,
        payer = user,
        space = 8 + UserStake::INIT_SPACE,
        seeds = [b"user_stake", user.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ClaimYields<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        constraint = !pool.is_paused
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        seeds = [b"pool_vault"],
        bump
    )]
    pub pool_vault: SystemAccount<'info>,
    
    #[account(
        mut,
        seeds = [b"user_stake", user.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        constraint = !pool.is_paused
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        seeds = [b"pool_vault"],
        bump
    )]
    pub pool_vault: SystemAccount<'info>,
    
    #[account(
        mut,
        seeds = [b"user_stake", user.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    pub admin: Signer<'info>,
    
    #[account(mut)]
    pub pool: Account<'info, Pool>,
}

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        constraint = admin.key() == pool.admin
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        seeds = [b"pool_vault"],
        bump
    )]
    pub pool_vault: SystemAccount<'info>,
}

// Account structures
#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub admin: Pubkey,
    pub max_apy: u64,
    pub min_commitment_days: u64,
    pub max_commitment_days: u64,
    pub min_stake_amount: u64,
    pub max_stake_amount: u64,
    pub total_staked: u64,
    pub total_users: u64,
    pub total_fees_collected: u64,
    pub deposit_fee_bps: u64,
    pub is_paused: bool,
    pub created_at: i64,
    pub last_update: i64,
}

#[account]
#[derive(InitSpace)]
pub struct UserStake {
    pub user: Pubkey,
    pub amount: u64,
    pub committed_days: u64,
    pub stake_timestamp: i64,
    pub last_claim_timestamp: i64,
    pub total_claimed: u64,
}

// Error codes
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid APY")]
    InvalidApy,
    #[msg("Invalid commitment days")]
    InvalidCommitmentDays,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Invalid fee")]
    InvalidFee,
    #[msg("Invalid reason")]
    InvalidReason,
    #[msg("Amount too small")]
    AmountTooSmall,
    #[msg("Amount too large")]
    AmountTooLarge,
    #[msg("Pool is paused")]
    PoolPaused,
    #[msg("No stake found")]
    NoStake,
    #[msg("No yield to claim")]
    NoYieldToClaim,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Commitment not met")]
    CommitmentNotMet,
    #[msg("Unauthorized")]
    Unauthorized,
}

