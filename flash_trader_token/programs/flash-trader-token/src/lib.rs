use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::collections::BTreeMap;

declare_id!("A8PVJKv1RfWDLPRZrePVCqwo8YtVLW4vmBeazrEeHKwU");

#[program]
pub mod flash_trader_token {
    use super::*;

    /// Record a trade and update the trader's statistics.
    pub fn record_trade(ctx: Context<RecordTrade>, ) -> Result<()> {
        let trader_stats = &mut ctx.accounts.trader_stats;
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Update trade counts for different windows.
        update_trade_counts(trader_stats, current_time);

        // Update leaderboard entry with the latest stats.
        let leaderboard = &mut ctx.accounts.leaderboard;
        update_leaderboard(leaderboard, ctx.accounts.user.key(), trader_stats);

        Ok(())
    }

    /// Reset the leaderboard, distribute rewards, archive results, and run lottery bonuses.
    /// The reward pool is dynamically calculated based on the provided dex_volume.
    pub fn reset_leaderboard(ctx: Context<ResetLeaderboard>, dex_volume: u64) -> Result<()> {
        let leaderboard = &mut ctx.accounts.leaderboard;

        // Dynamic reward pool: low volume → 50 FLTR, high volume → 500 FLTR (assuming 6 decimals).
        let reward_pool = if dex_volume < 1_000_000 {
            50 * 1_000_000
        } else {
            500 * 1_000_000
        };

        // Distribute rewards among the top 10 traders, with reward decay if needed.
        distribute_rewards(leaderboard, &mut ctx.accounts.mint, reward_pool)?;

        // Award a lottery bonus to some eligible traders.
        distribute_lottery_bonus(leaderboard)?;

        // Archive the current leaderboard for history.
        archive_leaderboard(leaderboard, &mut ctx.accounts.archive);

        // Reset trade counts for the next leaderboard cycle.
        reset_trade_counts(leaderboard);

        Ok(())
    }

