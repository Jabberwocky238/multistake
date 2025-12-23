# AnySwap

AnySwap 是一个**超越并碾压 Balancer** 的多 token 交换系统。Balancer 受限于硬编码的 8 个 token，而 AnySwap 支持最多 **1024 个 token** 在同一个流动性池中自由交换。这使得 AnySwap 能够支持更复杂的 DeFi 场景，如大型指数基金、多资产组合池等，是 Balancer 无法企及的。

## 📍 程序部署地址

| Network | Program ID | IDL Account |
|---------|------------|-------------|
| Devnet  | `3GBxn5VSThpKNyUgaQ96xjSXD2zJ1164LzK28MXv4MDC` | `AHeBfQGsvCtWn2hFV3CrenfcqM38yk4ZAZMg2ZixQHPP` |
| Mainnet | 未部署 | - |

## 🔬 核心原理

### 恒定乘积和公式（Constant Sum of Products）

AnySwap 使用**权重恒定乘积公式**来维持池子的平衡：

```
Σ(weight_i × ln(vault_i)) = constant

aka

Π(vault_i ^ weight_i) = constant
```

## 🏗️ 技术架构

### 主要指令

- `create_pool`：创建新的流动性池
- `add_token`：添加 token 到池子
- `remove_token`：从池子移除 token
- `modify_weight`：修改 token 权重
- `modify_fee`：修改手续费率
- `add_liquidity`：添加流动性
- `remove_liquidity`：移除流动性
- `swap`：uniswap


**注意**：本项目仍在开发中，请勿在生产环境使用未经审计的版本。

