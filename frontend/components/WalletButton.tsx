'use client';

import { useWallet, truncateAddress } from '@/lib/wallet';

export function WalletButton() {
  const { address, isConnected, isFreighterInstalled, connect, disconnect } = useWallet();

  if (isConnected && address) {
    return (
      <div className="flex items-center gap-2">
        <span className="hidden sm:block text-xs text-gray-400 font-mono bg-gray-800 px-3 py-1.5 rounded-lg border border-gray-700">
          {truncateAddress(address)}
        </span>
        <button
          onClick={disconnect}
          className="text-xs text-gray-400 hover:text-gray-200 px-3 py-1.5 rounded-lg border border-gray-700 hover:border-gray-500 transition-colors"
        >
          Disconnect
        </button>
      </div>
    );
  }

  if (!isFreighterInstalled) {
    return (
      <a
        href="https://freighter.app/"
        target="_blank"
        rel="noreferrer"
        className="btn-secondary text-sm"
      >
        Install Freighter
      </a>
    );
  }

  return (
    <button onClick={connect} className="btn-primary text-sm">
      Connect Wallet
    </button>
  );
}