# Flash Trader Token

The `flash_trader_token` program is a Solana-based decentralized application that provides functionalities for recording trades, managing leaderboards, staking tokens, flash loans, and more.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Program Functions](#program-functions)
  - [record_trade](#record_trade)
  - [reset_leaderboard](#reset_leaderboard)
  - [stake_tokens](#stake_tokens)
  - [unstake_tokens](#unstake_tokens)
  - [flash_loan](#flash_loan)
  - [allocate_liquidity](#allocate_liquidity)
  - [buyback_and_burn](#buyback_and_burn)
- [Accounts](#accounts)
- [Helper Functions](#helper-functions)
- [Error Codes](#error-codes)

## Installation

To install the dependencies and build the program, run the following commands:

```sh
anchor build
```

## Usage

To deploy the program to the local Solana cluster, run:

```sh
anchor deploy
```

To run the tests, use:

```sh
anchor test
```

## Program Functions

### record_trade

Records a trade and updates the trader's statistics. It updates trade counts for different time windows and updates the leaderboard entry with the latest stats.

### reset_leaderboard

Resets the leaderboard, distributes rewards, archives results, and runs lottery bonuses. The reward pool is dynamically calculated based on the provided dex volume.

### stake_tokens

Allows users to stake tokens with advanced staking features. Users can specify a lock duration for bonus incentives. The staking tier is updated based on the new staked amount.

### unstake_tokens

Allows users to unstake tokens. It checks if the staked amount is sufficient before unstaking.

### flash_loan

Allows users to borrow temporary liquidity for high-frequency trading. The loan must be repaid within the same transaction.

### allocate_liquidity

Allocates liquidity pool access based on the staker's tier. Higher-tier stakers get priority in new liquidity pools.

### buyback_and_burn

Automatically uses a portion of fees from staking to buy back and burn tokens.

## Accounts

### RecordTrade

Contains the accounts required for recording a trade, including trader statistics, leaderboard, and user.

### ResetLeaderboard

Contains the accounts required for resetting the leaderboard, including leaderboard, mint, and archive.

### StakeTokens

Contains the accounts required for staking tokens, including staking account, user token account, and user.

### UnstakeTokens

Contains the accounts required for unstaking tokens, including staking account, user token account, and user.

### FlashLoan

Contains the accounts required for a flash loan, including liquidity pool, pool authority, user token account, token program, and user.

### AllocateLiquidity

Contains the accounts required for allocating liquidity, including staking account, liquidity pool, and user.

### BuybackBurn

Contains the accounts required for buyback and burn, including treasury, burn vault, and token program.

## Helper Functions

### update_trade_counts

Updates trade counts for different time windows.

### update_leaderboard

Updates or inserts a trader's stats into the leaderboard, then sorts and limits to the top 10 traders.

### distribute_rewards

Distributes rewards among top traders, applying a decay factor for high streaks.

### distribute_lottery_bonus

Distributes a lottery bonus to traders who exceed a threshold and meet a pseudo-random condition.

### archive_leaderboard

Archives the current leaderboard into the historical archive.

### reset_trade_counts

Resets the trade counts in the leaderboard for the next cycle.

### compute_tier

Computes the staking tier based on the staked amount.

## Error Codes

### ErrorCode

Defines error codes for the program, including an error for insufficient stake to complete an operation.
