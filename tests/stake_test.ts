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
  getAccount,
} from "@solana/spl-token";
import { assert } from "chai";

describe("Stake and Unstake Tests", () => {
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

  // Users
  let user1: Keypair;
  let user2: Keypair;
  let user1MainTokenAccount: PublicKey;
  let user2MainTokenAccount: PublicKey;

  // LP mints
  let lpMint1: Keypair;
  let lpMint2: Keypair;
  let user1LpTokenAccount: PublicKey;
  let user2LpTokenAccount: PublicKey;

  before(async () => {
    // Initialize accounts
    admin = Keypair.generate();
    payer = Keypair.generate();
    pool = Keypair.generate();
    user1 = Keypair.generate();
    user2 = Keypair.generate();
    lpMint1 = Keypair.generate();
    lpMint2 = Keypair.generate();

    // Airdrop SOL
    const airdropAmount = 10 * anchor.web3.LAMPORTS_PER_SOL;
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(admin.publicKey, airdropAmount)
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, airdropAmount)
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(user1.publicKey, airdropAmount)
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(user2.publicKey, airdropAmount)
    );

    // Create main token mint
    mainTokenMint = await createMint(
      provider.connection,
      payer,
      admin.publicKey,
      null,
      9
    );

    console.log("Main token mint:", mainTokenMint.toBase58());
  });

  it("Creates pool", async () => {
    [poolAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("anyswap_authority"), pool.publicKey.toBuffer()],
      program.programId
    );

    [poolVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool_vault"), pool.publicKey.toBuffer()],
      program.programId
    );

    const poolSize = 24704;
    const lamports = await provider.connection.getMinimumBalanceForRentExemption(poolSize);

    const createPoolAccountIx = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: pool.publicKey,
      lamports,
      space: poolSize,
      programId: program.programId,
    });

    await program.methods
      .createPool(new anchor.BN(3), new anchor.BN(1000))
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

    console.log("✅ Pool created");
  });

  it("User1 adds LP token type", async () => {
    await program.methods
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

    console.log("✅ User1 LP token type added");
  });

  it("User1 stakes 1000 tokens", async () => {
    // Create user1 token accounts
    const user1MainToken = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      mainTokenMint,
      user1.publicKey
    );
    user1MainTokenAccount = user1MainToken.address;

    // Mint 1000 tokens to user1
    await mintTo(
      provider.connection,
      payer,
      mainTokenMint,
      user1MainTokenAccount,
      admin,
      1000_000_000_000 // 1000 tokens
    );

    // Create user1 LP token account
    const user1LpToken = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      lpMint1.publicKey,
      user1.publicKey
    );
    user1LpTokenAccount = user1LpToken.address;

    console.log("User1 main token balance before:", user1MainToken.amount.toString());

    // Stake
    await program.methods
      .stake(0, new anchor.BN(100_000_000_000)) // 100 tokens
      .accounts({
        pool: pool.publicKey,
        poolAuthority: poolAuthority,
        poolVault: poolVault,
        lpMint: lpMint1.publicKey,
        userMainToken: user1MainTokenAccount,
        userLpToken: user1LpTokenAccount,
        user: user1.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user1])
      .rpc();

    // Verify balances
    const user1MainAfter = await getAccount(provider.connection, user1MainTokenAccount);
    const user1LpAfter = await getAccount(provider.connection, user1LpTokenAccount);
    const poolVaultAfter = await getAccount(provider.connection, poolVault);

    console.log("User1 main token after:", user1MainAfter.amount.toString());
    console.log("User1 LP token after:", user1LpAfter.amount.toString());
    console.log("Pool vault after:", poolVaultAfter.amount.toString());

    // With 0.3% fee: 100 tokens staked, 99.7 LP minted, 0.3 fee stays in vault
    assert.equal(user1MainAfter.amount.toString(), "900000000000"); // 1000 - 100
    assert.equal(user1LpAfter.amount.toString(), "99700000000"); // 100 * 0.997 = 99.7 LP
    assert.equal(poolVaultAfter.amount.toString(), "100000000000"); // 100 in vault (including 0.3 fee)

    console.log("✅ User1 staked 100 tokens");
  });

  it("User2 adds LP token type", async () => {
    await program.methods
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

    console.log("✅ User2 LP token type added");
  });

  it("User2 stakes 200 tokens", async () => {
    // Create user2 token accounts
    const user2MainToken = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      mainTokenMint,
      user2.publicKey
    );
    user2MainTokenAccount = user2MainToken.address;

    // Mint 1000 tokens to user2
    await mintTo(
      provider.connection,
      payer,
      mainTokenMint,
      user2MainTokenAccount,
      admin,
      1000_000_000_000
    );

    // Create user2 LP token account
    const user2LpToken = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      lpMint2.publicKey,
      user2.publicKey
    );
    user2LpTokenAccount = user2LpToken.address;

    // Stake
    await program.methods
      .stake(1, new anchor.BN(200_000_000_000)) // 200 tokens
      .accounts({
        pool: pool.publicKey,
        poolAuthority: poolAuthority,
        poolVault: poolVault,
        lpMint: lpMint2.publicKey,
        userMainToken: user2MainTokenAccount,
        userLpToken: user2LpTokenAccount,
        user: user2.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user2])
      .rpc();

    // Verify balances
    const user2MainAfter = await getAccount(provider.connection, user2MainTokenAccount);
    const user2LpAfter = await getAccount(provider.connection, user2LpTokenAccount);
    const poolVaultAfter = await getAccount(provider.connection, poolVault);

    // With 0.3% fee: 200 tokens staked, 199.4 LP minted, 0.6 fee stays in vault
    assert.equal(user2MainAfter.amount.toString(), "800000000000"); // 1000 - 200
    assert.equal(user2LpAfter.amount.toString(), "199400000000"); // 200 * 0.997 = 199.4 LP
    assert.equal(poolVaultAfter.amount.toString(), "300000000000"); // 100 + 200 (including fees)

    console.log("✅ User2 staked 200 tokens");
  });

  it("Admin modifies weights (user1: 2x, user2: 0.5x)", async () => {
    // Modify weights: user1 gets 2x (200_000_000), user2 gets 0.5x (50_000_000)
    await program.methods
      .modifyTokenWeight([
        new anchor.BN(200_000_000), // user1: 2x
        new anchor.BN(50_000_000),  // user2: 0.5x
      ])
      .accounts({
        pool: pool.publicKey,
        admin: admin.publicKey,
      })
      .remainingAccounts([
        { pubkey: lpMint1.publicKey, isSigner: false, isWritable: false },
        { pubkey: lpMint2.publicKey, isSigner: false, isWritable: false },
      ])
      .signers([admin])
      .rpc();

    console.log("✅ Weights modified: user1=2x, user2=0.5x");
  });

  it("User2 unstakes (should get less due to 0.5x weight)", async () => {
    const user2LpBefore = await getAccount(provider.connection, user2LpTokenAccount);
    const poolVaultBefore = await getAccount(provider.connection, poolVault);

    console.log("Pool vault before unstake:", poolVaultBefore.amount.toString());
    console.log("User2 LP before:", user2LpBefore.amount.toString());

    // Unstake all LP tokens (199.4 tokens, not 200)
    await program.methods
      .unstake(1, new anchor.BN(199_400_000_000))
      .accounts({
        pool: pool.publicKey,
        poolAuthority: poolAuthority,
        poolVault: poolVault,
        lpMint: lpMint2.publicKey,
        userLpToken: user2LpTokenAccount,
        userMainToken: user2MainTokenAccount,
        user: user2.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user2])
      .rpc();

    const user2MainAfter = await getAccount(provider.connection, user2MainTokenAccount);
    const user2LpAfter = await getAccount(provider.connection, user2LpTokenAccount);
    const poolVaultAfter = await getAccount(provider.connection, poolVault);

    console.log("User2 main token after:", user2MainAfter.amount.toString());
    console.log("User2 LP after:", user2LpAfter.amount.toString());
    console.log("Pool vault after:", poolVaultAfter.amount.toString());

    // User2 should get less than 200 tokens back due to 0.5x weight
    assert.equal(user2LpAfter.amount.toString(), "0");
    console.log("✅ User2 unstaked with 0.5x weight penalty");
  });

  it("User1 unstakes (should get more due to 2x weight)", async () => {
    const user1LpBefore = await getAccount(provider.connection, user1LpTokenAccount);
    const poolVaultBefore = await getAccount(provider.connection, poolVault);

    console.log("Pool vault before unstake:", poolVaultBefore.amount.toString());
    console.log("User1 LP before:", user1LpBefore.amount.toString());

    // Unstake all LP tokens (99.7 tokens, not 100)
    await program.methods
      .unstake(0, new anchor.BN(99_700_000_000))
      .accounts({
        pool: pool.publicKey,
        poolAuthority: poolAuthority,
        poolVault: poolVault,
        lpMint: lpMint1.publicKey,
        userLpToken: user1LpTokenAccount,
        userMainToken: user1MainTokenAccount,
        user: user1.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user1])
      .rpc();

    const user1MainAfter = await getAccount(provider.connection, user1MainTokenAccount);
    const user1LpAfter = await getAccount(provider.connection, user1LpTokenAccount);
    const poolVaultAfter = await getAccount(provider.connection, poolVault);

    console.log("User1 main token after:", user1MainAfter.amount.toString());
    console.log("User1 LP after:", user1LpAfter.amount.toString());
    console.log("Pool vault after:", poolVaultAfter.amount.toString());

    // User1 should get more than 100 tokens back due to 2x weight
    assert.equal(user1LpAfter.amount.toString(), "0");
    console.log("✅ User1 unstaked with 2x weight bonus");
  });
});

