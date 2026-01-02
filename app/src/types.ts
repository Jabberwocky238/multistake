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




