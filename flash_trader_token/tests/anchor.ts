import * as anchor from "@coral-xyz/anchor";
import BN from "bn.js";
import assert from "assert";
import * as web3 from "@solana/web3.js";
import type { FlashTraderToken } from "../target/types/flash_trader_token";

describe("FlashTraderToken Tests", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.FlashTraderToken as anchor.Program<FlashTraderToken>;

  it("initialize", async () => {
    // Generate keypair for the new account
    const newAccountKp = new web3.Keypair();

    // Send transaction
    const data = new BN(42);
    const txHash = await program.methods
      .initialize(data)
      .accounts({
        newAccount: newAccountKp.publicKey,
        signer: program.provider.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([newAccountKp])
      .rpc();
    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    // Confirm transaction
    await program.provider.connection.confirmTransaction(txHash);

    // Fetch the created account
    const newAccount = await program.account.newAccount.fetch(
      newAccountKp.publicKey
    );

    console.log("On-chain data is:", newAccount.data.toString());

    // Check whether the data on-chain is equal to local 'data'
    assert(data.eq(newAccount.data));
  });

  it("record_trade", async () => {
    const traderStatsKp = new web3.Keypair();
    const leaderboardKp = new web3.Keypair();
    const user = program.provider.wallet;

    await program.methods
      .recordTrade()
      .accounts({
        traderStats: traderStatsKp.publicKey,
        leaderboard: leaderboardKp.publicKey,
        user: user.publicKey,
      })
      .signers([traderStatsKp, leaderboardKp])
      .rpc();

    const traderStats = await program.account.traderStats.fetch(traderStatsKp.publicKey);
    console.log("Trader Stats:", traderStats);
  });

  it("reset_leaderboard", async () => {
    const leaderboardKp = new web3.Keypair();
    const mintKp = new web3.Keypair();
    const archiveKp = new web3.Keypair();

    await program.methods
      .resetLeaderboard(new BN(1_000_000))
      .accounts({
        leaderboard: leaderboardKp.publicKey,
        mint: mintKp.publicKey,
        archive: archiveKp.publicKey,
      })
      .signers([leaderboardKp, mintKp, archiveKp])
      .rpc();

    const leaderboard = await program.account.leaderboard.fetch(leaderboardKp.publicKey);
    console.log("Leaderboard:", leaderboard);
  });

  it("stake_tokens", async () => {
    const stakingAccountKp = new web3.Keypair();
    const userTokenAccountKp = new web3.Keypair();
    const user = program.provider.wallet;

    await program.methods
      .stakeTokens(new BN(100), new BN(3600))
      .accounts({
        stakingAccount: stakingAccountKp.publicKey,
        userTokenAccount: userTokenAccountKp.publicKey,
        user: user.publicKey,
      })
      .signers([stakingAccountKp, userTokenAccountKp])
      .rpc();

    const stakingAccount = await program.account.stakingAccount.fetch(stakingAccountKp.publicKey);
    console.log("Staking Account:", stakingAccount);
  });

  it("unstake_tokens", async () => {
    const stakingAccountKp = new web3.Keypair();
    const userTokenAccountKp = new web3.Keypair();
    const user = program.provider.wallet;

    await program.methods
      .unstakeTokens(new BN(50))
      .accounts({
        stakingAccount: stakingAccountKp.publicKey,
        userTokenAccount: userTokenAccountKp.publicKey,
        user: user.publicKey,
      })
      .signers([stakingAccountKp, userTokenAccountKp])
      .rpc();

    const stakingAccount = await program.account.stakingAccount.fetch(stakingAccountKp.publicKey);
    console.log("Staking Account:", stakingAccount);
  });

  it("flash_loan", async () => {
    const liquidityPoolKp = new web3.Keypair();
    const userTokenAccountKp = new web3.Keypair();
    const poolAuthority = program.provider.wallet;
    const user = program.provider.wallet;

    await program.methods
      .flashLoan(new BN(1000))
      .accounts({
        liquidityPool: liquidityPoolKp.publicKey,
        poolAuthority: poolAuthority.publicKey,
        userTokenAccount: userTokenAccountKp.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        user: user.publicKey,
      })
      .signers([liquidityPoolKp, userTokenAccountKp])
      .rpc();

    const liquidityPool = await program.account.tokenAccount.fetch(liquidityPoolKp.publicKey);
    console.log("Liquidity Pool:", liquidityPool);
  });

  it("allocate_liquidity", async () => {
    const stakingAccountKp = new web3.Keypair();
    const liquidityPoolKp = new web3.Keypair();
    const user = program.provider.wallet;

    await program.methods
      .allocateLiquidity()
      .accounts({
        stakingAccount: stakingAccountKp.publicKey,
        liquidityPool: liquidityPoolKp.publicKey,
        user: user.publicKey,
      })
      .signers([stakingAccountKp, liquidityPoolKp])
      .rpc();

    const liquidityPool = await program.account.liquidityPool.fetch(liquidityPoolKp.publicKey);
    console.log("Liquidity Pool:", liquidityPool);
  });

  it("buyback_and_burn", async () => {
    const treasuryKp = new web3.Keypair();
    const burnVaultKp = new web3.Keypair();
    const user = program.provider.wallet;

    await program.methods
      .buybackAndBurn(new BN(1000))
      .accounts({
        treasury: treasuryKp.publicKey,
        burnVault: burnVaultKp.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([treasuryKp, burnVaultKp])
      .rpc();

    const treasury = await program.account.tokenAccount.fetch(treasuryKp.publicKey);
    console.log("Treasury:", treasury);
  });
});
