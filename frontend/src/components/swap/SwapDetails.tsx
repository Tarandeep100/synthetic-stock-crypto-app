'use client';

import { useState } from 'react';
import { ChevronDownIcon } from '@heroicons/react/24/outline';
import { Token, SwapType } from '@/types';

interface SwapDetailsProps {
  fromToken: Token;
  toToken: Token;
  fromAmount: string;
  toAmount: string;
  slippage: string;
  swapType: SwapType;
}

export default function SwapDetails({
  fromToken,
  toToken,
  fromAmount,
  toAmount,
  slippage,
  swapType
}: SwapDetailsProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  const rate = parseFloat(toAmount) / parseFloat(fromAmount);
  const minimumReceived = (parseFloat(toAmount) * (1 - parseFloat(slippage) / 100)).toFixed(6);

  return (
    <div className="mt-4 bg-gray-700 rounded-xl p-4">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex justify-between items-center text-sm"
      >
        <span className="text-gray-300">
          1 {fromToken.symbol} = {rate.toFixed(6)} {toToken.symbol}
        </span>
        <ChevronDownIcon
          className={`w-4 h-4 text-gray-400 transition-transform ${
            isExpanded ? 'rotate-180' : ''
          }`}
        />
      </button>

      {isExpanded && (
        <div className="mt-4 space-y-3 text-sm">
          <div className="flex justify-between">
            <span className="text-gray-400">Price Impact</span>
            <span className="text-green-400">{'<0.01%'}</span>
          </div>
          
          <div className="flex justify-between">
            <span className="text-gray-400">Minimum received</span>
            <span className="text-white">
              {minimumReceived} {toToken.symbol}
            </span>
          </div>

          <div className="flex justify-between">
            <span className="text-gray-400">Slippage tolerance</span>
            <span className="text-white">{slippage}%</span>
          </div>

          <div className="flex justify-between">
            <span className="text-gray-400">Network fee</span>
            <span className="text-white">~$2.50</span>
          </div>

          <div className="flex justify-between">
            <span className="text-gray-400">Route</span>
            <span className="text-white">
              {swapType === 'crypto-to-stock' 
                ? `${fromToken.symbol} → USDT → ${toToken.symbol}`
                : `${fromToken.symbol} → USDT → ${toToken.symbol}`
              }
            </span>
          </div>
        </div>
      )}
    </div>
  );
}