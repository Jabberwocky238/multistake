# MultiStake SDK

单币质押系统的 TypeScript SDK

## 特性

- ✅ 完整的类型支持
- ✅ 内置 IDL，无需外部依赖
- ✅ 简洁的 API 设计
- ✅ 支持所有质押操作

## 安装

```bash
npm install @KM-studio/multistake-sdk @coral-xyz/anchor @solana/web3.js @solana/spl-token
```

## 快速开始

```typescript
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { Wallet } from "@coral-xyz/anchor";
import { MultiStakeSDK } from "@KM-studio/multistake-sdk";

// 初始化
const connection = new Connection("http://127.0.0.1:8899");
const wallet = new Wallet(Keypair.generate());
const programId = new PublicKey("2mgSDKAjDo8fQN6oms6YzczHhyeYEJunTzxjQgegYADf");
const sdk = new MultiStakeSDK(connection, wallet, programId);
```

## 主要功能

### 1. Pool 管理

#### 创建 Pool
```typescript
const { pool, signature } = await sdk.createPool(
  mainTokenMint,    // 主币 mint 地址
  admin,            // 管理员 Keypair
  payer,            // 支付账户 Keypair
  config            // Pool 配置（可选）
);
```

#### 派生 PDA
```typescript
const [poolAuthority, bump1] = sdk.derivePoolAuthority(pool);
const [poolVault, bump2] = sdk.derivePoolVault(pool);
```