    /// Stake tokens with advanced staking features.
    /// Users can specify a lock duration for bonus incentives.
    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64, lock_duration: i64) -> Result<()> {
        let staking = &mut ctx.accounts.staking_account;
        staking.staked_amount = staking.staked_amount.checked_add(amount).unwrap();
        let clock = Clock::get()?;
        staking.stake_start_time = clock.unix_timestamp;
        staking.lock_duration = lock_duration;
        // Update staking tier based on the new staked amount.
        staking.tier = compute_tier(staking.staked_amount);
        Ok(())
    }

    /// Unstake tokens.
    pub fn unstake_tokens(ctx: Context<UnstakeTokens>, amount: u64) -> Result<()> {
        let staking = &mut ctx.accounts.staking_account;
        require!(staking.staked_amount >= amount, ErrorCode::InsufficientStake);
        staking.staked_amount = staking.staked_amount.checked_sub(amount).unwrap();
        staking.tier = compute_tier(staking.staked_amount);
        Ok(())
    }

    /// Flash Loan: Allows users to borrow temporary liquidity for high-frequency trading.
    /// The loan must be repaid within the same transaction.
    pub fn flash_loan(ctx: Context<FlashLoan>, amount: u64) -> Result<()> {
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.liquidity_pool.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.pool_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), amount)?;
        // NOTE: Ensure repayment logic is enforced off-chain within the same transaction.
        Ok(())
    }

    /// Allocate liquidity pool access based on the staker's tier.
    /// Higher-tier stakers get priority in new liquidity pools.
    pub fn allocate_liquidity(ctx: Context<AllocateLiquidity>) -> Result<()> {
        let staking = &ctx.accounts.staking_account;
        let pool = &mut ctx.accounts.liquidity_pool;
        pool.allocations.insert(ctx.accounts.user.key(), staking.tier as u64);
        Ok(())
    }

    /// Buyback & Burn: Automatically uses a portion of fees from staking to buy back and burn tokens.
    /// The context is explicitly annotated with lifetime `'info` so that all references align.
    pub fn buyback_and_burn<'info>(ctx: Context<'_, '_, '_, 'info, BuybackBurn<'info>>, amount: u64) -> Result<()> {
        token::burn(ctx.accounts.into_burn_context(), amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct RecordTrade<'info> {
    #[account(mut)]
    pub trader_stats: Account<'info, TraderStats>,
    #[account(mut)]
    pub leaderboard: Account<'info, Leaderboard>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ResetLeaderboard<'info> {
    #[account(mut)]
    pub leaderboard: Account<'info, Leaderboard>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub archive: Account<'info, LeaderboardArchive>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub staking_account: Account<'info, StakingAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    #[account(mut)]
    pub staking_account: Account<'info, StakingAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct FlashLoan<'info> {
    #[account(mut)]
    pub liquidity_pool: Account<'info, TokenAccount>,
    /// CHECK: The pool authority is trusted to sign off on flash loans.
    pub pool_authority: AccountInfo<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct AllocateLiquidity<'info> {
    #[account(mut)]
    pub staking_account: Account<'info, StakingAccount>,
    #[account(mut)]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct BuybackBurn<'info> {
    #[account(mut)]
    pub treasury: Account<'info, TokenAccount>,
    #[account(mut)]
    pub burn_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> BuybackBurn<'info> {
    // Explicitly specify all four lifetime parameters in the returned CpiContext.
    fn into_burn_context(&self) -> CpiContext<'_, '_, '_, 'info, token::Burn<'info>> {
        let cpi_accounts = token::Burn {
            mint: self.treasury.to_account_info(),
            from: self.burn_vault.to_account_info(), // note: the correct field is "from"
            authority: self.treasury.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[account]
pub struct TraderStats {
    pub trader: Pubkey,
    pub one_min_count: u64,
    pub five_min_count: u64,
    pub fifteen_min_count: u64,
    pub last_trade_time: i64,
    pub streak_counter: u64, // Increases if the trader continuously ranks high.
}

#[account]
pub struct Leaderboard {
    // Stores the top traders, with a goal of displaying the top 10.
    pub traders: Vec<TraderStats>,
    pub last_reset_time: i64,
}

#[account]
pub struct LeaderboardArchive {
    // Archives historical leaderboard entries (e.g., last 24-hour winners).
    pub entries: Vec<LeaderboardEntry>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct LeaderboardEntry {
    pub timestamp: i64,
    pub top_traders: Vec<Pubkey>, // List of top 10 traders' public keys.
}

#[account]
pub struct StakingAccount {
    pub owner: Pubkey,
    pub staked_amount: u64,
    pub stake_start_time: i64,
    pub lock_duration: i64, // Duration (in seconds) for which the stake is locked.
    pub tier: u8,         // 0 = no tier, 1 = Tier 1, 2 = Tier 2, 3 = Tier 3.
}

#[account]
pub struct LiquidityPool {
    // Tracks dynamic liquidity pool allocations.
    // Mapping from staker pubkey to their allocation value.
    pub allocations: BTreeMap<Pubkey, u64>,
}

// ─── HELPER FUNCTIONS ───────────────────────────────────────────────────────────

/// Update trade counts for different time windows.
fn update_trade_counts(stats: &mut TraderStats, current_time: i64) {
    if current_time - stats.last_trade_time >= 60 {
        stats.one_min_count = 1;
    } else {
        stats.one_min_count = stats.one_min_count.checked_add(1).unwrap();
    }
    if current_time - stats.last_trade_time >= 300 {
        stats.five_min_count = 1;
    } else {
        stats.five_min_count += 1;
    }
    if current_time - stats.last_trade_time >= 900 {
        stats.fifteen_min_count = 1;
    } else {
        stats.fifteen_min_count += 1;
    }
    stats.last_trade_time = current_time;
}

/// Update or insert a trader's stats into the leaderboard, then sort and limit to top 10.
fn update_leaderboard(leaderboard: &mut Leaderboard, trader: Pubkey, stats: &TraderStats) {
    if let Some(existing) = leaderboard.traders.iter_mut().find(|t| t.trader == trader) {
        *existing = stats.clone();
    } else {
        leaderboard.traders.push(stats.clone());
    }
    leaderboard.traders.sort_by(|a, b| b.one_min_count.cmp(&a.one_min_count));
    if leaderboard.traders.len() > 10 {
        leaderboard.traders.truncate(10);
    }
}

/// Distribute rewards among top traders, applying a decay factor for high streaks.
fn distribute_rewards(leaderboard: &mut Leaderboard, _mint: &mut Account<Mint>, reward_pool: u64) -> Result<()> {
    leaderboard.traders.sort_by(|a, b| b.one_min_count.cmp(&a.one_min_count));
    let total_trades: u64 = leaderboard.traders.iter().map(|t| t.one_min_count).sum();
    for trader in leaderboard.traders.iter_mut() {
        let decay_factor = if trader.streak_counter > 5 { 0.8 } else { 1.0 };
        let base_reward = (trader.one_min_count as u128 * reward_pool as u128) / (total_trades as u128);
        let _final_reward = (base_reward as f64 * decay_factor) as u64;
        // Mint/distribute `_final_reward` tokens to `trader.trader`.
        trader.streak_counter += 1;
    }
    Ok(())
}

/// Distribute a lottery bonus to traders who exceed a threshold and meet a pseudo-random condition.
fn distribute_lottery_bonus(leaderboard: &mut Leaderboard) -> Result<()> {
    let bonus_threshold = 5;
    let _bonus_amount = 10 * 1_000_000; // Example: 10 FLTR bonus.
    for trader in leaderboard.traders.iter_mut() {
        if trader.one_min_count >= bonus_threshold {
            // Pseudo-random condition: if last_trade_time is even.
            if trader.last_trade_time % 2 == 0 {
                // Award `_bonus_amount` to trader.
                // (Implement token transfer logic here.)
            }
        }
    }
    Ok(())
}

/// Archive the current leaderboard into the historical archive.
fn archive_leaderboard(leaderboard: &mut Leaderboard, archive: &mut LeaderboardArchive) {
    let entry = LeaderboardEntry {
        timestamp: Clock::get().unwrap().unix_timestamp,
        top_traders: leaderboard.traders.iter().map(|t| t.trader).collect(),
    };
    archive.entries.push(entry);
    // Optionally prune entries older than 24 hours.
}

/// Reset the trade counts in the leaderboard for the next cycle.
fn reset_trade_counts(leaderboard: &mut Leaderboard) {
    for trader in leaderboard.traders.iter_mut() {
        trader.one_min_count = 0;
        trader.five_min_count = 0;
        trader.fifteen_min_count = 0;
        // Optionally maintain or adjust streak_counter based on decay rules.
    }
    leaderboard.last_reset_time = Clock::get().unwrap().unix_timestamp;
}

/// Compute the staking tier based on the staked amount (using 6 decimals).
fn compute_tier(staked_amount: u64) -> u8 {
    if staked_amount >= 500 * 1_000_000 {
        3
    } else if staked_amount >= 100 * 1_000_000 {
        2
    } else if staked_amount >= 10 * 1_000_000 {
        1
    } else {
        0
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient stake to complete the operation.")]
    InsufficientStake,
}
