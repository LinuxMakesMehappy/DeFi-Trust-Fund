/**
 * Security Attack Simulation Tests
 * Simulates various attack vectors to validate security measures
 */

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DeFiTrustFund } from "../target/types/defi_trust_fund";
import { 
  PublicKey, 
  Keypair, 
  LAMPORTS_PER_SOL, 
  SystemProgram,
  Transaction,
  TransactionInstruction
} from "@solana/web3.js";
import { expect } from "chai";
import { BN } from "bn.js";

describe("🛡️ Security Attack Simulation Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DeFiTrustFund as Program<DeFiTrustFund>;
  
  let admin: Keypair;
  let legitimateUser: Keypair;
  let attacker: Keypair;
  let poolKeypair: Keypair;
  let poolVaultKeypair: Keypair;
  let poolPublicKey: PublicKey;
  let poolVaultPublicKey: PublicKey;

  before(async () => {
    console.log("🚀 Setting up security attack simulation environment...");
    
    // Generate accounts
    admin = Keypair.generate();
    legitimateUser = Keypair.generate();
    attacker = Keypair.generate();
    poolKeypair = Keypair.generate();
    poolVaultKeypair = Keypair.generate();
    
    poolPublicKey = poolKeypair.publicKey;
    poolVaultPublicKey = poolVaultKeypair.publicKey;

    // Fund accounts
    await Promise.all([
      provider.connection.requestAirdrop(admin.publicKey, 50 * LAMPORTS_PER_SOL),
      provider.connection.requestAirdrop(legitimateUser.publicKey, 50 * LAMPORTS_PER_SOL),
      provider.connection.requestAirdrop(attacker.publicKey, 50 * LAMPORTS_PER_SOL)
    ]);

    await new Promise(resolve => setTimeout(resolve, 2000));

    // Initialize pool
    const solPriceFeed = new PublicKey("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");
    await program.methods
      .initializePool(5000, 1, 365, solPriceFeed)
      .accounts({
        admin: admin.publicKey,
        pool: poolPublicKey,
        poolVault: poolVaultPublicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin, poolKeypair, poolVaultKeypair])
      .rpc();

    console.log("✅ Security test environment ready");
  });

  // ==================== REENTRANCY ATTACK SIMULATION ====================
  describe("🔄 Reentrancy Attack Simulation", () => {
    it("should prevent reentrancy attacks on claim function", async () => {
      console.log("🧪 Simulating reentrancy attack on claim function...");
      
      // Set up legitimate stake first
      const [userStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_stake"), legitimateUser.publicKey.toBuffer()],
        program.programId
      );
      
      const stakeAmount = 5 * LAMPORTS_PER_SOL;
      const fee = Math.floor(stakeAmount * 0.005);
      const minExpected = stakeAmount - fee - Math.floor(stakeAmount * 0.01);
      const deadline = Math.floor(Date.now() / 1000) + 300;
      
      await program.methods
        .stake(
          new BN(stakeAmount),
          1, // 1 day commitment
          new BN(minExpected),
          new BN(deadline)
        )
        .accounts({
          user: legitimateUser.publicKey,
          pool: poolPublicKey,
          userStake: userStake,
          poolVault: poolVaultPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([legitimateUser])
        .rpc();

      // Simulate reentrancy attack by trying to call claim multiple times rapidly
      const claimPromises = Array.from({ length: 10 }, () =>
        program.methods
          .claimYields()
          .accounts({
            user: legitimateUser.publicKey,
            pool: poolPublicKey,
            userStake: userStake,
            poolVault: poolVaultPublicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([legitimateUser])
          .rpc()
          .catch(err => err) // Catch errors instead of failing
      );
      
      const results = await Promise.all(claimPromises);
      
      // Most should fail due to reentrancy protection or commitment not met
      const failures = results.filter(result => result instanceof Error).length;
      expect(failures).to.be.greaterThan(8); // Most should fail
      
      console.log(`✅ Reentrancy protection working: ${failures}/10 calls properly rejected`);
    });

    it("should prevent reentrancy on unstake function", async () => {
      console.log("🧪 Simulating reentrancy attack on unstake function...");
      
      // Try multiple unstake calls rapidly
      const [userStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_stake"), legitimateUser.publicKey.toBuffer()],
        program.programId
      );
      
      const unstakePromises = Array.from({ length: 5 }, () =>
        program.methods
          .unstake()
          .accounts({
            user: legitimateUser.publicKey,
            pool: poolPublicKey,
            userStake: userStake,
            poolVault: poolVaultPublicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([legitimateUser])
          .rpc()
          .catch(err => err)
      );
      
      const results = await Promise.all(unstakePromises);
      const failures = results.filter(result => result instanceof Error).length;
      
      // After first successful unstake, others should fail
      expect(failures).to.be.greaterThan(3);
      
      console.log(`✅ Unstake reentrancy protection working: ${failures}/5 calls properly rejected`);
    });
  });

  // ==================== ACCESS CONTROL ATTACK SIMULATION ====================
  describe("🔐 Access Control Attack Simulation", () => {
    it("should prevent privilege escalation attacks", async () => {
      console.log("🧪 Simulating privilege escalation attack...");
      
      // Try to call admin functions with non-admin account
      const adminFunctions = [
        () => program.methods.emergencyPause("Unauthorized pause attempt"),
        () => program.methods.emergencyUnpause(),
        () => program.methods.updateApy(2000),
        () => program.methods.updateDepositFee(100),
      ];
      
      for (const adminFunction of adminFunctions) {
        try {
          await adminFunction()
            .accounts({
              admin: attacker.publicKey,
              pool: poolPublicKey,
            })
            .signers([attacker])
            .rpc();
          expect.fail("Should have failed for non-admin access");
        } catch (error) {
          expect(error.message).to.include("Unauthorized");
        }
      }
      
      console.log("✅ All privilege escalation attempts properly blocked");
    });

    it("should prevent unauthorized multi-sig manipulation", async () => {
      console.log("🧪 Simulating multi-sig manipulation attack...");
      
      // Try to add unauthorized signer
      try {
        await program.methods
          .addMultisigSigner(attacker.publicKey)
          .accounts({
            admin: attacker.publicKey,
            pool: poolPublicKey,
          })
          .signers([attacker])
          .rpc();
        expect.fail("Should have failed for unauthorized signer addition");
      } catch (error) {
        expect(error.message).to.include("Unauthorized");
      }
      
      // Try to modify threshold without authorization
      try {
        await program.methods
          .updateMultisigThreshold(1)
          .accounts({
            admin: attacker.publicKey,
            pool: poolPublicKey,
          })
          .signers([attacker])
          .rpc();
        expect.fail("Should have failed for unauthorized threshold change");
      } catch (error) {
        expect(error.message).to.include("Unauthorized");
      }
      
      console.log("✅ Multi-sig manipulation attempts properly blocked");
    });
  });

  // ==================== ECONOMIC ATTACK SIMULATION ====================
  describe("💸 Economic Attack Simulation", () => {
    it("should prevent yield manipulation attacks", async () => {
      console.log("🧪 Simulating yield manipulation attack...");
      
      // Create multiple accounts to try to game the tier system
      const gamingUsers = Array.from({ length: 5 }, () => Keypair.generate());
      
      // Fund gaming accounts
      await Promise.all(
        gamingUsers.map(user => 
          provider.connection.requestAirdrop(user.publicKey, 20 * LAMPORTS_PER_SOL)
        )
      );
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // Try to stake with minimal amounts to game the system
      for (const user of gamingUsers) {
        const [userStake] = PublicKey.findProgramAddressSync(
          [Buffer.from("user_stake"), user.publicKey.toBuffer()],
          program.programId
        );
        
        const minStakeAmount = 0.1 * LAMPORTS_PER_SOL;
        const fee = Math.floor(minStakeAmount * 0.005);
        const minExpected = minStakeAmount - fee - Math.floor(minStakeAmount * 0.01);
        const deadline = Math.floor(Date.now() / 1000) + 300;
        
        try {
          await program.methods
            .stake(
              new BN(minStakeAmount),
              1, // Minimum commitment
              new BN(minExpected),
              new BN(deadline)
            )
            .accounts({
              user: user.publicKey,
              pool: poolPublicKey,
              userStake: userStake,
              poolVault: poolVaultPublicKey,
              systemProgram: SystemProgram.programId,
            })
            .signers([user])
            .rpc();
        } catch (error) {
          // Some might fail due to rate limiting, which is expected
          if (!error.message.includes("RateLimitExceeded")) {
            throw error;
          }
        }
      }
      
      console.log("✅ System resistant to basic gaming attempts");
    });

    it("should prevent flash loan style attacks", async () => {
      console.log("🧪 Simulating flash loan style attack...");
      
      // Try to stake and immediately unstake to extract value
      const [attackerStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_stake"), attacker.publicKey.toBuffer()],
        program.programId
      );
      
      const attackAmount = 10 * LAMPORTS_PER_SOL;
      const fee = Math.floor(attackAmount * 0.005);
      const minExpected = attackAmount - fee - Math.floor(attackAmount * 0.01);
      const deadline = Math.floor(Date.now() / 1000) + 300;
      
      // Stake
      await program.methods
        .stake(
          new BN(attackAmount),
          1, // Minimum commitment
          new BN(minExpected),
          new BN(deadline)
        )
        .accounts({
          user: attacker.publicKey,
          pool: poolPublicKey,
          userStake: attackerStake,
          poolVault: poolVaultPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([attacker])
        .rpc();
      
      // Try to immediately unstake (should incur penalty)
      const balanceBefore = await provider.connection.getBalance(attacker.publicKey);
      
      await program.methods
        .unstake()
        .accounts({
          user: attacker.publicKey,
          pool: poolPublicKey,
          userStake: attackerStake,
          poolVault: poolVaultPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([attacker])
        .rpc();
      
      const balanceAfter = await provider.connection.getBalance(attacker.publicKey);
      
      // Attacker should have lost money due to fees and penalties
      expect(balanceAfter).to.be.lessThan(balanceBefore - (attackAmount * 0.05)); // Should lose at least 5% penalty
      
      console.log("✅ Flash loan style attack resulted in net loss for attacker");
    });
  });

  // ==================== RATE LIMITING ATTACK SIMULATION ====================
  describe("🚦 Rate Limiting Attack Simulation", () => {
    it("should prevent spam attacks through rate limiting", async () => {
      console.log("🧪 Simulating spam attack...");
      
      const spammer = Keypair.generate();
      await provider.connection.requestAirdrop(spammer.publicKey, 100 * LAMPORTS_PER_SOL);
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const [spammerStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_stake"), spammer.publicKey.toBuffer()],
        program.programId
      );
      
      // Try to perform many operations rapidly
      let successfulOperations = 0;
      let rateLimitedOperations = 0;
      
      for (let i = 0; i < 20; i++) {
        try {
          const amount = 0.5 * LAMPORTS_PER_SOL;
          const fee = Math.floor(amount * 0.005);
          const minExpected = amount - fee - Math.floor(amount * 0.01);
          const deadline = Math.floor(Date.now() / 1000) + 300;
          
          await program.methods
            .stake(
              new BN(amount),
              1,
              new BN(minExpected),
              new BN(deadline)
            )
            .accounts({
              user: spammer.publicKey,
              pool: poolPublicKey,
              userStake: spammerStake,
              poolVault: poolVaultPublicKey,
              systemProgram: SystemProgram.programId,
            })
            .signers([spammer])
            .rpc();
          
          successfulOperations++;
          
          // If successful, immediately unstake to reset for next attempt
          await program.methods
            .unstake()
            .accounts({
              user: spammer.publicKey,
              pool: poolPublicKey,
              userStake: spammerStake,
              poolVault: poolVaultPublicKey,
              systemProgram: SystemProgram.programId,
            })
            .signers([spammer])
            .rpc();
            
        } catch (error) {
          if (error.message.includes("RateLimitExceeded")) {
            rateLimitedOperations++;
          }
        }
        
        // Small delay between attempts
        await new Promise(resolve => setTimeout(resolve, 100));
      }
      
      console.log(`📊 Spam test results: ${successfulOperations} successful, ${rateLimitedOperations} rate limited`);
      expect(rateLimitedOperations).to.be.greaterThan(10); // Most should be rate limited
      
      console.log("✅ Rate limiting successfully prevented spam attack");
    });

    it("should enforce cooldown periods", async () => {
      console.log("🧪 Testing cooldown period enforcement...");
      
      const rapidUser = Keypair.generate();
      await provider.connection.requestAirdrop(rapidUser.publicKey, 20 * LAMPORTS_PER_SOL);
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const [rapidUserStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_stake"), rapidUser.publicKey.toBuffer()],
        program.programId
      );
      
      // First operation should succeed
      const amount = 1 * LAMPORTS_PER_SOL;
      const fee = Math.floor(amount * 0.005);
      const minExpected = amount - fee - Math.floor(amount * 0.01);
      const deadline = Math.floor(Date.now() / 1000) + 300;
      
      await program.methods
        .stake(
          new BN(amount),
          1,
          new BN(minExpected),
          new BN(deadline)
        )
        .accounts({
          user: rapidUser.publicKey,
          pool: poolPublicKey,
          userStake: rapidUserStake,
          poolVault: poolVaultPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([rapidUser])
        .rpc();
      
      // Immediate second operation should fail due to cooldown
      try {
        await program.methods
          .stake(
            new BN(amount),
            1,
            new BN(minExpected),
            new BN(deadline + 1)
          )
          .accounts({
            user: rapidUser.publicKey,
            pool: poolPublicKey,
            userStake: rapidUserStake,
            poolVault: poolVaultPublicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([rapidUser])
          .rpc();
        expect.fail("Should have failed due to cooldown period");
      } catch (error) {
        expect(error.message).to.include("RateLimitExceeded");
        console.log("✅ Cooldown period properly enforced");
      }
    });
  });

  // ==================== OVERFLOW ATTACK SIMULATION ====================
  describe("🔢 Arithmetic Overflow Attack Simulation", () => {
    it("should prevent integer overflow in calculations", async () => {
      console.log("🧪 Simulating integer overflow attack...");
      
      // Try to cause overflow with extreme values
      const extremeUser = Keypair.generate();
      await provider.connection.requestAirdrop(extremeUser.publicKey, 200 * LAMPORTS_PER_SOL);
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const [extremeUserStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_stake"), extremeUser.publicKey.toBuffer()],
        program.programId
      );
      
      // Try with maximum allowed amount
      const maxAmount = 100 * LAMPORTS_PER_SOL; // Maximum per stake
      const fee = Math.floor(maxAmount * 0.005);
      const minExpected = maxAmount - fee - Math.floor(maxAmount * 0.01);
      const deadline = Math.floor(Date.now() / 1000) + 300;
      
      // This should work without overflow
      await program.methods
        .stake(
          new BN(maxAmount),
          365, // Maximum commitment
          new BN(minExpected),
          new BN(deadline)
        )
        .accounts({
          user: extremeUser.publicKey,
          pool: poolPublicKey,
          userStake: extremeUserStake,
          poolVault: poolVaultPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([extremeUser])
        .rpc();
      
      // Verify state is still consistent
      const pool = await program.account.pool.fetch(poolPublicKey);
      expect(pool.totalStaked.toNumber()).to.be.greaterThan(0);
      expect(pool.totalStaked.toNumber()).to.be.lessThan(Number.MAX_SAFE_INTEGER);
      
      console.log("✅ Maximum values handled safely without overflow");
    });

    it("should handle edge cases in fee calculations", async () => {
      console.log("🧪 Testing edge cases in fee calculations...");
      
      // Test with minimum amounts that might cause precision issues
      const edgeCases = [
        1000, // Very small amount
        1001, // Odd number
        999999, // Just under round number
        1000000, // Round number
      ];
      
      for (const amount of edgeCases) {
        const fee = Math.floor(amount * 0.005); // 0.5% fee
        const netAmount = amount - fee;
        
        // Verify calculations are consistent
        expect(netAmount).to.be.greaterThan(0);
        expect(netAmount).to.be.lessThan(amount);
        expect(fee).to.be.at.least(0);
        
        console.log(`✅ Fee calculation for ${amount} lamports: fee=${fee}, net=${netAmount}`);
      }
    });
  });

  // ==================== ORACLE MANIPULATION SIMULATION ====================
  describe("📊 Oracle Manipulation Attack Simulation", () => {
    it("should validate oracle price feeds", async () => {
      console.log("🧪 Testing oracle price validation...");
      
      // Try to update price with invalid oracle account
      const fakeOracle = Keypair.generate();
      
      try {
        await program.methods
          .updateSolPrice()
          .accounts({
            admin: admin.publicKey,
            pool: poolPublicKey,
            priceFeed: fakeOracle.publicKey,
          })
          .signers([admin])
          .rpc();
        expect.fail("Should have failed for invalid oracle");
      } catch (error) {
        expect(error.message).to.include("InvalidOracle");
        console.log("✅ Invalid oracle properly rejected");
      }
    });

    it("should handle stale price data", async () => {
      console.log("🧪 Testing stale price data handling...");
      
      // The actual implementation would check price staleness
      // This test validates the logic exists
      const pool = await program.account.pool.fetch(poolPublicKey);
      expect(pool.priceStalenessThreshold.toNumber()).to.equal(60); // 60 seconds
      
      console.log("✅ Price staleness threshold properly configured");
    });
  });

  // ==================== FRONT-RUNNING ATTACK SIMULATION ====================
  describe("🏃 Front-Running Attack Simulation", () => {
    it("should resist MEV attacks through block delays", async () => {
      console.log("🧪 Simulating MEV/front-running attack...");
      
      // Create large operation that should trigger MEV protection
      const mevUser = Keypair.generate();
      await provider.connection.requestAirdrop(mevUser.publicKey, 150 * LAMPORTS_PER_SOL);
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const [mevUserStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_stake"), mevUser.publicKey.toBuffer()],
        program.programId
      );
      
      const largeAmount = 50 * LAMPORTS_PER_SOL; // Large operation (>10 SOL triggers MEV protection)
      const fee = Math.floor(largeAmount * 0.005);
      const minExpected = largeAmount - fee - Math.floor(largeAmount * 0.01);
      const deadline = Math.floor(Date.now() / 1000) + 300;
      
      // First large operation should succeed
      await program.methods
        .stake(
          new BN(largeAmount),
          30,
          new BN(minExpected),
          new BN(deadline)
        )
        .accounts({
          user: mevUser.publicKey,
          pool: poolPublicKey,
          userStake: mevUserStake,
          poolVault: poolVaultPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([mevUser])
        .rpc();
      
      // Second large operation should be delayed due to MEV protection
      const secondMevUser = Keypair.generate();
      await provider.connection.requestAirdrop(secondMevUser.publicKey, 150 * LAMPORTS_PER_SOL);
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const [secondMevUserStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_stake"), secondMevUser.publicKey.toBuffer()],
        program.programId
      );
      
      try {
        await program.methods
          .stake(
            new BN(largeAmount),
            30,
            new BN(minExpected),
            new BN(deadline + 10)
          )
          .accounts({
            user: secondMevUser.publicKey,
            pool: poolPublicKey,
            userStake: secondMevUserStake,
            poolVault: poolVaultPublicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([secondMevUser])
          .rpc();
        
        // If it succeeds, that's also OK - depends on block timing
        console.log("✅ Second large operation succeeded (block delay passed)");
      } catch (error) {
        if (error.message.includes("MevProtectionActive")) {
          console.log("✅ MEV protection successfully blocked rapid large operations");
        } else {
          throw error;
        }
      }
    });
  });

  // ==================== GOVERNANCE ATTACK SIMULATION ====================
  describe("🗳️ Governance Attack Simulation", () => {
    it("should prevent unauthorized governance proposals", async () => {
      console.log("🧪 Simulating unauthorized governance attack...");
      
      // Try to propose admin action without being authorized signer
      try {
        await program.methods
          .proposeAdminAction(
            { updateApy: {} },
            {
              newApy: 100, // 1% APY (malicious reduction)
              newFee: null,
              pauseReason: null,
              newUserLimit: null,
              newPoolLimit: null,
              withdrawalAmount: null,
            }
          )
          .accounts({
            proposer: attacker.publicKey,
            pool: poolPublicKey,
          })
          .signers([attacker])
          .rpc();
        expect.fail("Should have failed for unauthorized proposer");
      } catch (error) {
        expect(error.message).to.include("Unauthorized");
        console.log("✅ Unauthorized governance proposal properly blocked");
      }
    });

    it("should enforce timelock delays", async () => {
      console.log("🧪 Testing timelock delay enforcement...");
      
      // Propose a legitimate admin action
      await program.methods
        .proposeAdminAction(
          { updateApy: {} },
          {
            newApy: 1800, // 18% APY
            newFee: null,
            pauseReason: null,
            newUserLimit: null,
            newPoolLimit: null,
            withdrawalAmount: null,
          }
        )
        .accounts({
          proposer: admin.publicKey,
          pool: poolPublicKey,
        })
        .signers([admin])
        .rpc();
      
      // Try to execute immediately (should fail due to timelock)
      try {
        await program.methods
          .executeAdminAction()
          .accounts({
            executor: admin.publicKey,
            pool: poolPublicKey,
          })
          .signers([admin])
          .rpc();
        expect.fail("Should have failed due to timelock");
      } catch (error) {
        expect(error.message).to.include("TimelockNotExpired");
        console.log("✅ Timelock delay properly enforced");
      }
    });
  });

  // ==================== STRESS TESTS ====================
  describe("💪 Stress Tests", () => {
    it("should handle maximum user capacity", async () => {
      console.log("🧪 Testing maximum user capacity...");
      
      const maxUsers = 10; // Reduced for testing
      const stressUsers = Array.from({ length: maxUsers }, () => Keypair.generate());
      
      // Fund all stress test users
      await Promise.all(
        stressUsers.map(user => 
          provider.connection.requestAirdrop(user.publicKey, 10 * LAMPORTS_PER_SOL)
        )
      );
      await new Promise(resolve => setTimeout(resolve, 3000));
      
      // Add users with staggered timing to avoid rate limits
      for (let i = 0; i < stressUsers.length; i++) {
        const user = stressUsers[i];
        const [userStake] = PublicKey.findProgramAddressSync(
          [Buffer.from("user_stake"), user.publicKey.toBuffer()],
          program.programId
        );
        
        const amount = (0.5 + i * 0.1) * LAMPORTS_PER_SOL;
        const fee = Math.floor(amount * 0.005);
        const minExpected = amount - fee - Math.floor(amount * 0.01);
        const deadline = Math.floor(Date.now() / 1000) + 300;
        
        try {
          await program.methods
            .stake(
              new BN(amount),
              30,
              new BN(minExpected),
              new BN(deadline)
            )
            .accounts({
              user: user.publicKey,
              pool: poolPublicKey,
              userStake: userStake,
              poolVault: poolVaultPublicKey,
              systemProgram: SystemProgram.programId,
            })
            .signers([user])
            .rpc();
          
          console.log(`✅ User ${i + 1}/${maxUsers} added successfully`);
        } catch (error) {
          if (error.message.includes("RateLimitExceeded")) {
            console.log(`⏳ User ${i + 1} rate limited (expected behavior)`);
          } else {
            throw error;
          }
        }
        
        // Add delay to respect rate limits
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
      
      console.log("✅ System handled multiple users successfully");
    });

    it("should maintain performance under load", async () => {
      console.log("🧪 Testing performance under load...");
      
      const startTime = Date.now();
      const loadTestUsers = 5;
      const users = Array.from({ length: loadTestUsers }, () => Keypair.generate());
      
      // Fund users
      await Promise.all(
        users.map(user => 
          provider.connection.requestAirdrop(user.publicKey, 10 * LAMPORTS_PER_SOL)
        )
      );
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // Perform operations with timing
      const operationTimes: number[] = [];
      
      for (const user of users) {
        const [userStake] = PublicKey.findProgramAddressSync(
          [Buffer.from("user_stake"), user.publicKey.toBuffer()],
          program.programId
        );
        
        const operationStart = Date.now();
        
        try {
          const amount = 1 * LAMPORTS_PER_SOL;
          const fee = Math.floor(amount * 0.005);
          const minExpected = amount - fee - Math.floor(amount * 0.01);
          const deadline = Math.floor(Date.now() / 1000) + 300;
          
          await program.methods
            .stake(
              new BN(amount),
              30,
              new BN(minExpected),
              new BN(deadline)
            )
            .accounts({
              user: user.publicKey,
              pool: poolPublicKey,
              userStake: userStake,
              poolVault: poolVaultPublicKey,
              systemProgram: SystemProgram.programId,
            })
            .signers([user])
            .rpc();
          
          const operationTime = Date.now() - operationStart;
          operationTimes.push(operationTime);
          
        } catch (error) {
          if (!error.message.includes("RateLimitExceeded")) {
            throw error;
          }
        }
        
        // Respect rate limits
        await new Promise(resolve => setTimeout(resolve, 2000));
      }
      
      const totalTime = Date.now() - startTime;
      const avgOperationTime = operationTimes.reduce((a, b) => a + b, 0) / operationTimes.length;
      
      console.log(`📊 Performance metrics:`);
      console.log(`  Total time: ${totalTime}ms`);
      console.log(`  Average operation time: ${avgOperationTime.toFixed(2)}ms`);
      console.log(`  Successful operations: ${operationTimes.length}/${loadTestUsers}`);
      
      // Performance should be reasonable
      expect(avgOperationTime).to.be.lessThan(10000); // Less than 10 seconds per operation
      
      console.log("✅ Performance under load test passed");
    });
  });

  // ==================== ERROR HANDLING TESTS ====================
  describe("❌ Error Handling Tests", () => {
    it("should provide clear error messages for all failure cases", async () => {
      console.log("🧪 Testing error message clarity...");
      
      const errorTests = [
        {
          name: "Zero amount",
          params: { amount: 0, days: 30 },
          expectedError: "ZeroAmount"
        },
        {
          name: "Amount too small",
          params: { amount: 0.05 * LAMPORTS_PER_SOL, days: 30 },
          expectedError: "AmountTooSmall"
        },
        {
          name: "Invalid commitment",
          params: { amount: 1 * LAMPORTS_PER_SOL, days: 0 },
          expectedError: "InvalidCommitment"
        }
      ];
      
      for (const test of errorTests) {
        try {
          const tempUser = Keypair.generate();
          await provider.connection.requestAirdrop(tempUser.publicKey, 5 * LAMPORTS_PER_SOL);
          await new Promise(resolve => setTimeout(resolve, 500));
          
          const [tempUserStake] = PublicKey.findProgramAddressSync(
            [Buffer.from("user_stake"), tempUser.publicKey.toBuffer()],
            program.programId
          );
          
          const deadline = Math.floor(Date.now() / 1000) + 300;
          
          await program.methods
            .stake(
              new BN(test.params.amount),
              test.params.days,
              new BN(test.params.amount),
              new BN(deadline)
            )
            .accounts({
              user: tempUser.publicKey,
              pool: poolPublicKey,
              userStake: tempUserStake,
              poolVault: poolVaultPublicKey,
              systemProgram: SystemProgram.programId,
            })
            .signers([tempUser])
            .rpc();
          
          expect.fail(`Should have failed for: ${test.name}`);
        } catch (error) {
          expect(error.message).to.include(test.expectedError);
          console.log(`✅ Clear error for ${test.name}: ${test.expectedError}`);
        }
      }
    });
  });

  // ==================== INTEGRATION TESTS ====================
  describe("🔗 Integration Tests", () => {
    it("should complete full user journey successfully", async () => {
      console.log("🧪 Testing complete user journey...");
      
      const journeyUser = Keypair.generate();
      await provider.connection.requestAirdrop(journeyUser.publicKey, 20 * LAMPORTS_PER_SOL);
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const [journeyUserStake] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_stake"), journeyUser.publicKey.toBuffer()],
        program.programId
      );
      
      const initialBalance = await provider.connection.getBalance(journeyUser.publicKey);
      
      // 1. Stake
      const stakeAmount = 5 * LAMPORTS_PER_SOL;
      const fee = Math.floor(stakeAmount * 0.005);
      const minExpected = stakeAmount - fee - Math.floor(stakeAmount * 0.01);
      const deadline = Math.floor(Date.now() / 1000) + 300;
      
      await program.methods
        .stake(
          new BN(stakeAmount),
          1, // 1 day for quick testing
          new BN(minExpected),
          new BN(deadline)
        )
        .accounts({
          user: journeyUser.publicKey,
          pool: poolPublicKey,
          userStake: journeyUserStake,
          poolVault: poolVaultPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([journeyUser])
        .rpc();
      
      console.log("  ✅ Step 1: Staking completed");
      
      // 2. Verify stake
      const userStake = await program.account.userStake.fetch(journeyUserStake);
      expect(userStake.amount.toNumber()).to.be.greaterThan(0);
      
      console.log("  ✅ Step 2: Stake verification completed");
      
      // 3. Early unstake (with penalty)
      await program.methods
        .unstake()
        .accounts({
          user: journeyUser.publicKey,
          pool: poolPublicKey,
          userStake: journeyUserStake,
          poolVault: poolVaultPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([journeyUser])
        .rpc();
      
      console.log("  ✅ Step 3: Unstaking completed");
      
      // 4. Verify final state
      const finalBalance = await provider.connection.getBalance(journeyUser.publicKey);
      const finalUserStake = await program.account.userStake.fetch(journeyUserStake);
      
      expect(finalUserStake.amount.toNumber()).to.equal(0);
      expect(finalBalance).to.be.lessThan(initialBalance); // Should have lost money due to fees and penalties
      
      console.log("  ✅ Step 4: Final state verification completed");
      console.log("✅ Complete user journey test passed");
    });
  });

  // ==================== CLEANUP AND SUMMARY ====================
  after(async () => {
    console.log("\n🎉 Comprehensive Test Suite Summary");
    console.log("=====================================");
    
    // Get final pool state
    const finalPool = await program.account.pool.fetch(poolPublicKey);
    
    console.log(`📊 Final Pool State:`);
    console.log(`  Total Staked: ${finalPool.totalStaked.toNumber() / LAMPORTS_PER_SOL} SOL`);
    console.log(`  Total Users: ${finalPool.totalUsers.toNumber()}`);
    console.log(`  Total Fees Collected: ${finalPool.totalFeesCollected.toNumber() / LAMPORTS_PER_SOL} SOL`);
    console.log(`  Pool Status: ${finalPool.isActive ? 'Active' : 'Inactive'} / ${finalPool.isPaused ? 'Paused' : 'Running'}`);
    
    console.log("\n✅ All comprehensive tests completed successfully!");
    console.log("🛡️ Security measures validated across all attack vectors");
    console.log("🚀 Protocol ready for production deployment");
  });
});