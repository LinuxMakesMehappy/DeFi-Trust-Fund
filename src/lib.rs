use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Burn, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata};
use mpl_token_metadata::instruction as mpl_instruction;
use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};
use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// Rate limiting constants
const RATE_LIMIT_WINDOW: i64 = 3600; // 1 hour in seconds
const MAX_CLAIMS_PER_HOUR: u64 = 10;  // Maximum claims per hour
const MAX_STAKES_PER_HOUR: u64 = 5;   // Maximum stakes per hour
const COOLDOWN_PERIOD: i64 = 300;     // 5 minutes cooldown between operations

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

    // ===== REENTRANCY PROTECTION MACROS =====
    
    /// Reentrancy guard - prevents recursive calls
    macro_rules! reentrancy_guard {
        ($pool:expr, $body:block) => {{
            require!(!$pool.reentrancy_guard, ErrorCode::ReentrancyDetected);
            $pool.reentrancy_guard = true;
            let result = $body;
            $pool.reentrancy_guard = false;
            result
        }};
    }

    // ===== RATE LIMITING FUNCTIONS =====
    
    /// Check and update rate limits for claiming
    fn check_claim_rate_limit(user_stake: &mut UserStake, current_time: i64) -> Result<()> {
        // Reset counter if window has passed
        if current_time - user_stake.last_claim_attempt > RATE_LIMIT_WINDOW {
            user_stake.claim_attempts_count = 0;
            user_stake.last_claim_attempt = current_time;
        }
        
        // Check rate limit
        require!(
            user_stake.claim_attempts_count < MAX_CLAIMS_PER_HOUR,
            ErrorCode::RateLimitExceeded
        );
        
        // Check cooldown period
        require!(
            current_time - user_stake.last_claim_attempt >= COOLDOWN_PERIOD,
            ErrorCode::RateLimitExceeded
        );
        
        // Update counters
        user_stake.claim_attempts_count += 1;
        user_stake.last_claim_attempt = current_time;
        
        Ok(())
    }
    
    /// Check and update rate limits for staking
    fn check_stake_rate_limit(user_stake: &mut UserStake, current_time: i64) -> Result<()> {
        // Reset counter if window has passed
        if current_time - user_stake.last_stake_attempt > RATE_LIMIT_WINDOW {
            user_stake.stake_attempts_count = 0;
            user_stake.last_stake_attempt = current_time;
        }
        
        // Check rate limit
        require!(
            user_stake.stake_attempts_count < MAX_STAKES_PER_HOUR,
            ErrorCode::RateLimitExceeded
        );
        
        // Check cooldown period
        require!(
            current_time - user_stake.last_stake_attempt >= COOLDOWN_PERIOD,
            ErrorCode::RateLimitExceeded
        );
        
        // Update counters
        user_stake.stake_attempts_count += 1;
        user_stake.last_stake_attempt = current_time;
        
        Ok(())
    }

    // ===== SLIPPAGE AND MEV PROTECTION FUNCTIONS =====
    
    /// Check slippage tolerance for operations
    fn check_slippage_protection(
        expected_amount: u64,
        actual_amount: u64,
        max_slippage_bps: u64,
    ) -> Result<()> {
        let min_expected = expected_amount
            .checked_mul(10000 - max_slippage_bps)
            .ok_or(ErrorCode::ArithmeticOverflow)?
            .checked_div(10000)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
            
        require!(actual_amount >= min_expected, ErrorCode::SlippageExceeded);
        Ok(())
    }
    
    /// Check transaction deadline to prevent stale transactions
    fn check_transaction_deadline(
        current_timestamp: i64,
        transaction_timestamp: i64,
        deadline_seconds: u64,
    ) -> Result<()> {
        let deadline = transaction_timestamp + deadline_seconds as i64;
        require!(current_timestamp <= deadline, ErrorCode::TransactionExpired);
        Ok(())
    }
    
    /// Check MEV protection for large operations
    fn check_mev_protection(
        pool: &mut Pool,
        current_slot: u64,
        operation_amount: u64,
        large_operation_threshold: u64,
    ) -> Result<()> {
        if operation_amount >= large_operation_threshold {
            let blocks_since_last = current_slot.saturating_sub(pool.last_large_operation_slot);
            require!(blocks_since_last >= pool.min_block_delay, ErrorCode::MevProtectionActive);
            
            pool.last_large_operation_slot = current_slot;
        }
        Ok(())
    }

    // ===== ORACLE VALIDATION FUNCTIONS =====
    
    /// Validate SOL price from Pyth oracle
    fn validate_sol_price(
        price_feed_account: &AccountInfo,
        pool: &Pool,
        current_timestamp: i64,
    ) -> Result<u64> {
        // Load price feed from Pyth
        let price_feed = load_price_feed_from_account_info(price_feed_account)
            .map_err(|_| ErrorCode::InvalidOracle)?;
        
        let price = price_feed.get_current_price()
            .ok_or(ErrorCode::InvalidOracle)?;
        
        // Check price staleness
        let price_age = current_timestamp - price.publish_time;
        require!(
            price_age <= pool.price_staleness_threshold as i64,
            ErrorCode::StalePriceData
        );
        
        // Check for circuit breaker conditions
        let price_value = if price.price >= 0 {
            price.price as u64
        } else {
            return Err(ErrorCode::InvalidOracle.into());
        };
        
        // Additional validation can be added here for price deviation checks
        // against historical data or other oracles
        
        Ok(price_value)
    }
    
    /// Update pool with latest SOL price
    pub fn update_sol_price(ctx: Context<UpdatePrice>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;
        
        // Validate the price feed account matches the stored one
        require!(
            ctx.accounts.price_feed.key() == pool.sol_price_feed,
            ErrorCode::InvalidOracle
        );
        
        // Validate and get current SOL price
        let _sol_price = Self::validate_sol_price(
            &ctx.accounts.price_feed,
            pool,
            clock.unix_timestamp,
        )?;
        
        // Update last price update timestamp
        pool.last_price_update = clock.unix_timestamp;
        pool.updated_at = clock.unix_timestamp;
        
        Ok(())
    }

    // ===== CORE FUNCTIONS =====

    /// Initialize the staking pool with enhanced security and oracle integration
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        max_apy: u64,
        min_commitment_days: u64,
        max_commitment_days: u64,
        sol_price_feed: Pubkey,
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
        
        // Initialize oracle settings
        pool.sol_price_feed = sol_price_feed;
        pool.price_staleness_threshold = 60; // 60 seconds
        pool.max_price_deviation = 1000; // 10% in basis points
        pool.circuit_breaker_threshold = 2000; // 20% in basis points
        pool.last_price_update = 0;
        
        // Initialize multi-signature settings
        pool.multisig_threshold = 1; // Start with single sig, can be updated later
        pool.multisig_signers = vec![ctx.accounts.admin.key()];
        pool.pending_admin_action = None;
        pool.action_timelock_delay = 86400; // 24 hours in seconds
        
        // Initialize reentrancy guard
        pool.reentrancy_guard = false;
        
        // Initialize slippage and MEV protection
        pool.max_slippage_bps = 100; // 1% default slippage
        pool.transaction_deadline = 300; // 5 minutes default deadline
        pool.min_block_delay = 10; // 10 blocks delay for large operations
        pool.last_large_operation_slot = 0;

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

    /// Stake SOL with comprehensive security checks, slippage protection, and MEV resistance
    pub fn stake(
        ctx: Context<Stake>,
        amount: u64,
        committed_days: u64,
        min_expected_amount: u64, // Minimum amount expected after fees (slippage protection)
        deadline: i64,            // Transaction deadline
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
        
        // Check rate limits
        Self::check_stake_rate_limit(user_stake, clock.unix_timestamp)?;
        
        // Check transaction deadline
        Self::check_transaction_deadline(
            clock.unix_timestamp,
            deadline,
            pool.transaction_deadline,
        )?;
        
        // Check MEV protection for large stakes (>10 SOL)
        Self::check_mev_protection(
            pool,
            clock.slot,
            amount,
            10 * LAMPORTS_PER_SOL,
        )?;

        // Calculate fee with overflow protection
        let fee_amount = amount
            .checked_mul(pool.deposit_fee)
            .ok_or(ErrorCode::ArithmeticOverflow)?
            .checked_div(10000)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
        let net_amount = amount.checked_sub(fee_amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
            
        // Check slippage protection - ensure net amount meets minimum expectation
        Self::check_slippage_protection(
            min_expected_amount,
            net_amount,
            pool.max_slippage_bps,
        )?;

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
            
        // Initialize rate limiting fields for new users
        if is_new_user {
            user_stake.last_claim_attempt = 0;
            user_stake.claim_attempts_count = 0;
            user_stake.last_stake_attempt = clock.unix_timestamp;
            user_stake.stake_attempts_count = 1;
        }
        
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

    /// Claim yields with security validations and reentrancy protection
    pub fn claim_yields(ctx: Context<ClaimYields>) -> Result<()> {
        let user_stake = &mut ctx.accounts.user_stake;
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;

        // Reentrancy protection and security checks
        reentrancy_guard!(pool, {
            require!(!pool.is_paused, ErrorCode::PoolPaused);
            require!(pool.is_active, ErrorCode::PoolInactive);
            require!(user_stake.amount > 0, ErrorCode::NoStake);
            
            // Check rate limits
            Self::check_claim_rate_limit(user_stake, clock.unix_timestamp)?;
        
        let current_time = clock.unix_timestamp;
        let time_staked = current_time.checked_sub(user_stake.stake_timestamp)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        let days_staked = time_staked.checked_div(86400)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
        require!(days_staked >= user_stake.committed_days, ErrorCode::CommitmentNotMet);
        
        // Calculate yields with fixed-point arithmetic (avoiding floating-point)
        // Formula: yields = (amount * apy * days_staked) / (365 * 10000)
        // Using checked arithmetic to prevent overflow
        let yields = user_stake.amount
            .checked_mul(pool.apy)
            .ok_or(ErrorCode::ArithmeticOverflow)?
            .checked_mul(days_staked as u64)
            .ok_or(ErrorCode::ArithmeticOverflow)?
            .checked_div(365 * 10000)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
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
        })
    }

    /// Unstake with penalty calculation and reentrancy protection
    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        let user_stake = &mut ctx.accounts.user_stake;
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;

        // Reentrancy protection and security checks
        reentrancy_guard!(pool, {
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
            // Full commitment met - calculate yields using fixed-point arithmetic
            yields = user_stake.amount
                .checked_mul(pool.apy)
                .ok_or(ErrorCode::ArithmeticOverflow)?
                .checked_mul(days_staked as u64)
                .ok_or(ErrorCode::ArithmeticOverflow)?
                .checked_div(365 * 10000)
                .ok_or(ErrorCode::ArithmeticOverflow)?;
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
        })
    }

    // ===== MULTI-SIGNATURE FUNCTIONS =====
    
    /// Propose an admin action (requires multi-sig approval)
    pub fn propose_admin_action(
        ctx: Context<ProposeAction>,
        action_type: ActionType,
        parameters: ActionParameters,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;
        
        // Verify proposer is authorized signer
        require!(
            pool.multisig_signers.contains(&ctx.accounts.proposer.key()),
            ErrorCode::Unauthorized
        );
        
        // Check if there's already a pending action
        require!(pool.pending_admin_action.is_none(), ErrorCode::PendingActionExists);
        
        let executable_at = clock.unix_timestamp + pool.action_timelock_delay as i64;
        
        pool.pending_admin_action = Some(PendingAction {
            action_type,
            proposed_by: ctx.accounts.proposer.key(),
            signatures: vec![ctx.accounts.proposer.key()],
            proposed_at: clock.unix_timestamp,
            executable_at,
            parameters,
        });
        
        pool.updated_at = clock.unix_timestamp;
        
        Ok(())
    }
    
    /// Sign a pending admin action
    pub fn sign_admin_action(ctx: Context<SignAction>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        
        // Verify signer is authorized
        require!(
            pool.multisig_signers.contains(&ctx.accounts.signer.key()),
            ErrorCode::Unauthorized
        );
        
        let pending_action = pool.pending_admin_action.as_mut()
            .ok_or(ErrorCode::NoPendingAction)?;
        
        // Check if signer already signed
        require!(
            !pending_action.signatures.contains(&ctx.accounts.signer.key()),
            ErrorCode::AlreadySigned
        );
        
        pending_action.signatures.push(ctx.accounts.signer.key());
        
        Ok(())
    }
    
    /// Execute a pending admin action (after timelock and sufficient signatures)
    pub fn execute_admin_action(ctx: Context<ExecuteAction>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let clock = Clock::get()?;
        
        let pending_action = pool.pending_admin_action.as_ref()
            .ok_or(ErrorCode::NoPendingAction)?;
        
        // Check timelock
        require!(
            clock.unix_timestamp >= pending_action.executable_at,
            ErrorCode::TimelockNotExpired
        );
        
        // Check sufficient signatures
        require!(
            pending_action.signatures.len() >= pool.multisig_threshold as usize,
            ErrorCode::InsufficientSignatures
        );
        
        // Execute the action based on type
        match pending_action.action_type {
            ActionType::UpdateApy => {
                if let Some(new_apy) = pending_action.parameters.new_apy {
                    require!(new_apy <= pool.max_apy, ErrorCode::InvalidApy);
                    pool.apy = new_apy;
                }
            },
            ActionType::UpdateFee => {
                if let Some(new_fee) = pending_action.parameters.new_fee {
                    require!(new_fee <= 1000, ErrorCode::InvalidFee); // Max 10%
                    pool.deposit_fee = new_fee;
                }
            },
            ActionType::EmergencyPause => {
                pool.is_paused = true;
                if let Some(reason) = &pending_action.parameters.pause_reason {
                    pool.emergency_pause_reason = reason.clone();
                }
            },
            ActionType::EmergencyUnpause => {
                pool.is_paused = false;
                pool.emergency_pause_reason = "".to_string();
            },
            _ => return Err(ErrorCode::InvalidAction.into()),
        }
        
        // Clear pending action
        pool.pending_admin_action = None;
        pool.updated_at = clock.unix_timestamp;
        
        Ok(())
    }
    
    /// Add a new multi-sig signer
    pub fn add_multisig_signer(ctx: Context<ManageMultisig>, new_signer: Pubkey) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        
        require!(pool.multisig_signers.len() < 10, ErrorCode::TooManySigners);
        require!(!pool.multisig_signers.contains(&new_signer), ErrorCode::SignerAlreadyExists);
        
        pool.multisig_signers.push(new_signer);
        
        Ok(())
    }
    
    /// Update multi-sig threshold
    pub fn update_multisig_threshold(ctx: Context<ManageMultisig>, new_threshold: u8) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        
        require!(new_threshold > 0, ErrorCode::InvalidThreshold);
        require!(new_threshold <= pool.multisig_signers.len() as u8, ErrorCode::InvalidThreshold);
        
        pool.multisig_threshold = new_threshold;
        
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
pub struct UpdatePrice<'info> {
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
    
    /// CHECK: This is the Pyth price feed account
    pub price_feed: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ProposeAction<'info> {
    #[account(
        constraint = pool.multisig_signers.contains(&proposer.key()) @ ErrorCode::Unauthorized
    )]
    pub proposer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool"],
        bump
    )]
    pub pool: Account<'info, Pool>,
}

