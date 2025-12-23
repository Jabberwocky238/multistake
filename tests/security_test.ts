import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Multistake } from "../target/types/multistake";
import {
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";
import { setupPool } from "./pool_setup";

describe("Security Tests", () => {
  let setup: Awaited<ReturnType<typeof setupPool>>;
  let user1: Keypair;
  let lpMint1: Keypair;
  let lpMintForTest: Keypair;

  before(async () => {
    // 创建 pool
    setup = await setupPool();

    // 创建 user1（非 admin）
    user1 = Keypair.generate();
    lpMint1 = Keypair.generate();

    // 给 user1 空投 SOL
    const airdropAmount = 10 * anchor.web3.LAMPORTS_PER_SOL;
    await setup.provider.connection.confirmTransaction(
      await setup.provider.connection.requestAirdrop(user1.publicKey, airdropAmount)
    );

    console.log("✅ Pool created and user1 initialized");
  });

  it("Pool created successfully", async () => {
    const poolAccount = await setup.program.account.anySwapPool.fetch(setup.pool.publicKey);
    assert.equal(poolAccount.admin.toBase58(), setup.admin.publicKey.toBase58());
    console.log("✅ Pool admin verified");
  });

  it("User1 cannot add token (unauthorized)", async () => {
    try {
      await setup.program.methods
        .addTokenToPool()
        .accounts({
          pool: setup.pool.publicKey,
          poolAuthority: setup.poolAuthority,
          lpMint: lpMint1.publicKey,
          admin: user1.publicKey, // user1 尝试冒充 admin
          payer: user1.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([user1, lpMint1])
        .rpc();

      assert.fail("Should have failed with unauthorized error");
    } catch (error) {
      assert.include(error.toString(), "InvalidAdmin");
      console.log("✅ User1 correctly blocked from adding token");
    }
  });

  it("Admin adds a token for testing", async () => {
    lpMintForTest = Keypair.generate();
    await setup.program.methods
      .addTokenToPool()
      .accounts({
        pool: setup.pool.publicKey,
        poolAuthority: setup.poolAuthority,
        lpMint: lpMintForTest.publicKey,
        admin: setup.admin.publicKey,
        payer: setup.payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([setup.admin, setup.payer, lpMintForTest])
      .rpc();

    console.log("✅ Admin added token for testing");
  });

  it("User1 cannot modify weight (unauthorized)", async () => {
    try {
      await setup.program.methods
        .modifyTokenWeight([new anchor.BN(100_000_000)])
        .accounts({
          pool: setup.pool.publicKey,
          admin: user1.publicKey, // user1 尝试冒充 admin
        })
        .remainingAccounts([
          { pubkey: lpMintForTest.publicKey, isSigner: false, isWritable: false }
        ])
        .signers([user1])
        .rpc();

      assert.fail("Should have failed with unauthorized error");
    } catch (error) {
      assert.include(error.toString(), "InvalidAdmin");
      console.log("✅ User1 correctly blocked from modifying weight");
    }
  });
});
