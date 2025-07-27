'use client';

import { useAccount, useConnect, useDisconnect } from 'wagmi';
import { useState } from 'react';
import { WalletIcon, ChevronDownIcon } from '@heroicons/react/24/outline';

export default function Header() {
  const { address, isConnected } = useAccount();
  const { connect, connectors } = useConnect();
  const { disconnect } = useDisconnect();
  const [showWalletMenu, setShowWalletMenu] = useState(false);

  const formatAddress = (addr: string) => {
    return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
  };

  return (
    <header className="border-b border-gray-700">
      <div className="container mx-auto px-4 py-4">
        <div className="flex justify-between items-center">
          {/* Logo */}
          <div className="flex items-center gap-2">
            <div className="w-10 h-10 bg-blue-600 rounded-lg flex items-center justify-center">
              <span className="text-white font-bold text-xl">S</span>
            </div>
            <span className="text-xl font-bold text-white">StockSwap</span>
          </div>

          {/* Navigation */}
          <nav className="hidden md:flex items-center gap-8">
            <a href="#" className="text-white hover:text-blue-400 transition-colors">
              Swap
            </a>
            <a href="#" className="text-gray-400 hover:text-white transition-colors">
              Portfolio
            </a>
            <a href="#" className="text-gray-400 hover:text-white transition-colors">
              Markets
            </a>
          </nav>

          {/* Wallet Connection */}
          <div className="relative">
            {isConnected ? (
              <button
                onClick={() => setShowWalletMenu(!showWalletMenu)}
                className="flex items-center gap-2 bg-gray-700 hover:bg-gray-600 px-4 py-2 rounded-xl transition-colors"
              >
                <div className="w-2 h-2 bg-green-400 rounded-full"></div>
                <span className="text-white font-medium">
                  {formatAddress(address!)}
                </span>
                <ChevronDownIcon className="w-4 h-4 text-gray-400" />
              </button>
            ) : (
              <button
                onClick={() => connect({ connector: connectors[0] })}
                className="flex items-center gap-2 bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-xl transition-colors text-white font-medium"
              >
                <WalletIcon className="w-5 h-5" />
                Connect Wallet
              </button>
            )}

            {/* Wallet Dropdown */}
            {showWalletMenu && isConnected && (
              <div className="absolute right-0 mt-2 w-48 bg-gray-800 rounded-xl shadow-lg border border-gray-700 overflow-hidden">
                <button
                  onClick={() => {
                    disconnect();
                    setShowWalletMenu(false);
                  }}
                  className="w-full text-left px-4 py-3 text-white hover:bg-gray-700 transition-colors"
                >
                  Disconnect
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </header>
  );
}