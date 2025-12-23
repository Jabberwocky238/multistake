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
} from "@solana/spl-token";

export interface PoolSetup {
  program: Program<Multistake>;
  provider: anchor.AnchorProvider;
  admin: Keypair;
  payer: Keypair;
  pool: Keypair;
  mainTokenMint: PublicKey;
  poolAuthority: PublicKey;
  poolVault: PublicKey;
}

/**
 * 创建并初始化一个测试用的 pool
 */
export async function setupPool(): Promise<PoolSetup> {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Multistake as Program<Multistake>;

  // 生成账户
  const admin = Keypair.generate();
  const payer = Keypair.generate();
  const pool = Keypair.generate();

  // 空投 SOL
  const airdropAmount = 10 * anchor.web3.LAMPORTS_PER_SOL;
  await provider.connection.confirmTransaction(
    await provider.connection.requestAirdrop(admin.publicKey, airdropAmount)
  );
  await provider.connection.confirmTransaction(
    await provider.connection.requestAirdrop(payer.publicKey, airdropAmount)
  );

  // 创建主币 mint
  const mainTokenMint = await createMint(
    provider.connection,
    payer,
    admin.publicKey,
    null,
    9
  );

  // 派生 PDA
  const [poolAuthority] = PublicKey.findProgramAddressSync(
    [new TextEncoder().encode("anyswap_authority"), pool.publicKey.toBytes()],
    program.programId
  );

  const [poolVault] = PublicKey.findProgramAddressSync(
    [new TextEncoder().encode("pool_vault"), pool.publicKey.toBytes()],
    program.programId
  );

  // 创建 pool 账户
  const poolSize = 24704;
  const lamports = await provider.connection.getMinimumBalanceForRentExemption(poolSize);

  const createPoolAccountIx = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: pool.publicKey,
    lamports,
    space: poolSize,
    programId: program.programId,
  });

  // 初始化 pool
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

  return {
    program,
    provider,
    admin,
    payer,
    pool,
    mainTokenMint,
    poolAuthority,
    poolVault,
  };
}
