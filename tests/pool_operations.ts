import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Multistake } from "../target/types/multistake";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { assert } from "chai";

describe("Pool Operations Tests", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Multistake as Program<Multistake>;

  // Test accounts
  let admin: Keypair;
  let payer: Keypair;
  let pool: Keypair;
  let mainTokenMint: PublicKey;
  let poolAuthority: PublicKey;
  let poolVault: PublicKey;

  // User accounts for testing
  let user: Keypair;
  let userMainTokenAccount: PublicKey;

  // LP mints for testing
  let lpMint1: Keypair;
  let lpMint2: Keypair;
  let lpMint3: Keypair;

  before(async () => {
    // Initialize test accounts
    admin = Keypair.generate();
    payer = Keypair.generate();
    pool = Keypair.generate();
    user = Keypair.generate();

    // Initialize LP mint keypairs
    lpMint1 = Keypair.generate();
    lpMint2 = Keypair.generate();
    lpMint3 = Keypair.generate();

    // Airdrop SOL to test accounts
    const airdropAmount = 10 * anchor.web3.LAMPORTS_PER_SOL;

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(admin.publicKey, airdropAmount)
    );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, airdropAmount)
    );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(user.publicKey, airdropAmount)
    );

    // Create main token mint
    mainTokenMint = await createMint(
      provider.connection,
      payer,
      admin.publicKey,
      null,
      9
    );

    console.log("Main token mint created:", mainTokenMint.toBase58());

    // Create user's main token account and mint some tokens
    const userTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      mainTokenMint,
      user.publicKey
    );
    userMainTokenAccount = userTokenAccount.address;

    await mintTo(
      provider.connection,
      payer,
      mainTokenMint,
      userMainTokenAccount,
      admin,
      1_000_000_000_000 // 1000 tokens with 9 decimals
    );

    console.log("User main token account created and funded");
  });

  it("Creates a staking pool", async () => {
    // Derive PDAs
    [poolAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("anyswap_authority"), pool.publicKey.toBuffer()],
      program.programId
    );

    [poolVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool_vault"), pool.publicKey.toBuffer()],
      program.programId
    );

    console.log("Pool:", pool.publicKey.toBase58());
    console.log("Pool Authority:", poolAuthority.toBase58());
    console.log("Pool Vault:", poolVault.toBase58());

    // Create pool account
    const poolSize = 24704; // Pool account size (24696 + 8 discriminator)
    const lamports = await provider.connection.getMinimumBalanceForRentExemption(poolSize);

    const createPoolAccountIx = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: pool.publicKey,
      lamports,
      space: poolSize,
      programId: program.programId,
    });

    // Create pool
    const tx = await program.methods
      .createPool(
        new anchor.BN(3), // fee_numerator: 0.3%
        new anchor.BN(1000) // fee_denominator
      )
      .accounts({
        pool: pool.publicKey,
        poolAuthority: poolAuthority,
        mainTokenMint: mainTokenMint,
        poolVault: poolVault,
        admin: admin.publicKey,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .preInstructions([createPoolAccountIx])
      .signers([payer, pool, admin])
      .rpc();

    console.log("Create pool transaction:", tx);

    // Verify pool was created
    const poolAccount = await program.account.anySwapPool.fetch(pool.publicKey);
    assert.equal(poolAccount.tokenCount, 0);
    assert.equal(poolAccount.incrementCount, 0);
    assert.equal(poolAccount.admin.toBase58(), admin.publicKey.toBase58());
    assert.equal(poolAccount.poolVault.toBase58(), poolVault.toBase58());
    assert.equal(poolAccount.poolMint.toBase58(), mainTokenMint.toBase58());
    assert.equal(poolAccount.feeNumerator.toNumber(), 3);
    assert.equal(poolAccount.feeDenominator.toNumber(), 1000);

    console.log("✅ Pool created successfully");
  });

  it("Adds first staking type to pool", async () => {
    console.log("LP Mint 1:", lpMint1.publicKey.toBase58());

    // Add token to pool
    const tx = await program.methods
      .addTokenToPool()
      .accounts({
        pool: pool.publicKey,
        poolAuthority: poolAuthority,
        lpMint: lpMint1.publicKey,
        admin: admin.publicKey,
        payer: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([admin, payer, lpMint1])
      .rpc();

    console.log("Add token transaction:", tx);

    // Verify token was added
    const poolAccount = await program.account.anySwapPool.fetch(pool.publicKey);
    assert.equal(poolAccount.tokenCount, 1);
    assert.equal(poolAccount.incrementCount, 1);

    console.log("✅ First staking type added successfully");
  });

  it("Adds second staking type to pool", async () => {
    console.log("LP Mint 2:", lpMint2.publicKey.toBase58());

    const tx = await program.methods
      .addTokenToPool()
      .accounts({
        pool: pool.publicKey,
        poolAuthority: poolAuthority,
        lpMint: lpMint2.publicKey,
        admin: admin.publicKey,
        payer: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([admin, payer, lpMint2])
      .rpc();

    console.log("Add second token transaction:", tx);

    const poolAccount = await program.account.anySwapPool.fetch(pool.publicKey);
    assert.equal(poolAccount.tokenCount, 2);
    assert.equal(poolAccount.incrementCount, 2);

    console.log("✅ Second staking type added successfully");
  });

  it("Removes first staking type from pool", async () => {
    console.log("Removing LP Mint 1:", lpMint1.publicKey.toBase58());

    const tx = await program.methods
      .removeTokenFromPool()
      .accounts({
        pool: pool.publicKey,
        lpMint: lpMint1.publicKey,
        admin: admin.publicKey,
      })
      .signers([admin])
      .rpc();

    console.log("Remove token transaction:", tx);

    const poolAccount = await program.account.anySwapPool.fetch(pool.publicKey);
    assert.equal(poolAccount.tokenCount, 1);
    assert.equal(poolAccount.incrementCount, 2); // Should NOT change

    console.log("✅ First staking type removed successfully");
  });

  it("Adds third staking type after removal (no address conflict)", async () => {
    console.log("LP Mint 3:", lpMint3.publicKey.toBase58());

    const tx = await program.methods
      .addTokenToPool()
      .accounts({
        pool: pool.publicKey,
        poolAuthority: poolAuthority,
        lpMint: lpMint3.publicKey,
        admin: admin.publicKey,
        payer: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([admin, payer, lpMint3])
      .rpc();

    console.log("Add third token transaction:", tx);

    const poolAccount = await program.account.anySwapPool.fetch(pool.publicKey);
    assert.equal(poolAccount.tokenCount, 2);
    assert.equal(poolAccount.incrementCount, 3);

    console.log("✅ Third staking type added successfully");
  });
});

