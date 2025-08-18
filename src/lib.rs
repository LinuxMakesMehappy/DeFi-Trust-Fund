use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Burn};
use spl_token_metadata::instruction::{create_metadata_accounts_v3, CreateMetadataAccountsV3};
use jupiter_swap_api::{Swap, Quote, JupiterSwapProgram};

declare_id!("YourProgramIdHere1111111111111111111111111111");

#[program]
pub mod defi_trust_fund {
    use super::*;

    #[event]
    pub struct TierRebalanceEvent {
        pub user: Pubkey,
        pub fund_index: u64,
        pub new_tier: u8,
        pub score: u64,
        pub est_apy: f64,
    }

    #[event]
    pub struct ReferralEvent {
        pub referrer: Pubkey,
        pub referred: Pubkey,
        pub fund_index: u64,
    }

    #[derive(Accounts)]
    #[instruction(fund_index: u64)]
    pub struct InitializeFund<'info> {
        #[account(mut)]
        pub admin: Signer<'info>,
        #[account(
            init_if_needed,
            payer = admin,
            space = 8 + 8,
            seeds = [b"fund_manager"],
            bump
        )]
        pub fund_manager: Account<'info, FundManager>,
        #[account(
            init,
            payer = admin,
            space = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8,
            seeds = [b"fund", fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub fund: Account<'info, Fund>,
        #[account(
            init,
            payer = admin,
            mint::decimals = 0,
            mint::authority = admin,
            seeds = [b"stake_nft_mint", fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub stake_nft_mint: Account<'info, Mint>,
        #[account(
            init,
            payer = admin,
            mint::decimals = 0,
            mint::authority = admin,
            seeds = [b"tier_nft_mint", fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub tier_nft_mint: Account<'info, Mint>,
        #[account(
            init,
            payer = admin,
            mint::decimals = 0,
            mint::authority = admin,
            seeds = [b"referrer_nft_mint", fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub referrer_nft_mint: Account<'info, Mint>,
        #[account(mut)]
        pub stake_metadata_account: AccountInfo<'info>,
        #[account(mut)]
        pub tier_metadata_account: AccountInfo<'info>,
        #[account(mut)]
        pub referrer_metadata_account: AccountInfo<'info>,
        #[account(mut)]
        pub yield_reserve: AccountInfo<'info>,
        #[account(
            init,
            payer = admin,
            space = 8 + 4 + (32 + 8) * 100,
            seeds = [b"temp_scores", fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub temp_scores: Account<'info, TempScores>,
        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
        pub token_metadata_program: Program<'info, TokenMetadata>,
        pub rent: Sysvar<'info, Rent>,
    }

    #[derive(Accounts)]
    #[instruction(fund_index: u64)]
    pub struct TriggerRebalance<'info> {
        pub caller: Signer<'info>,
        #[account(
            mut,
            seeds = [b"fund", fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub fund: Account<'info, Fund>,
    }

    #[derive(Accounts)]
    #[instruction(fund_index: u64, user_keys: Vec<Pubkey>)]
    pub struct RebalanceTiersBatch<'info> {
        #[account(mut)]
        pub admin: Signer<'info>,
        #[account(
            mut,
            seeds = [b"fund", fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub fund: Account<'info, Fund>,
        #[account(mut)]
        pub user_stake: Account<'info, UserStake>,
        #[account(mut)]
        pub sentinel_stake_ata: Account<'info, TokenAccount>,
        #[account(mut)]
        pub sentinel_tier_ata: Account<'info, TokenAccount>,
        #[account(mut)]
        pub sentinel_referrer_ata: Account<'info, TokenAccount>,
        #[account(mut)]
        pub stake_nft_mint: Account<'info, Mint>,
        #[account(mut)]
        pub tier_nft_mint: Account<'info, Mint>,
        #[account(mut)]
        pub referrer_nft_mint: Account<'info, Mint>,
        #[account(mut)]
        pub tier_metadata_account: AccountInfo<'info>,
        #[account(mut)]
        pub temp_scores: Account<'info, TempScores>,
        pub token_program: Program<'info, Token>,
        pub token_metadata_program: Program<'info, TokenMetadata>,
        pub system_program: Program<'info, System>,
        pub rent: Sysvar<'info, Rent>,
    }

    #[derive(Accounts)]
    #[instruction(fund_index: u64)]
    pub struct FinalizeRebalance<'info> {
        #[account(mut)]
        pub admin: Signer<'info>,
        #[account(
            mut,
            seeds = [b"fund", fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub fund: Account<'info, Fund>,
        #[account(mut)]
        pub temp_scores: Account<'info, TempScores>,
        #[account(mut)]
        pub tier_nft_mint: Account<'info, Mint>,
        #[account(mut)]
        pub tier_metadata_account: AccountInfo<'info>,
        pub token_metadata_program: Program<'info, TokenMetadata>,
        pub system_program: Program<'info, System>,
        pub rent: Sysvar<'info, Rent>,
    }

    #[derive(Accounts)]
    #[instruction(fund_index: u64)]
    pub struct BurnNFT<'info> {
        #[account(mut)]
        pub user: Signer<'info>,
        #[account(
            mut,
            seeds = [b"fund", fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub fund: Account<'info, Fund>,
        #[account(mut)]
        pub user_stake: Account<'info, UserStake>,
        #[account(mut)]
        pub sentinel_stake_ata: Account<'info, TokenAccount>,
        #[account(mut)]
        pub sentinel_tier_ata: Account<'info, TokenAccount>,
        #[account(mut)]
        pub sentinel_referrer_ata: Account<'info, TokenAccount>,
        #[account(mut)]
        pub stake_nft_mint: Account<'info, Mint>,
        #[account(mut)]
        pub tier_nft_mint: Account<'info, Mint>,
        #[account(mut)]
        pub referrer_nft_mint: Account<'info, Mint>,
        pub token_program: Program<'info, Token>,
    }

    #[derive(Accounts)]
    #[instruction(fund_index: u64, amount: u64, input_mint: Pubkey, referrer: Option<Pubkey>)]
    pub struct Deposit<'info> {
        #[account(mut)]
        pub user: Signer<'info>,
        #[account(
            mut,
            seeds = [b"fund", fund_index.to_le_bytes().as_ref()],
            bump,
            constraint = fund.user_count < 100 @ ErrorCode::FundFull
        )]
        pub fund: Account<'info, Fund>,
        #[account(
            init_if_needed,
            payer = user,
            space = 8 + 8 + 8 + 1 + 32 + 8 + 1,
            seeds = [b"user_stake", user.key().as_ref(), fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub user_stake: Account<'info, UserStake>,
        #[account(mut)]
        pub sentinel_stake_ata: Account<'info, TokenAccount>,
        #[account(mut)]
        pub sentinel_tier_ata: Account<'info, TokenAccount>,
        #[account(mut)]
        pub sentinel_referrer_ata: Account<'info, TokenAccount>,
        #[account(mut)]
        pub stake_nft_mint: Account<'info, Mint>,
        #[account(mut)]
        pub tier_nft_mint: Account<'info, Mint>,
        #[account(mut)]
        pub referrer_nft_mint: Account<'info, Mint>,
        #[account(mut)]
        pub stake_metadata_account: AccountInfo<'info>,
        #[account(mut)]
        pub tier_metadata_account: AccountInfo<'info>,
        #[account(mut)]
        pub referrer_metadata_account: AccountInfo<'info>,
        #[account(mut)]
        pub program_vault: AccountInfo<'info>,
        #[account(mut)]
        pub program_jsol_ata: Account<'info, TokenAccount>,
        #[account(mut)]
        pub input_token_account: AccountInfo<'info, TokenAccount>,
        #[account(mut)]
        pub jupiter_stake_pool: AccountInfo<'info>,
        #[account(mut)]
        pub kamino_vault: AccountInfo<'info>,
        #[account(mut)]
        pub kamino_collateral_ata: AccountInfo<'info, TokenAccount>,
        #[account(mut)]
        pub kamino_debt_ata: AccountInfo<'info, TokenAccount>,
        #[account(mut)]
        pub pyth_jupsol: AccountInfo<'info>,
        #[account(mut)]
        pub yield_reserve: AccountInfo<'info>,
        pub jupiter_swap_program: Program<'info, JupiterSwapProgram>,
        pub kamino_program: Program<'info, KaminoProgram>,
        pub token_program: Program<'info, Token>,
        pub system_program: Program<'info, System>,
        pub token_metadata_program: Program<'info, TokenMetadata>,
        pub rent: Sysvar<'info, Rent>,
    }

    #[derive(Accounts)]
    #[instruction(fund_index: u64)]
    pub struct ClaimYields<'info> {
        #[account(mut)]
        pub user: Signer<'info>,
        #[account(
            mut,
            seeds = [b"fund", fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub fund: Account<'info, Fund>,
        #[account(
            mut,
            seeds = [b"user_stake", user.key().as_ref(), fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub user_stake: Account<'info, UserStake>,
        #[account(
            constraint = sentinel_stake_ata.mint == fund.stake_nft_mint && sentinel_stake_ata.amount >= 1 @ ErrorCode::UnauthorizedWallet
        )]
        pub sentinel_stake_ata: Account<'info, TokenAccount>,
        #[account(
            constraint = sentinel_tier_ata.mint == fund.tier_nft_mint && sentinel_tier_ata.amount >= 1 @ ErrorCode::InvalidTierNFT
        )]
        pub sentinel_tier_ata: Account<'info, TokenAccount>,
        #[account(mut)]
        pub user_destination: AccountInfo<'info>,
        #[account(mut)]
        pub kamino_vault: AccountInfo<'info>,
        #[account(mut)]
        pub kamino_collateral_ata: AccountInfo<'info, TokenAccount>,
        #[account(mut)]
        pub kamino_debt_ata: AccountInfo<'info, TokenAccount>,
        #[account(mut)]
        pub pyth_jupsol: AccountInfo<'info>,
        #[account(mut)]
        pub yield_reserve: AccountInfo<'info>,
        pub kamino_program: Program<'info, KaminoProgram>,
        pub token_program: Program<'info, Token>,
        pub system_program: Program<'info, System>,
    }

    #[derive(Accounts)]
    pub struct FindOpenFund<'info> {
        #[account(seeds = [b"fund_manager"], bump)]
        pub fund_manager: Account<'info, FundManager>,
    }

    #[derive(Accounts)]
    #[instruction(fund_index: u64, auto_reinvest_percentage: u8)]
    pub struct SetAutoReinvest<'info> {
        #[account(mut)]
        pub user: Signer<'info>,
        #[account(
            mut,
            seeds = [b"user_stake", user.key().as_ref(), fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub user_stake: Account<'info, UserStake>,
        #[account(
            constraint = sentinel_stake_ata.mint == fund.stake_nft_mint && sentinel_stake_ata.amount >= 1 @ ErrorCode::UnauthorizedWallet
        )]
        pub sentinel_stake_ata: Account<'info, TokenAccount>,
        #[account(
            seeds = [b"fund", fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub fund: Account<'info, Fund>,
    }

    #[derive(Accounts)]
    #[instruction(fund_index: u64)]
    pub struct DepositYieldReserve<'info> {
        #[account(mut)]
        pub admin: Signer<'info>,
        #[account(
            mut,
            seeds = [b"fund", fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub fund: Account<'info, Fund>,
        #[account(mut)]
        pub yield_reserve: AccountInfo<'info>,
        #[account(mut)]
        pub program_jsol_ata: AccountInfo<'info, TokenAccount>,
        #[account(mut)]
        pub jupiter_stake_pool: AccountInfo<'info>,
        #[account(mut)]
        pub kamino_vault: AccountInfo<'info>,
        #[account(mut)]
        pub kamino_collateral_ata: AccountInfo<'info, TokenAccount>,
        #[account(mut)]
        pub kamino_debt_ata: AccountInfo<'info, TokenAccount>,
        pub jupiter_swap_program: Program<'info, JupiterSwapProgram>,
        pub kamino_program: Program<'info, KaminoProgram>,
        pub token_program: Program<'info, Token>,
        pub system_program: Program<'info, System>,
    }

    #[derive(Accounts)]
    #[instruction(fund_index: u64)]
    pub struct GetUserInfo<'info> {
        pub user: Signer<'info>,
        #[account(
            seeds = [b"fund", fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub fund: Account<'info, Fund>,
        #[account(
            seeds = [b"user_stake", user.key().as_ref(), fund_index.to_le_bytes().as_ref()],
            bump
        )]
        pub user_stake: Account<'info, UserStake>,
        #[account(
            constraint = sentinel_stake_ata.mint == fund.stake_nft_mint
        )]
        pub sentinel_stake_ata: Account<'info, TokenAccount>,
        #[account(
            constraint = sentinel_tier_ata.mint == fund.tier_nft_mint
        )]
        pub sentinel_tier_ata: Account<'info, TokenAccount>,
    }

    pub fn initialize_fund(ctx: Context<InitializeFund>, fund_index: u64) -> Result<()> {
        if fund_index == 0 {
            ctx.accounts.fund_manager.fund_count = 1;
        } else {
            let fund_manager = &mut ctx.accounts.fund_manager;
            fund_manager.fund_count = fund_manager.fund_count.checked_add(1).ok_or(ErrorCode::ArithmeticOverflow)?;
        }
        let fund = &mut ctx.accounts.fund;
        fund.stake_nft_mint = ctx.accounts.stake_nft_mint.key();
        fund.tier_nft_mint = ctx.accounts.tier_nft_mint.key();
        fund.referrer_nft_mint = ctx.accounts.referrer_nft_mint.key();
        fund.total_deposits = 0;
        fund.user_count = 0;
        fund.early_adopters = 0;
        fund.last_rebalance_timestamp = Clock::get()?.unix_timestamp as u64;

        // Create metadata for Stake NFT
        let cpi_program = ctx.accounts.token_metadata_program.to_account_info();
        let cpi_accounts = CreateMetadataAccountsV3 {
            metadata: ctx.accounts.stake_metadata_account.to_account_info(),
            mint: ctx.accounts.stake_nft_mint.to_account_info(),
            mint_authority: ctx.accounts.admin.to_account_info(),
            payer: ctx.accounts.admin.to_account_info(),
            update_authority: ctx.accounts.admin.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        create_metadata_accounts_v3(
            cpi_ctx,
            format!("Fund {} Stake Pass", fund_index),
            format!("F{}SP", fund_index),
            "Soulbound stake pass".to_string(),
            true,
            None,
            Some(spl_token_metadata::state::TokenStandard::NonFungible),
        )?;

        // Create metadata for Tier NFT (default Tier 1)
        let cpi_accounts = CreateMetadataAccountsV3 {
            metadata: ctx.accounts.tier_metadata_account.to_account_info(),
            mint: ctx.accounts.tier_nft_mint.to_account_info(),
            mint_authority: ctx.accounts.admin.to_account_info(),
            payer: ctx.accounts.admin.to_account_info(),
            update_authority: ctx.accounts.admin.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        create_metadata_accounts_v3(
            cpi_ctx,
            format!("Fund {} Tier 1 Badge", fund_index),
            format!("F{}T1", fund_index),
            "Soulbound tier 1 badge".to_string(),
            true,
            None,
            Some(spl_token_metadata::state::TokenStandard::NonFungible),
        )?;

        // Create metadata for Referrer NFT
        let cpi_accounts = CreateMetadataAccountsV3 {
            metadata: ctx.accounts.referrer_metadata_account.to_account_info(),
            mint: ctx.accounts.referrer_nft_mint.to_account_info(),
            mint_authority: ctx.accounts.admin.to_account_info(),
            payer: ctx.accounts.admin.to_account_info(),
            update_authority: ctx.accounts.admin.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        create_metadata_accounts_v3(
            cpi_ctx,
            format!("Fund {} Referrer Badge", fund_index),
            format!("F{}REF", fund_index),
            "Referral program badge".to_string(),
            true,
            None,
            Some(spl_token_metadata::state::TokenStandard::NonFungible),
        )?;

        Ok(())
    }

    pub fn trigger_rebalance(ctx: Context<TriggerRebalance>, fund_index: u64) -> Result<()> {
        let fund = &mut ctx.accounts.fund;
        let current_time = Clock::get()?.unix_timestamp as u64;
        let month_seconds = 30 * 24 * 3600;
        require!(current_time >= fund.last_rebalance_timestamp.checked_add(month_seconds).ok_or(ErrorCode::ArithmeticOverflow)?, ErrorCode::RebalanceNotDue);
        fund.last_rebalance_timestamp = current_time;
        Ok(())
    }

    pub fn rebalance_tiers_batch(ctx: Context<RebalanceTiersBatch>, fund_index: u64, user_keys: Vec<Pubkey>) -> Result<()> {
        require!(user_keys.len() <= 25, ErrorCode::InvalidBatchSize);
        let fund = &mut ctx.accounts.fund;
        let current_time = Clock::get()?.unix_timestamp as u64;
        let day_seconds = 24 * 3600;

        for (i, user_key) in user_keys.iter().enumerate() {
            let user_stake = &mut ctx.accounts.user_stake;
            let days_staked = (current_time - user_stake.stake_timestamp) / day_seconds;

            // Loyalty multiplier: 1.0 + (days_staked / 365.0) * 0.2 (max 2x after 5 years)
            let years_staked = days_staked as f64 / 365.0;
            let loyalty_multiplier = 1.0 + (years_staked * 0.2).min(1.0); // Cap at 2x

            // Base score: 5 * deposit_amount + 5 * days_staked
            let base_score = 5 * user_stake.deposit_amount + 5 * days_staked;
            
            // Apply loyalty multiplier
            let score = (base_score as f64 * loyalty_multiplier) as u64;

            // Store score in TempScores
            let temp_scores = &mut ctx.accounts.temp_scores;
            temp_scores.scores[i] = TempScore {
                user: *user_key,
                score,
            };

            // Burn for inactivity (score < 6)
            if score < 6 {
                let cpi_accounts = Burn {
                    mint: ctx.accounts.stake_nft_mint.to_account_info(),
                    from: ctx.accounts.sentinel_stake_ata.to_account_info(),
                    authority: ctx.accounts.sentinel_stake_ata.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                token::burn(CpiContext::new(cpi_program, cpi_accounts), 1)?;

                let cpi_accounts = Burn {
                    mint: ctx.accounts.tier_nft_mint.to_account_info(),
                    from: ctx.accounts.sentinel_tier_ata.to_account_info(),
                    authority: ctx.accounts.sentinel_tier_ata.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                token::burn(CpiContext::new(cpi_program, cpi_accounts), 1)?;

                user_stake.deposit_amount = 0;
                user_stake.tier = 0;
                fund.user_count = fund.user_count.checked_sub(1).ok_or(ErrorCode::ArithmeticOverflow)?;
                continue;
            }
        }

        Ok(())
    }

    pub fn finalize_rebalance(ctx: Context<FinalizeRebalance>, fund_index: u64) -> Result<()> {
        let fund = &mut ctx.accounts.fund;
        let temp_scores = &mut ctx.accounts.temp_scores;

        // Simple bubble sort for scores (100 users, ~200K compute units)
        for i in 0..temp_scores.scores.len() {
            for j in 0..temp_scores.scores.len() - i - 1 {
                if temp_scores.scores[j].score < temp_scores.scores[j + 1].score {
                    let temp = temp_scores.scores[j];
                    temp_scores.scores[j] = temp_scores.scores[j + 1];
                    temp_scores.scores[j + 1] = temp;
                }
            }
        }

        // Assign tiers: top 10 -> Tier 3, next 20 -> Tier 2, rest -> Tier 1
        for (i, temp_score) in temp_scores.scores.iter().enumerate() {
            let tier = if i < 10 { 3 } else if i < 30 { 2 } else { 1 };
            
            // Update Tier NFT metadata for the new tier
            let cpi_program = ctx.accounts.token_metadata_program.to_account_info();
            let cpi_accounts = CreateMetadataAccountsV3 {
                metadata: ctx.accounts.tier_metadata_account.to_account_info(),
                mint: ctx.accounts.tier_nft_mint.to_account_info(),
                mint_authority: ctx.accounts.admin.to_account_info(),
                payer: ctx.accounts.admin.to_account_info(),
                update_authority: ctx.accounts.admin.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            create_metadata_accounts_v3(
                cpi_ctx,
                format!("Fund {} Tier {} Badge", fund_index, tier),
                format!("F{}T{}", fund_index, tier),
                format!("Soulbound tier {} badge - Score: {}, Est APY: {}%", tier, temp_score.score, get_est_apy(tier)).to_string(),
                true,
                None,
                Some(spl_token_metadata::state::TokenStandard::NonFungible),
            )?;

            // Emit event
            emit!(TierRebalanceEvent {
                user: temp_score.user,
                fund_index,
                new_tier: tier,
                score: temp_score.score,
                est_apy: get_est_apy(tier),
            });
        }

        // Clear temp_scores
        temp_scores.scores.clear();

        Ok(())
    }

    pub fn burn_nft(ctx: Context<BurnNFT>, fund_index: u64) -> Result<()> {
        let fund = &ctx.accounts.fund;
        let user_stake = &mut ctx.accounts.user_stake;

        // Burn Stake NFT
        if ctx.accounts.sentinel_stake_ata.amount >= 1 && ctx.accounts.sentinel_stake_ata.mint == fund.stake_nft_mint {
            let cpi_accounts = Burn {
                mint: ctx.accounts.stake_nft_mint.to_account_info(),
                from: ctx.accounts.sentinel_stake_ata.to_account_info(),
                authority: ctx.accounts.sentinel_stake_ata.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            token::burn(CpiContext::new(cpi_program, cpi_accounts), 1)?;
            user_stake.deposit_amount = 0;
            fund.user_count = fund.user_count.checked_sub(1).ok_or(ErrorCode::ArithmeticOverflow)?;
        }

        // Burn Tier NFT
        if ctx.accounts.sentinel_tier_ata.amount >= 1 && ctx.accounts.sentinel_tier_ata.mint == fund.tier_nft_mint {
            let cpi_accounts = Burn {
                mint: ctx.accounts.tier_nft_mint.to_account_info(),
                from: ctx.accounts.sentinel_tier_ata.to_account_info(),
                authority: ctx.accounts.sentinel_tier_ata.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            token::burn(CpiContext::new(cpi_program, cpi_accounts), 1)?;
            user_stake.tier = 0;
        }

        // Burn Referrer NFT
        if ctx.accounts.sentinel_referrer_ata.amount >= 1 && ctx.accounts.sentinel_referrer_ata.mint == fund.referrer_nft_mint {
            let cpi_accounts = Burn {
                mint: ctx.accounts.referrer_nft_mint.to_account_info(),
                from: ctx.accounts.sentinel_referrer_ata.to_account_info(),
                authority: ctx.accounts.sentinel_referrer_ata.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            token::burn(CpiContext::new(cpi_program, cpi_accounts), 1)?;
            user_stake.referrer = Pubkey::default();
            user_stake.referral_expiry = 0;
        }

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, fund_index: u64, amount: u64, input_mint: Pubkey, referrer: Option<Pubkey>) -> Result<()> {
        let fund = &mut ctx.accounts.fund;
        let is_first_deposit = ctx.accounts.user_stake.deposit_amount == 0;

        if is_first_deposit {
            require_lt!(fund.user_count, 100, ErrorCode::FundFull);
            fund.user_count = fund.user_count.checked_add(1).ok_or(ErrorCode::ArithmeticOverflow)?;
            if fund.early_adopters < 10 {
                fund.early_adopters = fund.early_adopters.checked_add(1).ok_or(ErrorCode::ArithmeticOverflow)?;
            }
            
            // Set referrer if provided
            if let Some(referrer_key) = referrer {
                ctx.accounts.user_stake.referrer = referrer_key;
                let current_time = Clock::get()?.unix_timestamp as u64;
                ctx.accounts.user_stake.referral_expiry = current_time + 90 * 24 * 3600; // 3 months
                
                // Mint referrer badge to sentinel ATA (soulbound)
                let cpi_accounts = MintTo {
                    mint: ctx.accounts.referrer_nft_mint.to_account_info(),
                    to: ctx.accounts.sentinel_referrer_ata.to_account_info(),
                    authority: ctx.accounts.admin.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                token::mint_to(CpiContext::new(cpi_program, cpi_accounts), 1)?;
                
                // Emit referral event
                emit!(ReferralEvent {
                    referrer: referrer_key,
                    referred: ctx.accounts.user.key(),
                    fund_index,
                });
            }
        } else {
            require!(ctx.accounts.sentinel_stake_ata.amount >= 1 && ctx.accounts.sentinel_stake_ata.mint == fund.stake_nft_mint, ErrorCode::UnauthorizedWallet);
            require!(ctx.accounts.sentinel_tier_ata.amount >= 1 && ctx.accounts.sentinel_tier_ata.mint == fund.tier_nft_mint, ErrorCode::InvalidTierNFT);
        }

        require_gt!(amount, 0, ErrorCode::ZeroAmount);

        // Calculate fee (0.5%)
        let fee_percentage = 0.005;
        let fee_amount = (amount as f64 * fee_percentage) as u64;
        let deposit_amount = amount.checked_sub(fee_amount).ok_or(ErrorCode::ArithmeticOverflow)?;

        // Swap to SOL
        let is_sol = input_mint == Pubkey::default();
        let sol_amount = if !is_sol {
            let quote = jupiter_swap_api::get_quote(
                input_mint,
                Pubkey::default(),
                deposit_amount,
                0.01,
            )?;
            require!(quote.output_amount >= deposit_amount.checked_div(100).ok_or(ErrorCode::ArithmeticOverflow)?, ErrorCode::HighSlippage);
            let swap_ix = jupiter_swap_api::swap(
                &ctx.accounts.jupiter_swap_program.key(),
                &ctx.accounts.input_token_account.key(),
                &ctx.accounts.program_vault.key(),
                &quote.route,
                deposit_amount,
                &ctx.accounts.user.key(),
                &ctx.accounts.token_program.key(),
            )?;
            invoke(
                &swap_ix,
                &[
                    ctx.accounts.input_token_account.to_account_info(),
                    ctx.accounts.program_vault.clone(),
                    ctx.accounts.jupiter_swap_program.to_account_info(),
                    ctx.accounts.token_program.to_account_info(),
                    ctx.accounts.user.to_account_info(),
                ],
            )?;
            quote.output_amount
        } else {
            let transfer_ix = system_instruction::transfer(
                &ctx.accounts.user.key(),
                &ctx.accounts.program_vault.key(),
                deposit_amount,
            );
            invoke(
                &transfer_ix,
                &[
                    ctx.accounts.user.to_account_info(),
                    ctx.accounts.program_vault.clone(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
            deposit_amount
        };

        // Transfer fee to yield_reserve
        if fee_amount > 0 {
            let fee_transfer_ix = system_instruction::transfer(
                &ctx.accounts.user.key(),
                &ctx.accounts.yield_reserve.key(),
                fee_amount,
            );
            invoke(
                &fee_transfer_ix,
                &[
                    ctx.accounts.user.to_account_info(),
                    ctx.accounts.yield_reserve.clone(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }

        // Stake to JupSOL
        let stake_ix = jupiter_swap_api::stake_sol_to_jsol(
            &ctx.accounts.jupiter_stake_pool.key(),
            &ctx.accounts.program_vault.key(),
            &ctx.accounts.program_jsol_ata.key(),
            sol_amount,
            &ctx.accounts.token_program.key(),
        )?;
        invoke(
            &stake_ix,
            &[
                ctx.accounts.jupiter_stake_pool.clone(),
                ctx.accounts.program_vault.clone(),
                ctx.accounts.program_jsol_ata.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
        )?;

        // Deposit to Kamino
        let kamino_deposit_ix = kamino::deposit_to_multiply_vault(
            &ctx.accounts.kamino_program.key(),
            &ctx.accounts.program_jsol_ata.key(),
            &ctx.accounts.kamino_vault.key(),
            &ctx.accounts.kamino_collateral_ata.key(),
            &ctx.accounts.kamino_debt_ata.key(),
            sol_amount,
            4.6,
            &ctx.accounts.token_program.key(),
        )?;
        invoke(
            &kamino_deposit_ix,
            &[
                ctx.accounts.program_jsol_ata.to_account_info(),
                ctx.accounts.kamino_vault.clone(),
                ctx.accounts.kamino_collateral_ata.to_account_info(),
                ctx.accounts.kamino_debt_ata.to_account_info(),
                ctx.accounts.kamino_program.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
        )?;

        // Mint NFTs and deposit to sentinel ATA
        if is_first_deposit {
            let cpi_accounts = MintTo {
                mint: ctx.accounts.stake_nft_mint.to_account_info(),
                to: ctx.accounts.sentinel_stake_ata.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            token::mint_to(CpiContext::new(cpi_program, cpi_accounts), 1)?;

            let cpi_accounts = MintTo {
                mint: ctx.accounts.tier_nft_mint.to_account_info(),
                to: ctx.accounts.sentinel_tier_ata.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            token::mint_to(CpiContext::new(cpi_program, cpi_accounts), 1)?;
        }

        // Record stake
        let user_stake = &mut ctx.accounts.user_stake;
        user_stake.deposit_amount = user_stake.deposit_amount.checked_add(sol_amount).ok_or(ErrorCode::ArithmeticOverflow)?;
        if user_stake.stake_timestamp == 0 {
            user_stake.stake_timestamp = Clock::get()?.unix_timestamp as u64;
            user_stake.tier = 1;
            user_stake.auto_reinvest_percentage = 20; // Default 20%
        }
        let fund = &mut ctx.accounts.fund;
        fund.total_deposits = fund.total_deposits.checked_add(sol_amount).ok_or(ErrorCode::ArithmeticOverflow)?;

        // Gas rebate for first deposit
        if is_first_deposit {
            let rebate_amount = 5000; // ~0.000005 SOL
            let transfer_ix = system_instruction::transfer(
                &ctx.accounts.yield_reserve.key(),
                &ctx.accounts.user.key(),
                rebate_amount,
            );
            invoke(
                &transfer_ix,
                &[
                    ctx.accounts.yield_reserve.clone(),
                    ctx.accounts.user.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }

        // Reinvest fee
        if fee_amount > 0 {
            let fee_stake_ix = jupiter_swap_api::stake_sol_to_jsol(
                &ctx.accounts.jupiter_stake_pool.key(),
                &ctx.accounts.yield_reserve.key(),
                &ctx.accounts.program_jsol_ata.key(),
                fee_amount,
                &ctx.accounts.token_program.key(),
            )?;
            invoke(
                &fee_stake_ix,
                &[
                    ctx.accounts.jupiter_stake_pool.clone(),
                    ctx.accounts.yield_reserve.clone(),
                    &ctx.accounts.program_jsol_ata.to_account_info(),
                    ctx.accounts.token_program.to_account_info(),
                ],
            )?;

            let fee_deposit_ix = kamino::deposit_to_multiply_vault(
                &ctx.accounts.kamino_program.key(),
                &ctx.accounts.program_jsol_ata.key(),
                &ctx.accounts.kamino_vault.key(),
                &ctx.accounts.kamino_collateral_ata.key(),
                &ctx.accounts.kamino_debt_ata.key(),
                fee_amount,
                4.6,
                &ctx.accounts.token_program.key(),
            )?;
            invoke(
                &fee_deposit_ix,
                &[
                    ctx.accounts.program_jsol_ata.to_account_info(),
                    ctx.accounts.kamino_vault.clone(),
                    ctx.accounts.kamino_collateral_ata.to_account_info(),
                    ctx.accounts.kamino_debt_ata.to_account_info(),
                    ctx.accounts.kamino_program.to_account_info(),
                    ctx.accounts.token_program.to_account_info(),
                ],
            )?;

            fund.total_deposits = fund.total_deposits.checked_add(fee_amount).ok_or(ErrorCode::ArithmeticOverflow)?;
        }

        Ok(())
    }

    pub fn claim_yields(ctx: Context<ClaimYields>, fund_index: u64) -> Result<()> {
        require!(ctx.accounts.sentinel_stake_ata.amount >= 1 && ctx.accounts.sentinel_stake_ata.mint == ctx.accounts.fund.stake_nft_mint, ErrorCode::UnauthorizedWallet);
        require!(ctx.accounts.sentinel_tier_ata.amount >= 1 && ctx.accounts.sentinel_tier_ata.mint == ctx.accounts.fund.tier_nft_mint, ErrorCode::InvalidTierNFT);
        let user_stake = &ctx.accounts.user_stake;
        require_gt!(user_stake.deposit_amount, 0, ErrorCode::NoDeposit);

        let current_time = Clock::get()?.unix_timestamp as u64;
        let has_active_referral = user_stake.referrer != Pubkey::default() && current_time < user_stake.referral_expiry;
        
        let base_weight = match user_stake.tier {
            1 => 1.0,
            2 => 1.5,
            3 => 2.0,
            _ => return err!(ErrorCode::InvalidTier),
        };
        
        let referral_multiplier = if has_active_referral { 1.1 } else { 1.0 };
        let early_adopter_multiplier = if ctx.accounts.fund.early_adopters < 10 && user_stake.deposit_amount > 0 { 1.1 } else { 1.0 };
        
        let weight = base_weight * referral_multiplier * early_adopter_multiplier;

        let protocol_reinvestment = if user_stake.tier == 3 { 0.0 } else { 0.2 };
        let user_reinvestment = user_stake.auto_reinvest_percentage as f64 / 100.0;
        let total_reinvestment = (protocol_reinvestment + user_reinvestment).min(1.0);

        let jupsol_price = pyth::get_price(&ctx.accounts.pyth_jupsol.key())?;
        let current_pool_value = kamino::get_vault_value(&ctx.accounts.kamino_vault.key())?;
        require!(current_pool_value >= ctx.accounts.fund.total_deposits.checked_mul(jupsol_price).ok_or(ErrorCode::ArithmeticOverflow)?, ErrorCode::InvalidVaultValue);
        let total_yields = current_pool_value.saturating_sub(ctx.accounts.fund.total_deposits);
        let total_weighted_deposits = ctx.accounts.fund.total_deposits as f64;
        let user_weighted_share = (user_stake.deposit_amount as f64 * weight) / total_weighted_deposits;
        let user_yield = (user_weighted_share * total_yields as f64) as u64;

        if user_yield > 0 {
            let reinvest_amount = (user_yield as f64 * total_reinvestment) as u64;
            let user_payout = user_yield.checked_sub(reinvest_amount).ok_or(ErrorCode::ArithmeticOverflow)?;

            // Payout to user
            if user_payout > 0 {
                let unwind_ix = kamino::reduce_position(
                    &ctx.accounts.kamino_program.key(),
                    &ctx.accounts.kamino_vault.key(),
                    &ctx.accounts.kamino_collateral_ata.key(),
                    &ctx.accounts.kamino_debt_ata.key(),
                    user_payout,
                    &ctx.accounts.user_destination.key(),
                    &ctx.accounts.token_program.key(),
                )?;
                invoke(
                    &unwind_ix,
                    &[
                        ctx.accounts.kamino_vault.clone(),
                        ctx.accounts.kamino_collateral_ata.to_account_info(),
                        ctx.accounts.kamino_debt_ata.to_account_info(),
                        ctx.accounts.user_destination.clone(),
                        ctx.accounts.kamino_program.to_account_info(),
                        ctx.accounts.token_program.to_account_info(),
                    ],
                )?;
            }

            // Reinvest to yield_reserve
            if reinvest_amount > 0 {
                let reinvest_unwind_ix = kamino::reduce_position(
                    &ctx.accounts.kamino_program.key(),
                    &ctx.accounts.kamino_vault.key(),
                    &ctx.accounts.kamino_collateral_ata.key(),
                    &ctx.accounts.kamino_debt_ata.key(),
                    reinvest_amount,
                    &ctx.accounts.yield_reserve.key(),
                    &ctx.accounts.token_program.key(),
                )?;
                invoke(
                    &reinvest_unwind_ix,
                    &[
                        ctx.accounts.kamino_vault.clone(),
                        ctx.accounts.kamino_collateral_ata.to_account_info(),
                        ctx.accounts.kamino_debt_ata.to_account_info(),
                        ctx.accounts.yield_reserve.clone(),
                        ctx.accounts.kamino_program.to_account_info(),
                        ctx.accounts.token_program.to_account_info(),
                    ],
                )?;

                let reinvest_stake_ix = jupiter_swap_api::stake_sol_to_jsol(
                    &ctx.accounts.jupiter_stake_pool.key(),
                    &ctx.accounts.yield_reserve.key(),
                    &ctx.accounts.program_jsol_ata.key(),
                    reinvest_amount,
                    &ctx.accounts.token_program.key(),
                )?;
                invoke(
                    &reinvest_stake_ix,
                    &[
                        ctx.accounts.jupiter_stake_pool.clone(),
                        ctx.accounts.yield_reserve.clone(),
                        &ctx.accounts.program_jsol_ata.to_account_info(),
                        ctx.accounts.token_program.to_account_info(),
                    ],
                )?;

                let reinvest_deposit_ix = kamino::deposit_to_multiply_vault(
                    &ctx.accounts.kamino_program.key(),
                    &ctx.accounts.program_jsol_ata.key(),
                    &ctx.accounts.kamino_vault.key(),
                    &ctx.accounts.kamino_collateral_ata.key(),
                    &ctx.accounts.kamino_debt_ata.key(),
                    reinvest_amount,
                    4.6,
                    &ctx.accounts.token_program.key(),
                )?;
                invoke(
                    &reinvest_deposit_ix,
                    &[
                        ctx.accounts.program_jsol_ata.to_account_info(),
                        ctx.accounts.kamino_vault.clone(),
                        ctx.accounts.kamino_collateral_ata.to_account_info(),
                        ctx.accounts.kamino_debt_ata.to_account_info(),
                        ctx.accounts.kamino_program.to_account_info(),
                        ctx.accounts.token_program.to_account_info(),
                    ],
                )?;

                let fund = &mut ctx.accounts.fund;
                fund.total_deposits = fund.total_deposits.checked_add(reinvest_amount).ok_or(ErrorCode::ArithmeticOverflow)?;
            }
        }

        Ok(())
    }

    pub fn set_auto_reinvest(ctx: Context<SetAutoReinvest>, fund_index: u64, auto_reinvest_percentage: u8) -> Result<()> {
        require!(auto_reinvest_percentage <= 100, ErrorCode::InvalidPercentage);
        ctx.accounts.user_stake.auto_reinvest_percentage = auto_reinvest_percentage;
        Ok(())
    }

    pub fn deposit_yield_reserve(ctx: Context<DepositYieldReserve>, fund_index: u64) -> Result<()> {
        let fund = &mut ctx.accounts.fund;
        
        // Get actual yield reserve balance from account data
        let yield_reserve_data = ctx.accounts.yield_reserve.try_borrow_data()?;
        let yield_balance = u64::from_le_bytes(yield_reserve_data[0..8].try_into().unwrap());
        
        if yield_balance > 0 {
            // Stake to JupSOL
            let stake_ix = jupiter_swap_api::stake_sol_to_jsol(
                &ctx.accounts.jupiter_stake_pool.key(),
                &ctx.accounts.yield_reserve.key(),
                &ctx.accounts.program_jsol_ata.key(),
                yield_balance,
                &ctx.accounts.token_program.key(),
            )?;
            invoke(
                &stake_ix,
                &[
                    ctx.accounts.jupiter_stake_pool.clone(),
                    ctx.accounts.yield_reserve.clone(),
                    ctx.accounts.program_jsol_ata.to_account_info(),
                    ctx.accounts.token_program.to_account_info(),
                ],
            )?;

            // Deposit to Kamino
            let kamino_deposit_ix = kamino::deposit_to_multiply_vault(
                &ctx.accounts.kamino_program.key(),
                &ctx.accounts.program_jsol_ata.key(),
                &ctx.accounts.kamino_vault.key(),
                &ctx.accounts.kamino_collateral_ata.key(),
                &ctx.accounts.kamino_debt_ata.key(),
                yield_balance,
                4.6,
                &ctx.accounts.token_program.key(),
            )?;
            invoke(
                &kamino_deposit_ix,
                &[
                    ctx.accounts.program_jsol_ata.to_account_info(),
                    ctx.accounts.kamino_vault.clone(),
                    ctx.accounts.kamino_collateral_ata.to_account_info(),
                    ctx.accounts.kamino_debt_ata.to_account_info(),
                    ctx.accounts.kamino_program.to_account_info(),
                    ctx.accounts.token_program.to_account_info(),
                ],
            )?;

            fund.total_deposits = fund.total_deposits.checked_add(yield_balance).ok_or(ErrorCode::ArithmeticOverflow)?;
        }

        Ok(())
    }

    pub fn find_open_fund(ctx: Context<FindOpenFund>) -> Result<u64> {
        let fund_manager = &ctx.accounts.fund_manager;
        for i in 0..fund_manager.fund_count {
            let fund_seeds = &[b"fund".as_ref(), i.to_le_bytes().as_ref()];
            let fund_key = Pubkey::find_program_address(fund_seeds, &Self::id()).0;
            let fund: Account<Fund> = Account::try_from(&fund_key)?;
            if fund.user_count < 100 {
                return Ok(i);
            }
        }
        Ok(fund_manager.fund_count)
    }

    pub fn get_user_info(ctx: Context<GetUserInfo>, fund_index: u64) -> Result<(u64, u64, u8, bool, bool, Pubkey, u64, u8)> {
        let has_stake_nft = ctx.accounts.sentinel_stake_ata.amount >= 1 && 
                           ctx.accounts.sentinel_stake_ata.mint == ctx.accounts.fund.stake_nft_mint;
        let has_tier_nft = ctx.accounts.sentinel_tier_ata.amount >= 1 && 
                          ctx.accounts.sentinel_tier_ata.mint == ctx.accounts.fund.tier_nft_mint;
        Ok((
            ctx.accounts.user_stake.deposit_amount,
            ctx.accounts.user_stake.stake_timestamp,
            ctx.accounts.user_stake.tier,
            has_stake_nft,
            has_tier_nft,
            ctx.accounts.user_stake.referrer,
            ctx.accounts.user_stake.referral_expiry,
            ctx.accounts.user_stake.auto_reinvest_percentage,
        ))
    }

    fn get_est_apy(tier: u8) -> f64 {
        match tier {
            1 => 11.64,
            2 => 17.45,
            3 => 23.27,
            _ => 0.0,
        }
    }
}

#[account]
pub struct FundManager {
    pub fund_count: u64,
}

#[account]
pub struct Fund {
    pub stake_nft_mint: Pubkey,
    pub tier_nft_mint: Pubkey,
    pub referrer_nft_mint: Pubkey,
    pub total_deposits: u64,
    pub user_count: u64,
    pub early_adopters: u8,
    pub last_rebalance_timestamp: u64,
}

#[account]
pub struct UserStake {
    pub deposit_amount: u64,
    pub stake_timestamp: u64,
    pub tier: u8,
    pub referrer: Pubkey,
    pub referral_expiry: u64,
    pub auto_reinvest_percentage: u8,
}

#[account]
pub struct TempScores {
    pub scores: Vec<TempScore>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TempScore {
    pub user: Pubkey,
    pub score: u64,
}

#[account]
pub struct KaminoProgram;

#[account]
pub struct TokenMetadata;

#[error_code]
pub enum ErrorCode {
    #[msg("User wallet is not authorized (missing Stake NFT)")]
    UnauthorizedWallet,
    #[msg("User has no deposit")]
    NoDeposit,
    #[msg("Fund is already full")]
    FundFull,
    #[msg("Deposit amount must be greater than zero")]
    ZeroAmount,
    #[msg("Invalid tier")]
    InvalidTier,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Invalid vault value")]
    InvalidVaultValue,
    #[msg("High slippage in swap")]
    HighSlippage,
    #[msg("Invalid Tier NFT")]
    InvalidTierNFT,
    #[msg("Invalid batch size")]
    InvalidBatchSize,
    #[msg("Rebalance not due")]
    RebalanceNotDue,
    #[msg("Invalid percentage")]
    InvalidPercentage,
}

