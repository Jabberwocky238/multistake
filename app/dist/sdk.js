import { Program, BN } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram, } from "@solana/web3.js";
import { getAssociatedTokenAddress, } from "@solana/spl-token";
import IDL from "./multistake.json";
/**
 * AnySwap SDK - 单币质押系统
 */
export class MultiStakeSDK {
    constructor(program, provider) {
        this.program = program;
        this.provider = provider || program.provider;
    }
    /**
     * 使用内置 IDL 创建 SDK 实例
     */
    static create(provider) {
        const program = new Program(IDL, provider);
        return new MultiStakeSDK(program, provider);
    }
    /**
     * 获取 Program 实例
     */
    getProgram() {
        return this.program;
    }
    /**
     * 获取 Provider 实例
     */
    getProvider() {
        return this.provider;
    }
    /**
     * 派生 Pool Authority PDA
     */
    derivePoolAuthority(pool) {
        return PublicKey.findProgramAddressSync([new TextEncoder().encode("anyswap_authority"), pool.toBytes()], this.program.programId);
    }
    /**
     * 派生 Pool Vault PDA
     */
    derivePoolVault(pool) {
        return PublicKey.findProgramAddressSync([new TextEncoder().encode("pool_vault"), pool.toBytes()], this.program.programId);
    }
    /**
     * 创建 Pool
     * @param mainTokenMint 主币 mint 地址
     * @param admin Pool 管理员
     * @param payer 支付账户
     * @param config Pool 配置
     * @returns Pool 公钥和交易签名
     */
    async createPool(mainTokenMint, config = { feeNumerator: 3, feeDenominator: 1000 }) {
        const pool = Keypair.generate();
        const wallet = this.provider.publicKey;
        const [poolAuthority] = this.derivePoolAuthority(pool.publicKey);
        const [poolVault] = this.derivePoolVault(pool.publicKey);
        const poolSize = 24704;
        const lamports = await this.provider.connection.getMinimumBalanceForRentExemption(poolSize);
        const createPoolAccountIx = SystemProgram.createAccount({
            fromPubkey: wallet,
            newAccountPubkey: pool.publicKey,
            lamports,
            space: poolSize,
            programId: this.program.programId,
        });
        const signature = await this.program.methods
            .createPool(new BN(config.feeNumerator), new BN(config.feeDenominator))
            .accountsPartial({
            pool: pool.publicKey,
            poolAuthority,
            mainTokenMint,
            poolVault,
            admin: wallet,
            payer: wallet,
        })
            .preInstructions([createPoolAccountIx])
            .signers([pool])
            .rpc();
        return { pool: pool.publicKey, signature };
    }
    /**
     * 添加质押类型到 Pool
     * @param pool Pool 公钥
     * @returns LP mint 公钥和交易签名
     */
    async addTokenToPool(pool) {
        const lpMint = Keypair.generate();
        const wallet = this.provider.publicKey;
        const [poolAuthority] = this.derivePoolAuthority(pool);
        const signature = await this.program.methods
            .addTokenToPool()
            .accountsPartial({
            pool,
            poolAuthority,
            lpMint: lpMint.publicKey,
            admin: wallet,
            payer: wallet,
        })
            .signers([lpMint])
            .rpc();
        return { lpMint: lpMint.publicKey, signature };
    }
    /**
     * 从 Pool 移除质押类型
     * @param pool Pool 公钥
     * @param lpMint LP mint 公钥
     * @param admin 管理员
     * @returns 交易签名
     */
    async removeTokenFromPool(pool, lpMint, admin) {
        const signature = await this.program.methods
            .removeTokenFromPool()
            .accounts({
            pool,
            lpMint,
            admin: admin.publicKey,
        })
            .signers([admin])
            .rpc();
        return signature;
    }
    /**
     * 修改 Token 权重
     * @param pool Pool 公钥
     * @param params 权重参数
     * @param admin 管理员
     * @returns 交易签名
     */
    async modifyTokenWeight(pool, params, admin) {
        const weights = params.weights.map((w) => typeof w === "number" ? new BN(w) : w);
        const signature = await this.program.methods
            .modifyTokenWeight(weights)
            .accounts({
            pool,
            admin: admin.publicKey,
        })
            .remainingAccounts(params.tokenMints.map((mint) => ({
            pubkey: mint,
            isSigner: false,
            isWritable: false,
        })))
            .signers([admin])
            .rpc();
        return signature;
    }
    /**
     * 质押主币，铸造 LP 凭证
     */
    async stake(pool, itemIndex, lpMint, amount) {
        const wallet = this.provider.publicKey;
        const [poolVault] = this.derivePoolVault(pool);
        // Convert amount to BN with proper decimals (9 decimals for SOL/WSOL)
        const amountBN = typeof amount === "number" ? new BN(amount * 1e9) : amount;
        // Get pool info to get main token mint
        const poolInfo = await this.getPoolInfo(pool);
        const mainTokenMint = poolInfo.poolMint;
        const userMainToken = await getAssociatedTokenAddress(mainTokenMint, wallet);
        // Get or create user's LP token account
        const userLpToken = await getAssociatedTokenAddress(lpMint, wallet);
        // Check if accounts exist, if not, create them
        const preInstructions = [];
        // Check user main token account
        const mainTokenAccountInfo = await this.provider.connection.getAccountInfo(userMainToken);
        if (!mainTokenAccountInfo) {
            const { createAssociatedTokenAccountInstruction } = await import("@solana/spl-token");
            preInstructions.push(createAssociatedTokenAccountInstruction(wallet, userMainToken, wallet, mainTokenMint));
        }
        // Check user LP token account
        const lpTokenAccountInfo = await this.provider.connection.getAccountInfo(userLpToken);
        if (!lpTokenAccountInfo) {
            const { createAssociatedTokenAccountInstruction } = await import("@solana/spl-token");
            preInstructions.push(createAssociatedTokenAccountInstruction(wallet, userLpToken, wallet, lpMint));
        }
        const signature = await this.program.methods
            .stake(itemIndex, amountBN)
            .accountsPartial({
            pool,
            poolVault,
            lpMint,
            userMainToken,
            userLpToken,
            user: wallet,
        })
            .preInstructions(preInstructions)
            .rpc();
        return signature;
    }
    /**
     * 销毁 LP 凭证，赎回主币
     */
    async unstake(pool, itemIndex, lpMint, lpAmount) {
        const wallet = this.provider.publicKey;
        const [poolVault] = this.derivePoolVault(pool);
        // Convert LP amount to BN with proper decimals (9 decimals for LP tokens)
        const lpAmountBN = typeof lpAmount === "number" ? new BN(lpAmount * 1e9) : lpAmount;
        // Get pool info to get main token mint
        const poolInfo = await this.getPoolInfo(pool);
        const mainTokenMint = poolInfo.poolMint;
        const userMainToken = await getAssociatedTokenAddress(mainTokenMint, wallet);
        const userLpToken = await getAssociatedTokenAddress(lpMint, wallet);
        const signature = await this.program.methods
            .unstake(itemIndex, lpAmountBN)
            .accountsPartial({
            pool,
            poolVault,
            lpMint,
            userLpToken,
            userMainToken,
            user: wallet,
        })
            .rpc();
        return signature;
    }
    /**
     * 获取 Pool 信息
     */
    async getPoolInfo(pool) {
        const poolAccount = await this.program.account.pool.fetch(pool);
        let poolItems = [];
        for (let i = 0; i < poolAccount.tokenCount; i++) {
            const token = poolAccount.tokens[i];
            if (token.mintAccount && token.mintAccount.toString() !== PublicKey.default.toString()) {
                poolItems.push({
                    mintAccount: token.mintAccount,
                    mintAmount: token.mintAmount,
                    weight: token.weight
                });
            }
        }
        return {
            admin: poolAccount.admin,
            poolVault: poolAccount.poolVault,
            poolMint: poolAccount.poolMint,
            tokenCount: poolAccount.tokenCount,
            feeNumerator: poolAccount.feeNumerator,
            feeDenominator: poolAccount.feeDenominator,
            items: poolItems,
        };
    }
    /**
     * 获取 Pool 中所有的 LP mint
     */
    async getPoolLpMints(pool) {
        const poolInfo = await this.getPoolInfo(pool);
        return poolInfo.items.map((item) => item.mintAccount);
    }
}
