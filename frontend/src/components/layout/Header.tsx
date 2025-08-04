'use client';

import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';

export default function Header() {
  const { connected, publicKey } = useWallet();

  const formatAddress = (addr: string) => {
    return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
  };

  return (
    <header className="border-b border-gray-700 bg-gray-900/50 backdrop-blur-sm">
      <div className="container mx-auto px-4 py-4">
        <div className="flex justify-between items-center">
          {/* Logo */}
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-gradient-to-r from-purple-600 to-blue-600 rounded-xl flex items-center justify-center">
              <span className="text-white font-bold text-xl">S</span>
            </div>
            <div className="flex flex-col">
              <span className="text-xl font-bold text-white">StockSwap</span>
              <span className="text-xs text-gray-400">Solana RWA Trading</span>
            </div>
          </div>

          {/* Navigation & Wallet */}
          <div className="flex items-center gap-4">
            {/* Navigation Links */}
            <nav className="hidden md:flex items-center gap-6">
              <a href="#" className="text-gray-300 hover:text-white transition-colors">
                Trade
              </a>
              <a href="#" className="text-gray-300 hover:text-white transition-colors">
                Portfolio
              </a>
              <a href="#" className="text-gray-300 hover:text-white transition-colors">
                Analytics
              </a>
            </nav>

            {/* Connection Status */}
            {connected && publicKey && (
              <div className="hidden sm:flex items-center gap-2 px-3 py-2 bg-gray-800 rounded-lg">
                <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                <span className="text-sm text-gray-300">
                  {formatAddress(publicKey.toString())}
                </span>
              </div>
            )}

            {/* Wallet Button */}
            <WalletMultiButton className="!bg-gradient-to-r !from-purple-600 !to-blue-600 hover:!from-purple-700 hover:!to-blue-700 !rounded-xl !font-semibold !transition-all !border-0" />
          </div>
        </div>
      </div>
    </header>
  );
}