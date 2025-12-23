import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { AnchorProvider, Wallet } from "@coral-xyz/anchor";
import { AnySwapSDK } from "./src";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";

/**
 * AnySwap SDK 使用示例
 */
async function example() {
  // 1. 初始化连接和钱包
  const connection = new Connection("http://127.0.0.1:8899", "confirmed");
  const payer = Keypair.generate();
  const admin = Keypair.generate();
  const user = Keypair.generate();

  // 空投 SOL
  await connection.confirmTransaction(
    await connection.requestAirdrop(payer.publicKey, 10e9)
  );
  await connection.confirmTransaction(
    await connection.requestAirdrop(admin.publicKey, 10e9)
  );
  await connection.confirmTransaction(
    await connection.requestAirdrop(user.publicKey, 10e9)
  );

  // 2. 创建 SDK 实例
  const programId = new PublicKey("A9CFGGXfQWrw7ZD5CinHqVvQyBKQswMiLnhfRVfbNMNe");
  const wallet = new Wallet(payer);
  const sdk = new AnySwapSDK(connection, wallet, programId);

  console.log("✅ SDK 初始化完成");

  // 3. 创建主币 mint
  const mainTokenMint = await createMint(
    connection,
    payer,
    admin.publicKey,
    null,
    9
  );
  console.log("✅ 主币 mint 创建:", mainTokenMint.toBase58());

  // 4. 创建 Pool
  const { pool, signature } = await sdk.createPool(
    mainTokenMint,
    admin,
    payer,
    { feeNumerator: 3, feeDenominator: 1000 }
  );
  console.log("✅ Pool 创建成功:", pool.toBase58());
  console.log("   交易签名:", signature);

  // 5. 添加质押类型
  const { lpMint: lpMint1, signature: sig1 } = await sdk.addTokenToPool(
    pool,
    admin,
    payer
  );
  console.log("✅ 质押类型 1 添加成功:", lpMint1.toBase58());

  const { lpMint: lpMint2, signature: sig2 } = await sdk.addTokenToPool(
    pool,
    admin,
    payer
  );
  console.log("✅ 质押类型 2 添加成功:", lpMint2.toBase58());

  // 6. 用户准备主币和 LP token 账户
  const userMainToken = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mainTokenMint,
    user.publicKey
  );

  // 给用户铸造一些主币
  await mintTo(
    connection,
    payer,
    mainTokenMint,
    userMainToken.address,
    admin,
    1000_000_000_000 // 1000 tokens
  );
  console.log("✅ 用户主币账户准备完成");

  // 7. 创建用户的 LP token 账户
  const userLpToken1 = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    lpMint1,
    user.publicKey
  );
  console.log("✅ 用户 LP token 账户创建完成");

  // 8. 用户质押 100 tokens
  const stakeSig = await sdk.stake(
    pool,
    lpMint1,
    userMainToken.address,
    userLpToken1.address,
    { itemIndex: 0, amount: 100_000_000_000 },
    user
  );
  console.log("✅ 用户质押成功，交易签名:", stakeSig);

  // 9. 修改权重
  const modifyWeightSig = await sdk.modifyTokenWeight(
    pool,
    {
      weights: [200_000_000, 50_000_000],
      tokenMints: [lpMint1, lpMint2],
    },
    admin
  );
  console.log("✅ 权重修改成功，交易签名:", modifyWeightSig);

  // 10. 用户取消质押
  const unstakeSig = await sdk.unstake(
    pool,
    lpMint1,
    userLpToken1.address,
    userMainToken.address,
    { itemIndex: 0, lpAmount: 100_000_000_000 },
    user
  );
  console.log("✅ 用户取消质押成功，交易签名:", unstakeSig);
}

// 运行示例
example().catch(console.error);