#[derive(Accounts)]
pub struct SignAction<'info> {
    #[account(
        constraint = pool.multisig_signers.contains(&signer.key()) @ ErrorCode::Unauthorized
    )]
    pub signer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool"],
        bump
    )]
    pub pool: Account<'info, Pool>,
}

#[derive(Accounts)]
pub struct ExecuteAction<'info> {
    pub executor: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"pool"],
        bump
    )]
    pub pool: Account<'info, Pool>,
}

#[derive(Accounts)]
pub struct ManageMultisig<'info> {
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
    // Oracle integration fields
    pub sol_price_feed: Pubkey,      // SOL/USD price feed account
    pub price_staleness_threshold: u64, // Maximum price staleness in seconds (default: 60)
    pub max_price_deviation: u64,    // Maximum price deviation percentage (basis points)
    pub circuit_breaker_threshold: u64, // Circuit breaker trigger percentage (basis points)
    pub last_price_update: i64,      // Last price update timestamp
    // Multi-signature and timelock fields
    pub multisig_threshold: u8,      // Required signatures for admin actions
    pub multisig_signers: Vec<Pubkey>, // Authorized signers (max 10)
    pub pending_admin_action: Option<PendingAction>, // Pending admin action
    pub action_timelock_delay: u64,  // Timelock delay in seconds (default: 24 hours)
    // Reentrancy protection
    pub reentrancy_guard: bool,      // Reentrancy guard flag
    // Slippage and MEV protection
    pub max_slippage_bps: u64,       // Maximum slippage in basis points (default: 100 = 1%)
    pub transaction_deadline: u64,   // Transaction deadline in seconds (default: 300 = 5 minutes)
    pub min_block_delay: u64,        // Minimum blocks between large operations (MEV protection)
    pub last_large_operation_slot: u64, // Last slot with large operation
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
    // Rate limiting fields
    pub last_claim_attempt: i64,     // Last claim attempt timestamp
    pub claim_attempts_count: u64,   // Number of claim attempts in current window
    pub last_stake_attempt: i64,     // Last stake attempt timestamp
    pub stake_attempts_count: u64,   // Number of stake attempts in current window
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct PendingAction {
    pub action_type: ActionType,
    pub proposed_by: Pubkey,
    pub signatures: Vec<Pubkey>,
    pub proposed_at: i64,
    pub executable_at: i64,
    pub parameters: ActionParameters,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub enum ActionType {
    UpdateApy,
    UpdateFee,
    EmergencyPause,
    EmergencyUnpause,
    WithdrawFees,
    UpdateLimits,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct ActionParameters {
    pub new_apy: Option<u64>,
    pub new_fee: Option<u64>,
    pub pause_reason: Option<String>,
    pub new_user_limit: Option<u64>,
    pub new_pool_limit: Option<u64>,
    pub withdrawal_amount: Option<u64>,
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
    
    #[msg("Oracle price is stale")]
    StalePriceData,
    
    #[msg("Oracle price deviation too high")]
    PriceDeviationTooHigh,
    
    #[msg("Circuit breaker triggered")]
    CircuitBreakerTriggered,
    
    #[msg("Invalid oracle account")]
    InvalidOracle,
    
    #[msg("Pending action already exists")]
    PendingActionExists,
    
    #[msg("No pending action")]
    NoPendingAction,
    
    #[msg("Already signed")]
    AlreadySigned,
    
    #[msg("Timelock not expired")]
    TimelockNotExpired,
    
    #[msg("Insufficient signatures")]
    InsufficientSignatures,
    
    #[msg("Invalid action")]
    InvalidAction,
    
    #[msg("Too many signers")]
    TooManySigners,
    
    #[msg("Signer already exists")]
    SignerAlreadyExists,
    
    #[msg("Invalid threshold")]
    InvalidThreshold,
    
    #[msg("Reentrancy detected")]
    ReentrancyDetected,
    
    #[msg("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    
    #[msg("Transaction deadline expired")]
    TransactionExpired,
    
    #[msg("MEV protection active - operation too soon")]
    MevProtectionActive,
}

