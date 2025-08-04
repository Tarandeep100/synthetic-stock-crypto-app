'use client';

import SwapInterface from '@/components/swap/SwapInterface';
import Header from '@/components/layout/Header';

export default function Home() {
  return (
    <main className="min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-blue-900">
      <Header />
      <div className="container mx-auto px-4 py-8">
        <div className="max-w-lg mx-auto">
          <div className="text-center mb-8">
            <h1 className="text-4xl font-bold text-white mb-4">
              Stock<span className="text-blue-400">Swap</span>
            </h1>
            <p className="text-gray-300">
              Trade crypto and tokenized stocks seamlessly on Solana
            </p>
          </div>
          
          <SwapInterface />
        </div>
      </div>
    </main>
  );
}
