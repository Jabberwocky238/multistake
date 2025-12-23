# MultiStake Web Demo

这是一个用于测试 MultiStake SDK 与 Phantom 钱包集成的 Web 演示页面。

## 功能特性

- ✅ Phantom 钱包连接
- ✅ 钱包地址显示
- ✅ Pool 操作界面
- ✅ Stake/Unstake 操作界面
- ✅ 实时活动日志

## 文件结构

```
web/
├── index.html              # 主测试页面
├── src/
│   └── phantom-adapter.ts  # Phantom 钱包适配器
└── README.md              # 本文件
```

## 使用方法

### 1. 启动本地服务器

```bash
# 在 web 目录下启动简单的 HTTP 服务器
cd web
python3 -m http.server 8080
```

或使用 Node.js:
```bash
npx http-server -p 8080
```

### 2. 访问页面

打开浏览器访问: `http://localhost:8080`

### 3. 连接 Phantom 钱包

1. 确保已安装 Phantom 钱包浏览器扩展
2. 点击 "Connect Phantom Wallet" 按钮
3. 在弹出的 Phantom 窗口中批准连接

### 4. 测试功能

- **Create Pool**: 创建新的质押池
- **Add LP Token**: 添加新的质押类型
- **Stake**: 质押代币
- **Unstake**: 取消质押

## Phantom 钱包适配器

`phantom-adapter.ts` 提供了将 Phantom 钱包接口适配为 Anchor Wallet 接口的功能。

### 核心功能

```typescript
// 创建适配 Phantom 的 SDK 实例
const result = await createSDKWithPhantom(connection, programId);
if (result) {
  const { sdk, wallet } = result;
  // 使用 sdk 进行操作
}
```

### PhantomWalletAdapter 类

实现了 Anchor Wallet 接口:
- `signTransaction()` - 签名单个交易
- `signAllTransactions()` - 批量签名交易
- `publicKey` - 钱包公钥

## 集成到实际项目

### 方法 1: 使用适配器

```typescript
import { createSDKWithPhantom } from './src/phantom-adapter';

const connection = new Connection("https://api.devnet.solana.com");
const programId = new PublicKey("YOUR_PROGRAM_ID");

const result = await createSDKWithPhantom(connection, programId);
if (result) {
  const { sdk } = result;

  // 使用 SDK
  const { pool } = await sdk.createPool(...);
}
```

### 方法 2: 直接使用 Phantom

```typescript
const phantom = window.solana;
const resp = await phantom.connect();

// 创建适配器
const wallet = new PhantomWalletAdapter(resp.publicKey, phantom);
const provider = new AnchorProvider(connection, wallet, opts);

// 创建 SDK
const program = new Program(idl, programId, provider);
const sdk = new AnySwapSDK(program);
```

## 注意事项

1. **本地测试**: 当前配置连接到本地 RPC (`http://127.0.0.1:8899`)
2. **Program ID**: 需要更新为实际部署的 program ID
3. **IDL 加载**: 生产环境需要正确配置 IDL 加载方式
4. **错误处理**: 实际项目中需要添加完善的错误处理

## 开发建议

### 构建工具

建议使用现代构建工具来处理 TypeScript 和模块打包:

- **Vite**: 快速的开发服务器和构建工具
- **Webpack**: 成熟的打包工具
- **Parcel**: 零配置打包工具

### 示例 (使用 Vite)

```bash
npm create vite@latest multistake-web -- --template vanilla-ts
cd multistake-web
npm install
npm install @solana/web3.js @coral-xyz/anchor @solana/spl-token
npm run dev
```

## 安全提示

⚠️ **重要**:
- 永远不要在前端代码中硬编码私钥
- 使用 Phantom 等钱包进行交易签名
- 验证所有用户输入
- 在生产环境使用 HTTPS

## 相关资源

- [Phantom Wallet Docs](https://docs.phantom.app/)
- [Solana Web3.js](https://solana-labs.github.io/solana-web3.js/)
- [Anchor Framework](https://www.anchor-lang.com/)
