import { Connection, PublicKey } from "@solana/web3.js";
import { AnchorProvider, Program, Wallet } from "@coral-xyz/anchor";
import { PhantomWalletAdapter } from "@solana/wallet-adapter-phantom";
import { AnySwapSDK } from "../../app/src/sdk";

// Program ID
const PROGRAM_ID = new PublicKey("2mgSDKAjDo8fQN6oms6YzczHhyeYEJunTzxjQgegYADf");
const RPC_URL = "http://127.0.0.1:8899";

let sdk: AnySwapSDK | null = null;
let phantomWallet: PhantomWalletAdapter | null = null;

// 日志函数
function log(message: string) {
  const logDiv = document.getElementById("log");
  if (logDiv) {
    const time = new Date().toLocaleTimeString();
    logDiv.innerHTML += `[${time}] ${message}<br>`;
    logDiv.scrollTop = logDiv.scrollHeight;
  }
  console.log(message);
}

// 连接 Phantom 钱包
async function connectWallet() {
  try {
    log("Connecting to Phantom wallet...");

    phantomWallet = new PhantomWalletAdapter();
    await phantomWallet.connect();

    if (!phantomWallet.publicKey) {
      throw new Error("Failed to get wallet public key");
    }

    log(`✅ Connected: ${phantomWallet.publicKey.toString()}`);

    // 创建 connection 和 provider
    const connection = new Connection(RPC_URL, "confirmed");
    const provider = new AnchorProvider(
      connection,
      phantomWallet as any,
      { commitment: "confirmed" }
    );

    // 加载 IDL 并创建 program
    const idl = await Program.fetchIdl(PROGRAM_ID, provider);
    if (!idl) {
      throw new Error("Failed to fetch IDL");
    }

    const program = new Program(idl, PROGRAM_ID, provider);
    sdk = new AnySwapSDK(program);

    log("✅ SDK initialized");

    // 更新 UI
    updateUI();
  } catch (error: any) {
    log(`❌ Error: ${error.message}`);
  }
}
