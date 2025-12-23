import { Program, AnchorProvider, BN } from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import { Multistake } from "./types/multistake";
import { PoolConfig, ModifyWeightParams } from "./types";
/**
 * AnySwap SDK - 单币质押系统
 */
export declare class AnySwapSDK {
    private program;
    private provider;
    constructor(program: Program<Multistake>, provider?: AnchorProvider);
    /**
     * 获取 Program 实例
     */
    getProgram(): Program<Multistake>;
    /**
     * 获取 Provider 实例
     */
    getProvider(): AnchorProvider;
    /**
     * 派生 Pool Authority PDA
     */
    derivePoolAuthority(pool: PublicKey): [PublicKey, number];
    /**
     * 派生 Pool Vault PDA
     */
    derivePoolVault(pool: PublicKey): [PublicKey, number];
    /**
     * 创建 Pool
     * @param mainTokenMint 主币 mint 地址
     * @param admin Pool 管理员
     * @param payer 支付账户
     * @param config Pool 配置
     * @returns Pool 公钥和交易签名
     */
    createPool(mainTokenMint: PublicKey, config?: PoolConfig): Promise<{
        pool: PublicKey;
        signature: string;
    }>;
    /**
     * 添加质押类型到 Pool
     * @param pool Pool 公钥
     * @returns LP mint 公钥和交易签名
     */
    addTokenToPool(pool: PublicKey): Promise<{
        lpMint: PublicKey;
        signature: string;
    }>;
    /**
     * 从 Pool 移除质押类型
     * @param pool Pool 公钥
     * @param lpMint LP mint 公钥
     * @param admin 管理员
     * @returns 交易签名
     */
    removeTokenFromPool(pool: PublicKey, lpMint: PublicKey, admin: Keypair): Promise<string>;
    /**
     * 修改 Token 权重
     * @param pool Pool 公钥
     * @param params 权重参数
     * @param admin 管理员
     * @returns 交易签名
     */
    modifyTokenWeight(pool: PublicKey, params: ModifyWeightParams, admin: Keypair): Promise<string>;
    /**
     * 质押主币，铸造 LP 凭证
     */
    stake(pool: PublicKey, itemIndex: number, lpMint: PublicKey, amount: number | BN): Promise<string>;
    /**
     * 销毁 LP 凭证，赎回主币
     */
    unstake(pool: PublicKey, itemIndex: number, lpMint: PublicKey, lpAmount: number | BN): Promise<string>;
    /**
     * 获取 Pool 信息
     */
    getPoolInfo(pool: PublicKey): Promise<any>;
    /**
     * 获取 Pool 中所有的 LP mint
     */
    getPoolLpMints(pool: PublicKey): Promise<PublicKey[]>;
}
