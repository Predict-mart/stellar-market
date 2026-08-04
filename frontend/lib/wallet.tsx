'use client';

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from 'react';

interface WalletContextValue {
  address: string | null;
  isConnected: boolean;
  isFreighterInstalled: boolean;
  connect: () => Promise<void>;
  disconnect: () => void;
  signTransaction: (xdr: string) => Promise<string>;
}

const WalletContext = createContext<WalletContextValue | null>(null);

const STORAGE_KEY = 'stellarmarket:wallet:connected';

export function WalletProvider({ children }: { children: ReactNode }) {
  const [address, setAddress] = useState<string | null>(null);
  const [isFreighterInstalled, setIsFreighterInstalled] = useState(false);

  // Check if Freighter is installed and restore session on mount
  useEffect(() => {
    const init = async () => {
      try {
        const { isConnected } = await import('@stellar/freighter-api');
        const installed = await isConnected();
        setIsFreighterInstalled(installed);

        if (installed && localStorage.getItem(STORAGE_KEY) === 'true') {
          const { getPublicKey } = await import('@stellar/freighter-api');
          const key = await getPublicKey();
          if (key) setAddress(key);
        }
      } catch {
        setIsFreighterInstalled(false);
      }
    };
    init();
  }, []);

  const connect = useCallback(async () => {
    if (!isFreighterInstalled) {
      window.open('https://freighter.app/', '_blank');
      return;
    }
    try {
      const { getPublicKey } = await import('@stellar/freighter-api');
      const key = await getPublicKey();
      if (key) {
        setAddress(key);
        localStorage.setItem(STORAGE_KEY, 'true');
      }
    } catch (err) {
      console.error('Failed to connect Freighter:', err);
    }
  }, [isFreighterInstalled]);

  const disconnect = useCallback(() => {
    setAddress(null);
    localStorage.removeItem(STORAGE_KEY);
  }, []);

  const signTransaction = useCallback(async (xdr: string): Promise<string> => {
    if (!address) throw new Error('Wallet not connected');
    const { signTransaction: sign } = await import('@stellar/freighter-api');
    const network = process.env.NEXT_PUBLIC_STELLAR_NETWORK ?? 'testnet';
    const result = await sign(xdr, { network });
    return result;
  }, [address]);

  return (
    <WalletContext.Provider
      value={{
        address,
        isConnected: !!address,
        isFreighterInstalled,
        connect,
        disconnect,
        signTransaction,
      }}
    >
      {children}
    </WalletContext.Provider>
  );
}

export function useWallet(): WalletContextValue {
  const ctx = useContext(WalletContext);
  if (!ctx) throw new Error('useWallet must be used inside WalletProvider');
  return ctx;
}

export function truncateAddress(address: string): string {
  return `${address.slice(0, 4)}...${address.slice(-4)}`;
}