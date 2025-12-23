import React, { StrictMode, FC, useState, useEffect } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey, Keypair } from '@solana/web3.js';
import { AnchorProvider, Program } from '@coral-xyz/anchor';
import { MultiStakeSDK } from '@KM-studio/multistake-sdk';

const PROGRAM_ID = new PublicKey('2mgSDKAjDo8fQN6oms6YzczHhyeYEJunTzxjQgegYADf');
// Native SOL wrapped token mint
const WSOL_MINT = new PublicKey('So11111111111111111111111111111111111111112');

const MultiStakeDemo: FC = () => {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [sdk, setSdk] = useState<MultiStakeSDK | null>(null);
  const [logs, setLogs] = useState<string[]>([]);
  const [poolAddress, setPoolAddress] = useState<string>('');
  const [stakeAmount, setStakeAmount] = useState<string>('100');
  const [lpMints, setLpMints] = useState<string[]>([]);
  const [selectedLpMint, setSelectedLpMint] = useState<string>('');
  const [selectedItemIndex, setSelectedItemIndex] = useState<number>(0);

  const log = (message: string) => {
    const time = new Date().toLocaleTimeString();
    setLogs(prev => [...prev, `[${time}] ${message}`]);
  };

  useEffect(() => {
    console.log('Wallet state changed:', {
      connected: wallet.connected,
      connecting: wallet.connecting,
      publicKey: wallet.publicKey?.toString(),
      wallet: wallet.wallet?.adapter.name
    });

    if (wallet.connected && wallet.publicKey) {
      initSDK();
    } else {
      setSdk(null);
    }
  }, [wallet.connected, wallet.publicKey]);

  useEffect(() => {
    if (wallet.error) {
      log(`⚠️ Wallet error: ${wallet.error.message}`);
      console.error('Wallet error details:', wallet.error);
    }
  }, [wallet.error]);

  const initSDK = async () => {
    try {
      if (!wallet.publicKey) return;

      log('Initializing SDK...');

      const provider = new AnchorProvider(
        connection,
        wallet as any,
        { commitment: 'confirmed' }
      );

      // 使用内置 IDL 创建 SDK
      const sdkInstance = MultiStakeSDK.create(provider);

      setSdk(sdkInstance);
      log('✅ SDK initialized');
    } catch (error: any) {
      log(`❌ Error initializing SDK: ${error.message}`);
      console.error('Full error:', error);
    }
  };

  const handleCreatePool = async () => {
    if (!sdk || !wallet.publicKey) return;

    try {
      log('Creating pool with WSOL as main token...');

      const { pool } = await sdk.createPool(
        WSOL_MINT,
        { feeNumerator: 3, feeDenominator: 1000 }
      );

      setPoolAddress(pool.toString());
      setLpMints([]); // Clear LP mints when creating new pool
      setSelectedLpMint('');
      log(`✅ Pool created: ${pool.toString()}`);
    } catch (error: any) {
      log(`❌ Error creating pool: ${error.message}`);
      console.error('Full error:', error);
    }
  };

  const loadLpMints = async () => {
    if (!sdk || !poolAddress) return;

    try {
      log('Loading LP mints from pool...');
      const mints = await sdk.getPoolLpMints(new PublicKey(poolAddress));
      const mintStrings = mints.map(m => m.toString());
      setLpMints(mintStrings);

      if (mintStrings.length > 0) {
        setSelectedLpMint(mintStrings[0]);
        setSelectedItemIndex(0);
      }

      log(`✅ Found ${mintStrings.length} LP mints`);
    } catch (error: any) {
      log(`❌ Error loading LP mints: ${error.message}`);
      console.error('Full error:', error);
    }
  };

  const handleAddToken = async () => {
    if (!sdk || !wallet.publicKey || !poolAddress) {
      log('❌ Please create a pool first');
      return;
    }

    try {
      log('Adding LP token to pool...');

      const { lpMint } = await sdk.addTokenToPool(
        new PublicKey(poolAddress)
      );

      log(`✅ LP token added: ${lpMint.toString()}`);

      // Reload LP mints after adding
      await loadLpMints();
    } catch (error: any) {
      log(`❌ Error adding token: ${error.message}`);
      console.error('Full error:', error);
    }
  };

  const handleWrapSOL = async () => {
    if (!wallet.publicKey) return;

    try {
      const amount = parseInt(stakeAmount);
      if (isNaN(amount) || amount <= 0) {
        log('❌ Invalid amount');
        return;
      }

      log(`Wrapping ${amount} SOL to WSOL...`);

      const { createSyncNativeInstruction, getAssociatedTokenAddress, createAssociatedTokenAccountInstruction, NATIVE_MINT } = await import('@solana/spl-token');
      const { Transaction, SystemProgram } = await import('@solana/web3.js');

      // Get WSOL associated token account
      const wsolAccount = await getAssociatedTokenAddress(
        NATIVE_MINT,
        wallet.publicKey
      );

      const transaction = new Transaction();

      // Check if WSOL account exists
      const accountInfo = await connection.getAccountInfo(wsolAccount);
      if (!accountInfo) {
        // Create WSOL account
        transaction.add(
          createAssociatedTokenAccountInstruction(
            wallet.publicKey,
            wsolAccount,
            wallet.publicKey,
            NATIVE_MINT
          )
        );
      }

      // Transfer SOL to WSOL account
      transaction.add(
        SystemProgram.transfer({
          fromPubkey: wallet.publicKey,
          toPubkey: wsolAccount,
          lamports: amount * 1e9, // Convert SOL to lamports
        })
      );

      // Sync native (wrap SOL)
      transaction.add(createSyncNativeInstruction(wsolAccount));

      // Send transaction
      const signature = await wallet.sendTransaction(transaction, connection);
      await connection.confirmTransaction(signature, 'confirmed');

      log(`✅ Wrapped ${amount} SOL to WSOL! Signature: ${signature}`);
    } catch (error: any) {
      log(`❌ Error wrapping SOL: ${error.message}`);
      console.error('Full error:', error);
    }
  };

  const handleStake = async () => {
    if (!sdk || !wallet.publicKey || !poolAddress || !selectedLpMint) {
      log('❌ Please create a pool, add token, and select LP mint first');
      return;
    }

    try {
      const amount = parseInt(stakeAmount);
      if (isNaN(amount) || amount <= 0) {
        log('❌ Invalid stake amount');
        return;
      }

      log(`Staking ${amount} tokens...`);

      const signature = await sdk.stake(
        new PublicKey(poolAddress),
        selectedItemIndex,
        new PublicKey(selectedLpMint),
        amount
      );

      log(`✅ Staked successfully! Signature: ${signature}`);

      // Query and display balances
      await displayBalances();
    } catch (error: any) {
      log(`❌ Error staking: ${error.message}`);
      console.error('Full error:', error);
    }
  };

  const displayBalances = async () => {
    if (!wallet.publicKey || !selectedLpMint) return;

    try {
      const { getAccount, getAssociatedTokenAddress, getMint } = await import('@solana/spl-token');

      // Get WSOL balance
      const wsolAccount = await getAssociatedTokenAddress(
        WSOL_MINT,
        wallet.publicKey
      );

      const wsolAccountInfo = await connection.getAccountInfo(wsolAccount);
      let wsolBalance = 0;
      if (wsolAccountInfo) {
        const wsolTokenAccount = await getAccount(connection, wsolAccount);
        wsolBalance = Number(wsolTokenAccount.amount) / 1e9;
      }

      // Get LP token balance
      const lpTokenAccount = await getAssociatedTokenAddress(
        new PublicKey(selectedLpMint),
        wallet.publicKey
      );

      log(`🔍 LP Token Account: ${lpTokenAccount.toString()}`);

      const lpAccountInfo = await connection.getAccountInfo(lpTokenAccount);
      let lpBalance = 0;
      let lpDecimals = 9;
      let rawAmount = '0';

      if (lpAccountInfo) {
        const lpMintInfo = await getMint(connection, new PublicKey(selectedLpMint));
        lpDecimals = lpMintInfo.decimals;

        const lpToken = await getAccount(connection, lpTokenAccount);
        rawAmount = lpToken.amount.toString();
        lpBalance = Number(lpToken.amount) / Math.pow(10, lpDecimals);

        log(`🔍 LP Raw Amount: ${rawAmount}`);
      } else {
        log(`⚠️ LP Token account not found`);
      }

      log(`💰 WSOL Balance: ${wsolBalance.toFixed(4)} WSOL`);
      log(`💰 LP Token Balance: ${lpBalance.toFixed(4)} LP (decimals: ${lpDecimals})`);

      // Get pool vault balance
      if (sdk && poolAddress) {
        const [poolVault] = sdk.derivePoolVault(new PublicKey(poolAddress));
        const poolVaultInfo = await connection.getAccountInfo(poolVault);

        if (poolVaultInfo) {
          const { getAccount } = await import('@solana/spl-token');
          const poolVaultAccount = await getAccount(connection, poolVault);
          const poolVaultBalance = Number(poolVaultAccount.amount) / 1e9;
          log(`🏦 Pool Vault Balance: ${poolVaultBalance.toFixed(4)} tokens`);
        }
      }
    } catch (error: any) {
      log(`⚠️ Error fetching balances: ${error.message}`);
      console.error('Balance error:', error);
    }
  };

  const handleUnstake = async () => {
    if (!sdk || !wallet.publicKey || !poolAddress || !selectedLpMint) {
      log('❌ Please create a pool, add token, and select LP mint first');
      return;
    }

    try {
      const amount = parseInt(stakeAmount);
      if (isNaN(amount) || amount <= 0) {
        log('❌ Invalid unstake amount');
        return;
      }

      log(`Unstaking ${amount} LP tokens...`);

      const signature = await sdk.unstake(
        new PublicKey(poolAddress),
        selectedItemIndex,
        new PublicKey(selectedLpMint),
        amount
      );

      log(`✅ Unstaked successfully! Signature: ${signature}`);

      // Query and display balances
      await displayBalances();
    } catch (error: any) {
      log(`❌ Error unstaking: ${error.message}`);
      console.error('Full error:', error);
    }
  };

  if (!wallet.connected) {
    return (
      <div className="demo-container">
        <p>Please connect your Phantom wallet to continue</p>
      </div>
    );
  }

  return (
    <div className="demo-container">
      <div className="section">
        <h2>Pool Operations</h2>
        <button onClick={handleCreatePool} disabled={!sdk}>
          Create Pool
        </button>
        <button onClick={handleAddToken} disabled={!sdk || !poolAddress}>
          Add LP Token
        </button>
        <button onClick={loadLpMints} disabled={!sdk || !poolAddress}>
          Load LP Mints
        </button>
        {poolAddress && (
          <div className="info">
            <strong>Pool Address:</strong> {poolAddress}
          </div>
        )}
      </div>

      <div className="section">
        <h2>Stake Operations</h2>
        <div style={{ marginBottom: '15px', padding: '10px', backgroundColor: '#fff3cd', borderRadius: '5px' }}>
          <strong>Note:</strong> You need WSOL to stake. Wrap your SOL first if needed.
        </div>
        {lpMints.length > 0 && (
          <div style={{ marginBottom: '10px' }}>
            <label>Select LP Mint: </label>
            <select
              value={selectedLpMint}
              onChange={(e) => {
                setSelectedLpMint(e.target.value);
                setSelectedItemIndex(lpMints.indexOf(e.target.value));
              }}
            >
              {lpMints.map((mint, index) => (
                <option key={mint} value={mint}>
                  {`Token ${index}: ${mint.slice(0, 8)}...${mint.slice(-8)}`}
                </option>
              ))}
            </select>
          </div>
        )}
        <input
          type="number"
          value={stakeAmount}
          onChange={(e) => setStakeAmount(e.target.value)}
          placeholder="Amount"
        />
        <button onClick={handleWrapSOL} disabled={!wallet.publicKey}>
          Wrap SOL to WSOL
        </button>
        <button onClick={handleStake} disabled={!sdk || !poolAddress || !selectedLpMint}>
          Stake
        </button>
        <button onClick={handleUnstake} disabled={!sdk || !poolAddress || !selectedLpMint}>
          Unstake
        </button>
      </div>

      <div className="section">
        <h2>Activity Log</h2>
        <div className="log">
          {logs.map((log, i) => (
            <div key={i}>{log}</div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default MultiStakeDemo;
