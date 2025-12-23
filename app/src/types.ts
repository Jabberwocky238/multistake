import { PublicKey } from "@solana/web3.js";
import { BN } from "@coral-xyz/anchor";

/**
 * Pool 配置
 */
export interface PoolConfig {
  feeNumerator: number;
  feeDenominator: number;
}

/**
 * Pool 信息
 */
export interface PoolInfo {
  admin: PublicKey;
  poolVault: PublicKey;
  poolMint: PublicKey;
  tokenCount: number;
  feeNumerator: BN;
  feeDenominator: BN;
}

/**
 * Token 信息
 */
export interface TokenInfo {
  mintAccount: PublicKey;
  mintAmount: BN;
  weight: BN;
}

/**
 * 质押参数
 */
export interface StakeParams {
  itemIndex: number;
  amount: number | BN;
}

/**
 * 取消质押参数
 */
export interface UnstakeParams {
  itemIndex: number;
  lpAmount: number | BN;
}

/**
 * 修改权重参数
 */
export interface ModifyWeightParams {
  weights: (number | BN)[];
  tokenMints: PublicKey[];
}
