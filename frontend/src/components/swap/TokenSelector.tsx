'use client';

import { useState } from 'react';
import { Token } from '@/types';

interface TokenSelectorProps {
  isOpen: boolean;
  onClose: () => void;
  onSelect: (token: Token) => void;
  tokenType: 'crypto' | 'stock';
  tokens: Token[];
  loading?: boolean;
}

export default function TokenSelector({ isOpen, onClose, onSelect, tokenType, tokens, loading = false }: TokenSelectorProps) {
  const [searchTerm, setSearchTerm] = useState('');
  
  if (!isOpen) return null;

  const filteredTokens = tokens.filter(token =>
    token.symbol.toLowerCase().includes(searchTerm.toLowerCase()) ||
    token.name.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const handleSelect = (token: Token) => {
    onSelect(token);
    onClose();
    setSearchTerm('');
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-2xl p-6 w-full max-w-md mx-4 max-h-[80vh] flex flex-col">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-semibold text-white">
            Select {tokenType === 'crypto' ? 'Crypto' : 'Stock'}
          </h3>
          <button 
            onClick={onClose}
            className="text-gray-400 hover:text-white"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="mb-4">
          <input
            type="text"
            placeholder={`Search ${tokenType}s...`}
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full bg-gray-700 border border-gray-600 rounded-xl px-4 py-3 text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
          />
        </div>

        <div className="flex-1 overflow-y-auto">
          {loading ? (
            <div className="flex flex-col items-center justify-center py-8 text-gray-400">
              <div className="w-8 h-8 border-2 border-gray-400 border-t-transparent rounded-full animate-spin mb-3"></div>
              <span>Loading {tokenType}s from backend...</span>
            </div>
          ) : filteredTokens.length === 0 ? (
            <div className="text-center py-8 text-gray-400">
              {tokens.length === 0 ? 
                `No ${tokenType}s available` : 
                `No ${tokenType}s found for "${searchTerm}"`
              }
            </div>
          ) : (
            <div className="space-y-2">
              {filteredTokens.map((token) => (
                <button
                  key={token.symbol}
                  onClick={() => handleSelect(token)}
                  className="w-full flex items-center gap-3 p-3 bg-gray-700 hover:bg-gray-600 rounded-xl transition-colors text-left"
                >
                  <div className="w-10 h-10 rounded-full bg-gradient-to-r from-blue-500 to-purple-500 flex items-center justify-center text-white font-bold">
                    {token.symbol.charAt(0)}
                  </div>
                  <div className="flex-1">
                    <div className="font-medium text-white">{token.symbol}</div>
                    <div className="text-sm text-gray-400 truncate">{token.name}</div>
                    {token.exchange && (
                      <div className="text-xs text-gray-500">{token.exchange}</div>
                    )}
                  </div>
                  {token.tradable && (
                    <div className="text-xs bg-green-600 text-white px-2 py-1 rounded">
                      Tradable
                    </div>
                  )}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
