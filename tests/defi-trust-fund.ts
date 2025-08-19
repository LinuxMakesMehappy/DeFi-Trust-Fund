import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DefiTrustFund } from "../target/types/defi_trust_fund";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";

describe("defi-trust-fund", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DefiTrustFund as Program<DefiTrustFund>;
  
  // Test accounts
  const admin = Keypair.generate();
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();
  
  // Program accounts
  let pool: PublicKey;
  let poolVault: PublicKey;
  let userStake: PublicKey;
  
  before(async () => {
    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(admin.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user1.publicKey, 5 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user2.publicKey, 5 * anchor.web3.LAMPORTS_PER_SOL);
    
    // Wait for airdrop confirmation
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Derive PDAs
    [pool] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool")],
      program.programId
    );
    
    [poolVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool_vault")],
      program.programId
    );
    
    [userStake] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_stake"), user1.publicKey.toBuffer()],
      program.programId
    );
  });

  it("Initializes the pool", async () => {
    const maxApy = 5000; // 50% APY
    const minCommitmentDays = 1;
    const maxCommitmentDays = 365;

    await program.methods
      .initializePool(maxApy, minCommitmentDays, maxCommitmentDays)
      .accounts({
        admin: admin.publicKey,
        pool: pool,
        poolVault: poolVault,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([admin])
      .rpc();

    const poolAccount = await program.account.pool.fetch(pool);
    expect(poolAccount.admin.toString()).to.equal(admin.publicKey.toString());
    expect(poolAccount.maxApy.toNumber()).to.equal(maxApy);
    expect(poolAccount.minCommitmentDays.toNumber()).to.equal(minCommitmentDays);
    expect(poolAccount.maxCommitmentDays.toNumber()).to.equal(maxCommitmentDays);
  });

  it("Allows users to stake", async () => {
    const amount = new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL); // 1 SOL
    const committedDays = 30;

    await program.methods
      .stake(amount, committedDays)
      .accounts({
        user: user1.publicKey,
        pool: pool,
        poolVault: poolVault,
        userStake: userStake,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([user1])
      .rpc();

    const userStakeAccount = await program.account.userStake.fetch(userStake);
    expect(userStakeAccount.user.toString()).to.equal(user1.publicKey.toString());
    expect(userStakeAccount.amount.toNumber()).to.equal(amount.toNumber());
    expect(userStakeAccount.committedDays.toNumber()).to.equal(committedDays);
  });

  it("Allows users to claim yields", async () => {
    // Wait some time to accumulate yields
    await new Promise(resolve => setTimeout(resolve, 1000));

    await program.methods
      .claimYields()
      .accounts({
        user: user1.publicKey,
        pool: pool,
        poolVault: poolVault,
        userStake: userStake,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc();

    // Verify yields were claimed
    const userStakeAccount = await program.account.userStake.fetch(userStake);
    expect(userStakeAccount.lastClaimTimestamp.toNumber()).to.be.greaterThan(0);
  });

  it("Allows users to unstake", async () => {
    await program.methods
      .unstake()
      .accounts({
        user: user1.publicKey,
        pool: pool,
        poolVault: poolVault,
        userStake: userStake,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc();

    // Verify unstaking
    const userStakeAccount = await program.account.userStake.fetch(userStake);
    expect(userStakeAccount.amount.toNumber()).to.equal(0);
  });

  it("Allows admin to pause the pool", async () => {
    await program.methods
      .emergencyPause("Testing pause functionality")
      .accounts({
        admin: admin.publicKey,
        pool: pool,
      })
      .signers([admin])
      .rpc();

    const poolAccount = await program.account.pool.fetch(pool);
    expect(poolAccount.isPaused).to.be.true;
  });

  it("Allows admin to unpause the pool", async () => {
    await program.methods
      .emergencyUnpause()
      .accounts({
        admin: admin.publicKey,
        pool: pool,
      })
      .signers([admin])
      .rpc();

    const poolAccount = await program.account.pool.fetch(pool);
    expect(poolAccount.isPaused).to.be.false;
  });
});

