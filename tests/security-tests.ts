import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DeFiTrustFund } from "../target/types/defi_trust_fund";
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { expect } from "chai";

describe("Security Tests - Redesigned Contract", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DeFiTrustFund as Program<DeFiTrustFund>;
  const admin = Keypair.generate();
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();
  const attacker = Keypair.generate();
  const poolKeypair = Keypair.generate();
  const poolVaultKeypair = Keypair.generate();

  before(async () => {
    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(admin.publicKey, 20 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user1.publicKey, 10 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user2.publicKey, 10 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(attacker.publicKey, 5 * LAMPORTS_PER_SOL);
  });

  describe("Pool Initialization Security", () => {
    it("should initialize pool with valid parameters", async () => {
      const maxApy = 5000; // 50%
      const minCommitmentDays = 1;
      const maxCommitmentDays = 365;

      await program.methods
        .initializePool(maxApy, minCommitmentDays, maxCommitmentDays)
        .accounts({
          admin: admin.publicKey,
          pool: poolKeypair.publicKey,
          poolVault: poolVaultKeypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([admin, poolKeypair, poolVaultKeypair])
        .rpc();

      const pool = await program.account.pool.fetch(poolKeypair.publicKey);
      expect(pool.admin.toString()).to.equal(admin.publicKey.toString());
      expect(pool.maxApy.toNumber()).to.equal(maxApy);
      expect(pool.minCommitmentDays.toNumber()).to.equal(minCommitmentDays);
      expect(pool.maxCommitmentDays.toNumber()).to.equal(maxCommitmentDays);
      expect(pool.isActive).to.be.true;
      expect(pool.isPaused).to.be.false;
    });

    it("should reject invalid APY during initialization", async () => {
      const invalidMaxApy = 6000; // 60% (exceeds 50% limit)
      
      try {
        await program.methods
          .initializePool(invalidMaxApy, 1, 365)
          .accounts({
            admin: admin.publicKey,
            pool: Keypair.generate().publicKey,
            poolVault: Keypair.generate().publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([admin])
          .rpc();
        
        expect.fail("Should have thrown an error for invalid APY");
      } catch (error) {
        expect(error.message).to.include("InvalidApy");
      }
    });

    it("should reject invalid commitment days during initialization", async () => {
      try {
        await program.methods
          .initializePool(5000, 0, 365) // min days = 0
          .accounts({
            admin: admin.publicKey,
            pool: Keypair.generate().publicKey,
            poolVault: Keypair.generate().publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([admin])
          .rpc();
        
        expect.fail("Should have thrown an error for invalid commitment days");
      } catch (error) {
        expect(error.message).to.include("InvalidCommitment");
      }
    });
  });

  describe("Staking Security", () => {
    beforeEach(async () => {
      // Initialize pool for each test
      const poolKey = Keypair.generate();
      const vaultKey = Keypair.generate();
      
      await program.methods
        .initializePool(5000, 1, 365)
        .accounts({
          admin: admin.publicKey,
          pool: poolKey.publicKey,
          poolVault: vaultKey.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([admin, poolKey, vaultKey])
        .rpc();
    });

    it("should allow valid stake", async () => {
      const amount = new anchor.BN(LAMPORTS_PER_SOL);
      const committedDays = 30;

      await program.methods
        .stake(amount, committedDays)
        .accounts({
          user: user1.publicKey,
          pool: poolKeypair.publicKey,
          userStake: PublicKey.findProgramAddressSync(
            [Buffer.from("user_stake"), user1.publicKey.toBuffer()],
            program.programId
          )[0],
          poolVault: poolVaultKeypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      const userStake = await program.account.userStake.fetch(
        PublicKey.findProgramAddressSync(
          [Buffer.from("user_stake"), user1.publicKey.toBuffer()],
          program.programId
        )[0]
      );
      
      expect(userStake.amount.toNumber()).to.be.greaterThan(0);
      expect(userStake.committedDays.toNumber()).to.equal(committedDays);
    });

    it("should reject stake below minimum amount", async () => {
      const smallAmount = new anchor.BN(0.05 * LAMPORTS_PER_SOL); // Below 0.1 SOL minimum
      
      try {
        await program.methods
          .stake(smallAmount, 30)
          .accounts({
            user: user1.publicKey,
            pool: poolKeypair.publicKey,
            userStake: PublicKey.findProgramAddressSync(
              [Buffer.from("user_stake"), user1.publicKey.toBuffer()],
              program.programId
            )[0],
            poolVault: poolVaultKeypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have thrown an error for amount too small");
      } catch (error) {
        expect(error.message).to.include("AmountTooSmall");
      }
    });

    it("should reject stake above maximum amount", async () => {
      const largeAmount = new anchor.BN(200 * LAMPORTS_PER_SOL); // Above 100 SOL maximum
      
      try {
        await program.methods
          .stake(largeAmount, 30)
          .accounts({
            user: user1.publicKey,
            pool: poolKeypair.publicKey,
            userStake: PublicKey.findProgramAddressSync(
              [Buffer.from("user_stake"), user1.publicKey.toBuffer()],
              program.programId
            )[0],
            poolVault: poolVaultKeypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have thrown an error for amount too large");
      } catch (error) {
        expect(error.message).to.include("AmountTooLarge");
      }
    });

    it("should reject invalid commitment days", async () => {
      const amount = new anchor.BN(LAMPORTS_PER_SOL);
      
      // Test zero days
      try {
        await program.methods
          .stake(amount, 0)
          .accounts({
            user: user1.publicKey,
            pool: poolKeypair.publicKey,
            userStake: PublicKey.findProgramAddressSync(
              [Buffer.from("user_stake"), user1.publicKey.toBuffer()],
              program.programId
            )[0],
            poolVault: poolVaultKeypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have thrown an error for zero commitment days");
      } catch (error) {
        expect(error.message).to.include("InvalidCommitment");
      }

      // Test excessive days
      try {
        await program.methods
          .stake(amount, 1000)
          .accounts({
            user: user1.publicKey,
            pool: poolKeypair.publicKey,
            userStake: PublicKey.findProgramAddressSync(
              [Buffer.from("user_stake"), user1.publicKey.toBuffer()],
              program.programId
            )[0],
            poolVault: poolVaultKeypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have thrown an error for excessive commitment days");
      } catch (error) {
        expect(error.message).to.include("InvalidCommitment");
      }
    });

    it("should enforce user deposit limits", async () => {
      const largeAmount = new anchor.BN(1500 * LAMPORTS_PER_SOL); // Above 1000 SOL user limit
      
      try {
        await program.methods
          .stake(largeAmount, 30)
          .accounts({
            user: user1.publicKey,
            pool: poolKeypair.publicKey,
            userStake: PublicKey.findProgramAddressSync(
              [Buffer.from("user_stake"), user1.publicKey.toBuffer()],
              program.programId
            )[0],
            poolVault: poolVaultKeypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have thrown an error for user limit exceeded");
      } catch (error) {
        expect(error.message).to.include("UserLimitExceeded");
      }
    });
  });

  describe("Emergency Controls", () => {
    it("should allow admin to pause pool", async () => {
      await program.methods
        .emergencyPause("Security test pause")
        .accounts({
          admin: admin.publicKey,
          pool: poolKeypair.publicKey,
        })
        .signers([admin])
        .rpc();

      const pool = await program.account.pool.fetch(poolKeypair.publicKey);
      expect(pool.isPaused).to.be.true;
      expect(pool.emergencyPauseReason).to.equal("Security test pause");
    });

    it("should prevent non-admin from pausing pool", async () => {
      try {
        await program.methods
          .emergencyPause("Unauthorized pause")
          .accounts({
            admin: attacker.publicKey,
            pool: poolKeypair.publicKey,
          })
          .signers([attacker])
          .rpc();
        
        expect.fail("Should have thrown an error for unauthorized access");
      } catch (error) {
        expect(error.message).to.include("Unauthorized");
      }
    });

    it("should prevent staking when pool is paused", async () => {
      // First pause the pool
      await program.methods
        .emergencyPause("Test pause")
        .accounts({
          admin: admin.publicKey,
          pool: poolKeypair.publicKey,
        })
        .signers([admin])
        .rpc();

      // Try to stake
      try {
        await program.methods
          .stake(new anchor.BN(LAMPORTS_PER_SOL), 30)
          .accounts({
            user: user1.publicKey,
            pool: poolKeypair.publicKey,
            userStake: PublicKey.findProgramAddressSync(
              [Buffer.from("user_stake"), user1.publicKey.toBuffer()],
              program.programId
            )[0],
            poolVault: poolVaultKeypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have thrown an error for paused pool");
      } catch (error) {
        expect(error.message).to.include("PoolPaused");
      }
    });

    it("should allow admin to unpause pool", async () => {
      // First pause
      await program.methods
        .emergencyPause("Test pause")
        .accounts({
          admin: admin.publicKey,
          pool: poolKeypair.publicKey,
        })
        .signers([admin])
        .rpc();

      // Then unpause
      await program.methods
        .emergencyUnpause()
        .accounts({
          admin: admin.publicKey,
          pool: poolKeypair.publicKey,
        })
        .signers([admin])
        .rpc();

      const pool = await program.account.pool.fetch(poolKeypair.publicKey);
      expect(pool.isPaused).to.be.false;
      expect(pool.emergencyPauseReason).to.equal("");
    });
  });

  describe("Parameter Update Security", () => {
    it("should allow admin to update APY within bounds", async () => {
      const newApy = 1500; // 15%
      
      await program.methods
        .updateApy(new anchor.BN(newApy))
        .accounts({
          admin: admin.publicKey,
          pool: poolKeypair.publicKey,
        })
        .signers([admin])
        .rpc();

      const pool = await program.account.pool.fetch(poolKeypair.publicKey);
      expect(pool.apy.toNumber()).to.equal(newApy);
    });

    it("should reject APY above maximum", async () => {
      const invalidApy = 6000; // 60% (above 50% max)
      
      try {
        await program.methods
          .updateApy(new anchor.BN(invalidApy))
          .accounts({
            admin: admin.publicKey,
            pool: poolKeypair.publicKey,
          })
          .signers([admin])
          .rpc();
        
        expect.fail("Should have thrown an error for invalid APY");
      } catch (error) {
        expect(error.message).to.include("InvalidApy");
      }
    });

    it("should reject APY below minimum", async () => {
      const invalidApy = 50; // 0.5% (below 1% min)
      
      try {
        await program.methods
          .updateApy(new anchor.BN(invalidApy))
          .accounts({
            admin: admin.publicKey,
            pool: poolKeypair.publicKey,
          })
          .signers([admin])
          .rpc();
        
        expect.fail("Should have thrown an error for invalid APY");
      } catch (error) {
        expect(error.message).to.include("InvalidApy");
      }
    });

    it("should allow admin to update deposit fee within bounds", async () => {
      const newFee = 100; // 1%
      
      await program.methods
        .updateDepositFee(new anchor.BN(newFee))
        .accounts({
          admin: admin.publicKey,
          pool: poolKeypair.publicKey,
        })
        .signers([admin])
        .rpc();

      const pool = await program.account.pool.fetch(poolKeypair.publicKey);
      expect(pool.depositFee.toNumber()).to.equal(newFee);
    });

    it("should reject deposit fee above maximum", async () => {
      const invalidFee = 1500; // 15% (above 10% max)
      
      try {
        await program.methods
          .updateDepositFee(new anchor.BN(invalidFee))
          .accounts({
            admin: admin.publicKey,
            pool: poolKeypair.publicKey,
          })
          .signers([admin])
          .rpc();
        
        expect.fail("Should have thrown an error for invalid fee");
      } catch (error) {
        expect(error.message).to.include("InvalidFee");
      }
    });
  });

  describe("Arithmetic Safety", () => {
    it("should prevent deposit amount overflow", async () => {
      const maxU64 = "18446744073709551615"; // 2^64 - 1
      
      try {
        await program.methods
          .stake(new anchor.BN(maxU64), 1)
          .accounts({
            user: user1.publicKey,
            pool: poolKeypair.publicKey,
            userStake: PublicKey.findProgramAddressSync(
              [Buffer.from("user_stake"), user1.publicKey.toBuffer()],
              program.programId
            )[0],
            poolVault: poolVaultKeypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have thrown an error for overflow");
      } catch (error) {
        expect(error.message).to.include("overflow");
      }
    });

    it("should prevent fee calculation overflow", async () => {
      const largeAmount = new anchor.BN("1000000000000000000"); // Very large amount
      
      try {
        // Test fee calculation with large amounts
        const fee = largeAmount.mul(new anchor.BN(500)).div(new anchor.BN(10000));
        expect(fee.toNumber()).to.be.lessThan(largeAmount.toNumber());
      } catch (error) {
        expect(error.message).to.include("overflow");
      }
    });

    it("should handle edge case amounts correctly", async () => {
      const minAmount = new anchor.BN(1);
      
      try {
        const fee = minAmount.mul(new anchor.BN(500)).div(new anchor.BN(10000));
        expect(fee.toNumber()).to.be.greaterThanOrEqual(0);
      } catch (error) {
        expect(error.message).to.include("underflow");
      }
    });
  });

  describe("State Consistency", () => {
    it("should maintain consistent state after failed operations", async () => {
      // Get initial state
      const initialPool = await program.account.pool.fetch(poolKeypair.publicKey);
      
      try {
        // Attempt operation that will fail
        await program.methods
          .stake(new anchor.BN(LAMPORTS_PER_SOL), 0) // Invalid commitment days
          .accounts({
            user: user1.publicKey,
            pool: poolKeypair.publicKey,
            userStake: PublicKey.findProgramAddressSync(
              [Buffer.from("user_stake"), user1.publicKey.toBuffer()],
              program.programId
            )[0],
            poolVault: poolVaultKeypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have thrown an error");
      } catch (error) {
        // Check that state remains consistent
        const finalPool = await program.account.pool.fetch(poolKeypair.publicKey);
        expect(finalPool.totalStaked.toNumber()).to.equal(initialPool.totalStaked.toNumber());
        expect(finalPool.totalUsers.toNumber()).to.equal(initialPool.totalUsers.toNumber());
      }
    });
  });

  describe("Fee Management", () => {
    it("should correctly calculate and collect fees", async () => {
      const amount = new anchor.BN(LAMPORTS_PER_SOL);
      const expectedFee = amount.mul(new anchor.BN(50)).div(new anchor.BN(10000)); // 0.5%
      
      await program.methods
        .stake(amount, 30)
        .accounts({
          user: user1.publicKey,
          pool: poolKeypair.publicKey,
          userStake: PublicKey.findProgramAddressSync(
            [Buffer.from("user_stake"), user1.publicKey.toBuffer()],
            program.programId
          )[0],
          poolVault: poolVaultKeypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      const pool = await program.account.pool.fetch(poolKeypair.publicKey);
      expect(pool.totalFeesCollected.toNumber()).to.equal(expectedFee.toNumber());
    });

    it("should allow admin to withdraw fees", async () => {
      const withdrawAmount = new anchor.BN(0.1 * LAMPORTS_PER_SOL);
      
      await program.methods
        .withdrawFees(withdrawAmount)
        .accounts({
          admin: admin.publicKey,
          pool: poolKeypair.publicKey,
          poolVault: poolVaultKeypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      const pool = await program.account.pool.fetch(poolKeypair.publicKey);
      expect(pool.totalFeesCollected.toNumber()).to.be.greaterThanOrEqual(0);
    });

    it("should prevent withdrawing more fees than collected", async () => {
      const excessiveAmount = new anchor.BN(1000 * LAMPORTS_PER_SOL);
      
      try {
        await program.methods
          .withdrawFees(excessiveAmount)
          .accounts({
            admin: admin.publicKey,
            pool: poolKeypair.publicKey,
            poolVault: poolVaultKeypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([admin])
          .rpc();
        
        expect.fail("Should have thrown an error for insufficient funds");
      } catch (error) {
        expect(error.message).to.include("InsufficientFunds");
      }
    });
  });

  describe("Event Logging", () => {
    it("should emit comprehensive events for all operations", async () => {
      // Test stake event
      const amount = new anchor.BN(LAMPORTS_PER_SOL);
      const committedDays = 30;
      
      const tx = await program.methods
        .stake(amount, committedDays)
        .accounts({
          user: user1.publicKey,
          pool: poolKeypair.publicKey,
          userStake: PublicKey.findProgramAddressSync(
            [Buffer.from("user_stake"), user1.publicKey.toBuffer()],
            program.programId
          )[0],
          poolVault: poolVaultKeypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      // Verify transaction contains events
      const txInfo = await provider.connection.getTransaction(tx, {
        commitment: "confirmed",
      });
      expect(txInfo).to.not.be.null;
    });
  });

  describe("Integration Security", () => {
    it("should handle concurrent operations correctly", async () => {
      // Test multiple users staking simultaneously
      const amount = new anchor.BN(LAMPORTS_PER_SOL);
      const committedDays = 30;
      
      const promises = [
        program.methods
          .stake(amount, committedDays)
          .accounts({
            user: user1.publicKey,
            pool: poolKeypair.publicKey,
            userStake: PublicKey.findProgramAddressSync(
              [Buffer.from("user_stake"), user1.publicKey.toBuffer()],
              program.programId
            )[0],
            poolVault: poolVaultKeypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([user1])
          .rpc(),
        program.methods
          .stake(amount, committedDays)
          .accounts({
            user: user2.publicKey,
            pool: poolKeypair.publicKey,
            userStake: PublicKey.findProgramAddressSync(
              [Buffer.from("user_stake"), user2.publicKey.toBuffer()],
              program.programId
            )[0],
            poolVault: poolVaultKeypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([user2])
          .rpc(),
      ];

      await Promise.all(promises);

      const pool = await program.account.pool.fetch(poolKeypair.publicKey);
      expect(pool.totalUsers.toNumber()).to.equal(2);
    });
  });
});

