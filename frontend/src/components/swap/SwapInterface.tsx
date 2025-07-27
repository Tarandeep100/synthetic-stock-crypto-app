'use client';

import { useState, useEffect } from 'react';
import TokenSelector from './TokenSelector';
import { usePrices } from '@/hooks/usePrices';
import { useSwapQuote } from '@/hooks/useSwapQuote';
import { Token } from '@/types';

// Token configurations with addresses
const CRYPTO_TOKENS: Token[] = [
  { symbol: 'ETH', name: 'Ethereum', type: 'crypto', address: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2', decimals: 18 },
  { symbol: 'USDC', name: 'USD Coin', type: 'crypto', address: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', decimals: 6 },
  { symbol: 'WBTC', name: 'Wrapped Bitcoin', type: 'crypto', address: '0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599', decimals: 8 },
  { symbol: 'DAI', name: 'Dai Stablecoin', type: 'crypto', address: '0x6B175474E89094C44Da98b954EedeAC495271d0F', decimals: 18 },
];

const STOCK_TOKENS: Token[] = [
  { symbol: 'AAPL', name: 'Apple Inc.', type: 'stock' },
  { symbol: 'GOOGL', name: 'Alphabet Inc.', type: 'stock' },
  { symbol: 'MSFT', name: 'Microsoft Corporation', type: 'stock' },
  { symbol: 'TSLA', name: 'Tesla Inc.', type: 'stock' },
];

export default function SwapInterface() {
  const [fromAmount, setFromAmount] = useState('');
  const [toAmount, setToAmount] = useState('');
  const [fromToken, setFromToken] = useState<Token | null>(null);
  const [toToken, setToToken] = useState<Token | null>(null);
  const [showFromSelector, setShowFromSelector] = useState(false);
  const [showToSelector, setShowToSelector] = useState(false);
  const [swapType, setSwapType] = useState<'crypto-to-stock' | 'stock-to-crypto'>('crypto-to-stock');

  const { prices } = usePrices([fromToken, toToken]);
  const { getQuote, quote, loading: quoteLoading } = useSwapQuote();

  // Update quote when input changes
  useEffect(() => {
    const updateQuote = async () => {
      if (fromToken && toToken && fromAmount && parseFloat(fromAmount) > 0) {
        const newQuote = await getQuote(fromToken, toToken, fromAmount, swapType);
        if (newQuote) {
          setToAmount(newQuote.toAmount);
        }
      } else {
        setToAmount('');
      }
    };

    const debounceTimer = setTimeout(updateQuote, 500);
    return () => clearTimeout(debounceTimer);
  }, [fromAmount, fromToken, toToken, swapType, getQuote]);

  const handleFlip = () => {
    setFromToken(toToken);
    setToToken(fromToken);
    setFromAmount(toAmount);
    setToAmount(fromAmount);
    setSwapType(swapType === 'crypto-to-stock' ? 'stock-to-crypto' : 'crypto-to-stock');
  };

  const handleAmountChange = (value: string) => {
    // Allow only numbers and decimal point
    if (value === '' || /^\d*\.?\d*$/.test(value)) {
      setFromAmount(value);
    }
  };

  const fromPrice = fromToken ? prices[fromToken.symbol] : 0;
  const toPrice = toToken ? prices[toToken.symbol] : 0;
  const fromValue = fromAmount && fromPrice ? (parseFloat(fromAmount) * fromPrice).toFixed(2) : null;
  const toValue = toAmount && toPrice ? (parseFloat(toAmount) * toPrice).toFixed(2) : null;

  return (
    <div className="bg-gray-800 rounded-3xl shadow-2xl p-6 border border-gray-700">
      <h2 className="text-xl font-semibold text-white mb-6">Swap</h2>
      
      {/* From Input */}
      <div className="bg-gray-700 rounded-2xl p-4 mb-2">
        <div className="flex justify-between mb-2">
          <span className="text-sm text-gray-400">From</span>
          {fromToken && <span className="text-sm text-gray-400">Balance: 0.00</span>}
        </div>
        <div className="flex justify-between items-center">
          <div className="flex-1">
            <input
              type="text"
              value={fromAmount}
              onChange={(e) => handleAmountChange(e.target.value)}
              placeholder="0.0"
              className="bg-transparent text-2xl font-medium text-white outline-none w-full"
            />
            {fromValue && (
              <div className="text-sm text-gray-400 mt-1">≈ ${fromValue}</div>
            )}
          </div>
          <button 
            onClick={() => setShowFromSelector(true)}
            className="flex items-center gap-2 bg-gray-600 hover:bg-gray-500 px-3 py-2 rounded-xl transition-colors ml-4"
          >
            {fromToken ? (
              <>
                <div className="w-6 h-6 rounded-full bg-gray-500 flex items-center justify-center text-xs font-bold">
                  {fromToken.symbol.charAt(0)}
                </div>
                <span className="font-medium text-white">{fromToken.symbol}</span>
              </>
            ) : (
              <span className="font-medium text-white">Select {swapType === 'crypto-to-stock' ? 'crypto' : 'stock'}</span>
            )}
            <svg className="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          </button>
        </div>
      </div>

      {/* Swap Button */}
      <div className="flex justify-center -my-2 relative z-10">
        <button 
          onClick={handleFlip}
          className="bg-gray-700 hover:bg-gray-600 border-4 border-gray-800 rounded-xl p-2 transition-all hover:rotate-180 duration-300"
        >
          <svg className="w-5 h-5 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
          </svg>
        </button>
      </div>

      {/* To Input */}
      <div className="bg-gray-700 rounded-2xl p-4 mb-2">
        <div className="flex justify-between mb-2">
          <span className="text-sm text-gray-400">To</span>
          {toToken && <span className="text-sm text-gray-400">Balance: 0.00</span>}
        </div>
        <div className="flex justify-between items-center">
          <div className="flex-1">
            <input
              type="text"
              value={quoteLoading ? 'Loading...' : toAmount}
              readOnly
              placeholder="0.0"
              className="bg-transparent text-2xl font-medium text-white outline-none w-full"
            />
            {toValue && (
              <div className="text-sm text-gray-400 mt-1">≈ ${toValue}</div>
            )}
          </div>
          <button 
            onClick={() => setShowToSelector(true)}
            className="flex items-center gap-2 bg-gray-600 hover:bg-gray-500 px-3 py-2 rounded-xl transition-colors ml-4"
          >
            {toToken ? (
              <>
                <div className="w-6 h-6 rounded-full bg-gray-500 flex items-center justify-center text-xs font-bold">
                  {toToken.symbol.charAt(0)}
                </div>
                <span className="font-medium text-white">{toToken.symbol}</span>
              </>
            ) : (
              <span className="font-medium text-white">Select {swapType === 'crypto-to-stock' ? 'stock' : 'crypto'}</span>
            )}
            <svg className="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          </button>
        </div>
      </div>

      {/* Swap Details */}
      {quote && (
        <div className="mt-4 p-3 bg-gray-700 rounded-xl text-sm space-y-2">
          <div className="flex justify-between text-gray-300">
            <span>Rate</span>
            <span>1 {fromToken?.symbol} = {(parseFloat(toAmount) / parseFloat(fromAmount)).toFixed(6)} {toToken?.symbol}</span>
          </div>
          <div className="flex justify-between text-gray-300">
            <span>Route</span>
            <span>{quote.route.join(' → ')}</span>
          </div>
          <div className="flex justify-between text-gray-300">
            <span>Price Impact</span>
            <span className="text-green-400">&lt;{quote.priceImpact}%</span>
          </div>
        </div>
      )}

      {/* Connect Wallet Button */}
      <button 
        disabled={!fromToken || !toToken || !fromAmount || quoteLoading}
        className="w-full py-4 mt-4 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed rounded-2xl font-semibold text-white transition-colors"
      >
        {fromToken && toToken && fromAmount ? `Swap ${fromToken.symbol} for ${toToken.symbol}` : 'Connect Wallet'}
      </button>

      {/* Token Selectors */}
      <TokenSelector
        isOpen={showFromSelector}
        onClose={() => setShowFromSelector(false)}
        onSelect={setFromToken}
        tokenType={swapType === 'crypto-to-stock' ? 'crypto' : 'stock'}
        tokens={swapType === 'crypto-to-stock' ? CRYPTO_TOKENS : STOCK_TOKENS}
      />
      <TokenSelector
        isOpen={showToSelector}
        onClose={() => setShowToSelector(false)}
        onSelect={setToToken}
        tokenType={swapType === 'crypto-to-stock' ? 'stock' : 'crypto'}
        tokens={swapType === 'crypto-to-stock' ? STOCK_TOKENS : CRYPTO_TOKENS}
      />
    </div>
  );
}