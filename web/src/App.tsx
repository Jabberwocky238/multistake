import React, { FC, useMemo, useCallback } from 'react';
import { ConnectionProvider, WalletProvider } from '@solana/wallet-adapter-react';
import { PhantomWalletAdapter } from '@solana/wallet-adapter-phantom';
import {
  WalletModalProvider,
  WalletMultiButton
} from '@solana/wallet-adapter-react-ui';
import MultiStakeDemo from './components/MultiStakeDemo';

// Import wallet adapter CSS
import '@solana/wallet-adapter-react-ui/styles.css';
import './App.css';

const App: FC = () => {
  // Use local network for testing
  const endpoint = 'http://127.0.0.1:8899';

  const wallets = useMemo(
    () => [new PhantomWalletAdapter()],
    []
  );

  const onError = useCallback((error: any) => {
    console.error('Wallet error:', error);
    alert(`Wallet connection failed: ${error.message}`);
  }, []);

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} onError={onError}>
        <WalletModalProvider>
          <div className="App">
            <header>
              <h1>MultiStake SDK Demo</h1>
              <WalletMultiButton />
            </header>
            <main>
              <MultiStakeDemo />
            </main>
          </div>
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
};

export default App;
