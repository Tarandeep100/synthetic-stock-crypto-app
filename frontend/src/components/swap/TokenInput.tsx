'use client';

import { useState } from 'react';
import { ChevronDownIcon } from '@heroicons/react/24/outline';
import TokenSelector from './TokenSelector';
import { Token } from '@/types';
import { useTokenList } from '@/hooks/useTokenList';

interface TokenInputProps {
  label: string;
  token: Token | null;
  amount: string;
  onTokenSelect: (token: Token) => void;
  onAmountChange: (amount: string) => void;
  tokenType: 'crypto' | 'stock';
  price?: number;
  readOnly?: boolean;
}

export default function TokenInput({
  label,
  token,
  amount,
  onTokenSelect,
  onAmountChange,
  tokenType,
  price,
  readOnly = false
}: TokenInputProps) {
  const [showTokenSelector, setShowTokenSelector] = useState(false);
  const { tokens } = useTokenList(tokenType);

  const handleAmountChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    // Allow only numbers and decimal point
    if (value === '' || /^\d*\.?\d*$/.test(value)) {
      onAmountChange(value);
    }
  };

  const usdValue = amount && price ? (parseFloat(amount) * price).toFixed(2) : null;

  return (
    <div className="bg-gray-700 rounded-2xl p-4 mb-2">
      <div className="flex justify-between mb-2">
        <span className="text-sm text-gray-400">{label}</span>
        {token && (
          <span className="text-sm text-gray-400">
            Balance: 0.00
          </span>
        )}
      </div>

      <div className="flex justify-between items-center">
        <input
          type="text"
          value={amount}
          onChange={handleAmountChange}
          placeholder="0.0"
          readOnly={readOnly}
          className={`bg-transparent text-2xl font-medium text-white outline-none w-full ${
            readOnly ? 'cursor-not-allowed' : ''
          }`}
        />

        <button
          onClick={() => setShowTokenSelector(true)}
          className="flex items-center gap-2 bg-gray-600 hover:bg-gray-500 px-3 py-2 rounded-xl transition-colors"
        >
          {token ? (
            <>
              <div className="w-6 h-6 rounded-full bg-gray-500 flex items-center justify-center text-xs font-bold">
                {token.symbol.charAt(0)}
              </div>
              <span className="font-medium text-white">{token.symbol}</span>
            </>
          ) : (
            <span className="font-medium text-white">Select {tokenType}</span>
          )}
          <ChevronDownIcon className="w-4 h-4 text-gray-400" />
        </button>
      </div>

      {usdValue && (
        <div className="mt-2 text-sm text-gray-400">
          ≈ ${usdValue}
        </div>
      )}

      {showTokenSelector && (
        <TokenSelector
          isOpen={showTokenSelector}
          onClose={() => setShowTokenSelector(false)}
          onSelect={(token) => {
            onTokenSelect(token);
            setShowTokenSelector(false);
          }}
          tokenType={tokenType}
          tokens={tokens}
        />
      )}
    </div>
  );
}