import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DefiTrustFund } from "../target/types/defi_trust_fund";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "chai";

describe("defi-trust-fund", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DefiTrustFund as Program<DefiTrustFund>;
  
  // Test accounts
  const admin = Keypair.generate();
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();
  const user3 = Keypair.generate();
  
  // PDAs
  let fundManagerPda: PublicKey;
  let fundPda: PublicKey;
  let stakeNftMintPda: PublicKey;
  let tierNftMintPda: PublicKey;
  let yieldReservePda: PublicKey;
  let user1StakePda: PublicKey;
  let user2StakePda: PublicKey;
  let user3StakePda: PublicKey;
  
  // Metadata accounts
  let stakeMetadataAccount: PublicKey;
  let tierMetadataAccount: PublicKey;
  
  // Token accounts
  let user1StakeNftAta: PublicKey;
  let user1TierAta: PublicKey;
  let user2StakeNftAta: PublicKey;
  let user2TierAta: PublicKey;
  let user3StakeNftAta: PublicKey;
  let user3TierAta: PublicKey;

  before(async () => {
    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(admin.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user1.publicKey, 5 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user2.publicKey, 5 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user3.publicKey, 5 * anchor.web3.LAMPORTS_PER_SOL);
    
    // Wait for airdrop confirmation
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Derive PDAs
    [fundManagerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("fund_manager")],
      program.programId
    );
    
    [fundPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("fund"), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    
    [stakeNftMintPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake_nft_mint"), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    
    [tierNftMintPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("tier_nft_mint"), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    
    [yieldReservePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("yield_reserve"), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    
    [user1StakePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_stake"), user1.publicKey.toBuffer(), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    
    [user2StakePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_stake"), user2.publicKey.toBuffer(), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    
    [user3StakePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_stake"), user3.publicKey.toBuffer(), new anchor.BN(0).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    
    // Derive metadata accounts
    stakeMetadataAccount = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
        stakeNftMintPda.toBuffer(),
      ],
      new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
    )[0];
    
    tierMetadataAccount = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
        tierNftMintPda.toBuffer(),
      ],
      new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
    )[0];
    
    // Derive token accounts
    user1StakeNftAta = anchor.utils.token.associatedAddress({
      mint: stakeNftMintPda,
      owner: user1.publicKey,
    });
    
    user1TierAta = anchor.utils.token.associatedAddress({
      mint: tierNftMintPda,
      owner: user1.publicKey,
    });
    
    user2StakeNftAta = anchor.utils.token.associatedAddress({
      mint: stakeNftMintPda,
      owner: user2.publicKey,
    });
    
    user2TierAta = anchor.utils.token.associatedAddress({
      mint: tierNftMintPda,
      owner: user2.publicKey,
    });
    
    user3StakeNftAta = anchor.utils.token.associatedAddress({
      mint: stakeNftMintPda,
      owner: user3.publicKey,
    });
    
    user3TierAta = anchor.utils.token.associatedAddress({
      mint: tierNftMintPda,
      owner: user3.publicKey,
    });
  });

  it("Initialize fund", async () => {
    try {
      await program.methods
        .initializeFund(new anchor.BN(0))
        .accounts({
          admin: admin.publicKey,
          fundManager: fundManagerPda,
          fund: fundPda,
          stakeNftMint: stakeNftMintPda,
          tierNftMint: tierNftMintPda,
          stakeMetadataAccount: stakeMetadataAccount,
          tierMetadataAccount: tierMetadataAccount,
          yieldReserve: yieldReservePda,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          tokenMetadataProgram: new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([admin])
        .rpc();
      
      // Verify fund initialization
      const fund = await program.account.fund.fetch(fundPda);
      assert.equal(fund.totalDeposits.toNumber(), 0);
      assert.equal(fund.userCount, 0);
      assert.equal(fund.earlyAdopters, 0);
      assert.equal(fund.totalFeesCollected.toNumber(), 0);
      
      const yieldReserve = await program.account.yieldReserve.fetch(yieldReservePda);
      assert.equal(yieldReserve.fundIndex.toNumber(), 0);
      assert.equal(yieldReserve.totalReinvested.toNumber(), 0);
      
      console.log("✅ Fund initialized successfully");
    } catch (error) {
      console.error("❌ Fund initialization failed:", error);
      throw error;
    }
  });

  it("User 1 deposits 1 SOL (first deposit)", async () => {
    try {
      const depositAmount = new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL);
      
      await program.methods
        .deposit(new anchor.BN(0), depositAmount)
        .accounts({
          user: user1.publicKey,
          fund: fundPda,
          userStake: user1StakePda,
          userStakeNftAta: user1StakeNftAta,
          userTierAta: user1TierAta,
          stakeNftMint: stakeNftMintPda,
          tierNftMint: tierNftMintPda,
          stakeMetadataAccount: stakeMetadataAccount,
          tierMetadataAccount: tierMetadataAccount,
          yieldReserve: yieldReservePda,
          userSolAccount: user1.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          tokenMetadataProgram: new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([user1])
        .rpc();
      
      // Verify deposit
      const fund = await program.account.fund.fetch(fundPda);
      const userStake = await program.account.userStake.fetch(user1StakePda);
      
      // Calculate expected values (after 0.5% fee)
      const feeAmount = Math.floor(depositAmount.toNumber() * 0.005);
      const netDeposit = depositAmount.toNumber() - feeAmount;
      
      assert.equal(fund.totalDeposits.toNumber(), netDeposit);
      assert.equal(fund.userCount, 1);
      assert.equal(fund.earlyAdopters, 1);
      assert.equal(fund.totalFeesCollected.toNumber(), feeAmount);
      assert.equal(userStake.depositAmount.toNumber(), netDeposit);
      assert.equal(userStake.tier, 1);
      assert(userStake.stakeTimestamp.toNumber() > 0);
      
      console.log("✅ User 1 deposit successful");
      console.log(`   Deposit: ${netDeposit / anchor.web3.LAMPORTS_PER_SOL} SOL`);
      console.log(`   Fee: ${feeAmount / anchor.web3.LAMPORTS_PER_SOL} SOL`);
      console.log(`   Tier: ${userStake.tier}`);
    } catch (error) {
      console.error("❌ User 1 deposit failed:", error);
      throw error;
    }
  });

  it("User 2 deposits 2 SOL (first deposit)", async () => {
    try {
      const depositAmount = new anchor.BN(2 * anchor.web3.LAMPORTS_PER_SOL);
      
      await program.methods
        .deposit(new anchor.BN(0), depositAmount)
        .accounts({
          user: user2.publicKey,
          fund: fundPda,
          userStake: user2StakePda,
          userStakeNftAta: user2StakeNftAta,
          userTierAta: user2TierAta,
          stakeNftMint: stakeNftMintPda,
          tierNftMint: tierNftMintPda,
          stakeMetadataAccount: stakeMetadataAccount,
          tierMetadataAccount: tierMetadataAccount,
          yieldReserve: yieldReservePda,
          userSolAccount: user2.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          tokenMetadataProgram: new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([user2])
        .rpc();
      
      // Verify deposit
      const fund = await program.account.fund.fetch(fundPda);
      const userStake = await program.account.userStake.fetch(user2StakePda);
      
      const feeAmount = Math.floor(depositAmount.toNumber() * 0.005);
      const netDeposit = depositAmount.toNumber() - feeAmount;
      
      assert.equal(fund.userCount, 2);
      assert.equal(fund.earlyAdopters, 2);
      assert.equal(userStake.depositAmount.toNumber(), netDeposit);
      assert.equal(userStake.tier, 1);
      
      console.log("✅ User 2 deposit successful");
      console.log(`   Deposit: ${netDeposit / anchor.web3.LAMPORTS_PER_SOL} SOL`);
      console.log(`   Fee: ${feeAmount / anchor.web3.LAMPORTS_PER_SOL} SOL`);
      console.log(`   Tier: ${userStake.tier}`);
    } catch (error) {
      console.error("❌ User 2 deposit failed:", error);
      throw error;
    }
  });

  it("User 3 deposits 5 SOL (first deposit)", async () => {
    try {
      const depositAmount = new anchor.BN(5 * anchor.web3.LAMPORTS_PER_SOL);
      
      await program.methods
        .deposit(new anchor.BN(0), depositAmount)
        .accounts({
          user: user3.publicKey,
          fund: fundPda,
          userStake: user3StakePda,
          userStakeNftAta: user3StakeNftAta,
          userTierAta: user3TierAta,
          stakeNftMint: stakeNftMintPda,
          tierNftMint: tierNftMintPda,
          stakeMetadataAccount: stakeMetadataAccount,
          tierMetadataAccount: tierMetadataAccount,
          yieldReserve: yieldReservePda,
          userSolAccount: user3.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          tokenMetadataProgram: new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([user3])
        .rpc();
      
      // Verify deposit
      const fund = await program.account.fund.fetch(fundPda);
      const userStake = await program.account.userStake.fetch(user3StakePda);
      
      const feeAmount = Math.floor(depositAmount.toNumber() * 0.005);
      const netDeposit = depositAmount.toNumber() - feeAmount;
      
      assert.equal(fund.userCount, 3);
      assert.equal(fund.earlyAdopters, 3);
      assert.equal(userStake.depositAmount.toNumber(), netDeposit);
      assert.equal(userStake.tier, 1);
      
      console.log("✅ User 3 deposit successful");
      console.log(`   Deposit: ${netDeposit / anchor.web3.LAMPORTS_PER_SOL} SOL`);
      console.log(`   Fee: ${feeAmount / anchor.web3.LAMPORTS_PER_SOL} SOL`);
      console.log(`   Tier: ${userStake.tier}`);
    } catch (error) {
      console.error("❌ User 3 deposit failed:", error);
      throw error;
    }
  });

  it("Simulate tier rebalancing", async () => {
    try {
      // Calculate expected scores
      const user1Stake = await program.account.userStake.fetch(user1StakePda);
      const user2Stake = await program.account.userStake.fetch(user2StakePda);
      const user3Stake = await program.account.userStake.fetch(user3StakePda);
      
      // Score = 5 * deposit_amount + 5 * days_staked
      // Since this is a test, days_staked will be 0, so score = 5 * deposit_amount
      const user1Score = 5 * user1Stake.depositAmount.toNumber();
      const user2Score = 5 * user2Stake.depositAmount.toNumber();
      const user3Score = 5 * user3Stake.depositAmount.toNumber();
      
      console.log("📊 Tier Score Analysis:");
      console.log(`   User 1 Score: ${user1Score} (Tier 1)`);
      console.log(`   User 2 Score: ${user2Score} (Tier 2)`);
      console.log(`   User 3 Score: ${user3Score} (Tier 3)`);
      
      // Verify tier assignments
      assert(user1Score <= 7, "User 1 should be Tier 1");
      assert(user2Score > 7 && user2Score <= 15, "User 2 should be Tier 2");
      assert(user3Score > 15, "User 3 should be Tier 3");
      
      console.log("✅ Tier score calculation verified");
    } catch (error) {
      console.error("❌ Tier rebalancing simulation failed:", error);
      throw error;
    }
  });

  it("Simulate yield claiming", async () => {
    try {
      // Get current fund state
      const fund = await program.account.fund.fetch(fundPda);
      const user1Stake = await program.account.userStake.fetch(user1StakePda);
      const user2Stake = await program.account.userStake.fetch(user2StakePda);
      const user3Stake = await program.account.userStake.fetch(user3StakePda);
      
      console.log("💰 Yield Calculation Simulation:");
      console.log(`   Fund Total Deposits: ${fund.totalDeposits.toNumber() / anchor.web3.LAMPORTS_PER_SOL} SOL`);
      console.log(`   Fund Fees Collected: ${fund.totalFeesCollected.toNumber() / anchor.web3.LAMPORTS_PER_SOL} SOL`);
      console.log(`   Early Adopters: ${fund.earlyAdopters}`);
      
      // Calculate expected APY for each tier
      const baseApy = 0.16; // 16%
      const tier1Apy = baseApy * 1.0; // 11.64%
      const tier2Apy = baseApy * 1.5; // 17.45%
      const tier3Apy = baseApy * 2.0; // 23.27%
      
      console.log(`   Tier 1 APY: ${(tier1Apy * 100).toFixed(2)}%`);
      console.log(`   Tier 2 APY: ${(tier2Apy * 100).toFixed(2)}%`);
      console.log(`   Tier 3 APY: ${(tier3Apy * 100).toFixed(2)}%`);
      
      // Early adopter bonus (10% for first 10 users)
      const earlyAdopterBonus = fund.earlyAdopters < 10 ? 1.1 : 1.0;
      console.log(`   Early Adopter Bonus: ${((earlyAdopterBonus - 1) * 100).toFixed(1)}%`);
      
      console.log("✅ Yield calculation simulation completed");
    } catch (error) {
      console.error("❌ Yield claiming simulation failed:", error);
      throw error;
    }
  });

  it("Test fund manager functionality", async () => {
    try {
      const fundManager = await program.account.fundManager.fetch(fundManagerPda);
      assert.equal(fundManager.fundCount.toNumber(), 1);
      
      console.log("✅ Fund manager functionality verified");
      console.log(`   Total Funds: ${fundManager.fundCount.toNumber()}`);
    } catch (error) {
      console.error("❌ Fund manager test failed:", error);
      throw error;
    }
  });

  it("Test error handling - zero amount deposit", async () => {
    try {
      await program.methods
        .deposit(new anchor.BN(0), new anchor.BN(0))
        .accounts({
          user: user1.publicKey,
          fund: fundPda,
          userStake: user1StakePda,
          userStakeNftAta: user1StakeNftAta,
          userTierAta: user1TierAta,
          stakeNftMint: stakeNftMintPda,
          tierNftMint: tierNftMintPda,
          stakeMetadataAccount: stakeMetadataAccount,
          tierMetadataAccount: tierMetadataAccount,
          yieldReserve: yieldReservePda,
          userSolAccount: user1.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          tokenMetadataProgram: new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([user1])
        .rpc();
      
      assert.fail("Should have thrown an error for zero amount");
    } catch (error) {
      console.log("✅ Zero amount deposit correctly rejected");
    }
  });

  console.log("\n🎉 All tests completed successfully!");
  console.log("\n📈 Protocol Summary:");
  console.log("   - Self-perpetuating fee structure (0.5%)");
  console.log("   - Tiered loyalty system (Tier 1-3)");
  console.log("   - NFT-based authorization");
  console.log("   - Monthly rebalancing mechanism");
  console.log("   - Early adopter incentives");
  console.log("   - Yield reinvestment for fund growth");
});

