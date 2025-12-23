import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { assert } from "chai";
import { MultiStakeSDK } from "../app/src/sdk";

describe("SDK Tests", () => {
  // SDK instance
  let sdk: MultiStakeSDK;
  const programId = new PublicKey("2mgSDKAjDo8fQN6oms6YzczHhyeYEJunTzxjQgegYADf");

  // Test accounts
  let payer: Keypair;
  let mainTokenMint: PublicKey;
  let pool: PublicKey;
  let lpMint1: PublicKey;
  let lpMint2: PublicKey;

  before(async () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    // Create a payer keypair for operations that need signing
    payer = Keypair.generate();

    // Airdrop SOL to payer
    const airdropAmount = 10 * anchor.web3.LAMPORTS_PER_SOL;
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, airdropAmount)
    );

    // Create main token mint
    mainTokenMint = await createMint(
      provider.connection,
      payer,
      payer.publicKey,
      null,
      9
    );

    // Initialize SDK with program from workspace
    const program = anchor.workspace.Multistake as anchor.Program<any>;
    sdk = new MultiStakeSDK(program, provider);

    console.log("✅ SDK initialized");
  });

  describe("Pool Operations", () => {
    it("Creates a pool using SDK", async () => {
      const result = await sdk.createPool(
        mainTokenMint,
        { feeNumerator: 3, feeDenominator: 1000 }
      );

      pool = result.pool;
      console.log("Pool created:", pool.toBase58());
      console.log("Transaction:", result.signature);

      assert.ok(pool);
      assert.ok(result.signature);
      console.log("✅ Pool created successfully using SDK");
    });

    it("Adds first LP token type using SDK", async () => {
      const result = await sdk.addTokenToPool(pool);
      lpMint1 = result.lpMint;

      console.log("LP Mint 1:", lpMint1.toBase58());
      assert.ok(lpMint1);
      console.log("✅ First LP token type added using SDK");
    });

    it("Adds second LP token type using SDK", async () => {
      const result = await sdk.addTokenToPool(pool);
      lpMint2 = result.lpMint;

      console.log("LP Mint 2:", lpMint2.toBase58());
      assert.ok(lpMint2);
      console.log("✅ Second LP token type added using SDK");
    });
  });

  describe("Stake and Unstake Operations", () => {
    it("Prepares token accounts and mints tokens", async () => {
      const provider = sdk.getProvider();

      // Create and mint main tokens to provider wallet
      const userMainTokenAccount = await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        mainTokenMint,
        provider.wallet.publicKey
      );

      await mintTo(
        provider.connection,
        payer,
        mainTokenMint,
        userMainTokenAccount.address,
        payer.publicKey,
        1000 * 1e9
      );

      const balance = await getAccount(provider.connection, userMainTokenAccount.address);
      assert.equal(balance.amount.toString(), (1000 * 1e9).toString());
      console.log("✅ Token accounts prepared");
    });

    it("Stakes tokens using SDK", async () => {
      const signature = await sdk.stake(
        pool,
        0, // itemIndex for lpMint1
        lpMint1,
        100 // amount in tokens (will be converted to lamports by SDK)
      );

      console.log("Stake transaction:", signature);
      assert.ok(signature);
      console.log("✅ Staked 100 tokens using SDK");
    });

    it("Unstakes tokens using SDK", async () => {
      const signature = await sdk.unstake(
        pool,
        0, // itemIndex for lpMint1
        lpMint1,
        50 // LP amount to unstake
      );

      console.log("Unstake transaction:", signature);
      assert.ok(signature);
      console.log("✅ Unstaked 50 LP tokens using SDK");
    });
  });
});
