'use client';

import { useState, useEffect } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import TokenSelector from './TokenSelector';
import WalletButton from '@/components/solana/WalletButton';
import { usePrices } from '@/hooks/usePrices';
import { useSwapQuote } from '@/hooks/useSwapQuote';
import { useProgram } from '@/hooks/useProgram';
import { useTransactionToast } from '@/hooks/useTransactionToast';
import { useStockList } from '@/hooks/useStockList';
import { useTokenList } from '@/hooks/useTokenList';
import { Token } from '@/types';

export default function SwapInterface() {
  const { connected, publicKey, sendTransaction } = useWallet();
  const { program, provider } = useProgram();
  const { notifySuccess, notifyError, notifyLoading } = useTransactionToast();
  const { stocks: dynamicStocks, loading: stocksLoading, error: stocksError } = useStockList();
  const { tokens: cryptoTokens, loading: cryptoLoading, error: cryptoError } = useTokenList('crypto');
  
  const [fromAmount, setFromAmount] = useState('');
  const [toAmount, setToAmount] = useState('');
  const [fromToken, setFromToken] = useState<Token | null>(null);
  const [toToken, setToToken] = useState<Token | null>(null);
  const [showFromSelector, setShowFromSelector] = useState(false);
  const [showToSelector, setShowToSelector] = useState(false);
  const [swapType, setSwapType] = useState<'crypto-to-stock' | 'stock-to-crypto'>('crypto-to-stock');
  const [isSwapping, setIsSwapping] = useState(false);

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

  const handleSwap = async () => {
    if (!connected || !publicKey || !fromToken || !toToken || !fromAmount) {
      notifyError(new Error('Please connect wallet and select tokens'));
      return;
    }

    setIsSwapping(true);
    notifyLoading('Preparing swap transaction...');

    try {
      // TODO: Implement actual swap logic using your Solana program
      // This will be replaced with real smart contract calls
      
      throw new Error('Swap functionality not yet implemented. Please deploy smart contracts first.');
      
    } catch (error) {
      notifyError(error as Error, 'Swap failed');
    } finally {
      setIsSwapping(false);
    }
  };

  const fromPrice = fromToken ? prices[fromToken.symbol] : 0;
  const toPrice = toToken ? prices[toToken.symbol] : 0;
  const fromValue = fromAmount && fromPrice ? (parseFloat(fromAmount) * fromPrice).toFixed(2) : null;
  const toValue = toAmount && toPrice ? (parseFloat(toAmount) * toPrice).toFixed(2) : null;

  const canSwap = connected && fromToken && toToken && fromAmount && !quoteLoading && !isSwapping;

  return (
    <div className="bg-gray-800 rounded-3xl shadow-2xl p-6 border border-gray-700">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold text-white">Swap Crypto ↔ Stocks</h2>
        
        {/* Connection Status Indicator */}
        <div className="flex items-center gap-2">
          {stocksLoading || cryptoLoading ? (
            <div className="flex items-center gap-2 text-yellow-400 text-sm">
              <div className="w-3 h-3 border border-yellow-400 border-t-transparent rounded-full animate-spin"></div>
              <span>Loading tokens...</span>
            </div>
          ) : stocksError || cryptoError ? (
            <div className="flex items-center gap-2 text-red-400 text-sm">
              <div className="w-3 h-3 bg-red-400 rounded-full"></div>
              <span>API Error</span>
            </div>
          ) : (
            <div className="flex items-center gap-2 text-green-400 text-sm">
              <div className="w-3 h-3 bg-green-400 rounded-full"></div>
              <span>{dynamicStocks.length} stocks, {cryptoTokens.length} crypto</span>
            </div>
          )}
        </div>
      </div>
      
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
                <div className="w-6 h-6 rounded-full bg-gradient-to-r from-purple-500 to-blue-500 flex items-center justify-center text-xs font-bold">
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
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
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
                <div className="w-6 h-6 rounded-full bg-gradient-to-r from-green-500 to-blue-500 flex items-center justify-center text-xs font-bold">
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
            <span>Network</span>
            <span className="text-purple-400">Solana</span>
          </div>
          <div className="flex justify-between text-gray-300">
            <span>Price Impact</span>
            <span className="text-green-400">&lt;{quote.priceImpact}%</span>
          </div>
        </div>
      )}

      {/* Error Display */}
      {(stocksError || cryptoError) && (
        <div className="mb-4 p-3 bg-red-900/50 border border-red-500 rounded-lg">
          <p className="text-red-300 text-sm">
            {stocksError && `Stocks: ${stocksError}`}
            {stocksError && cryptoError && ' • '}
            {cryptoError && `Crypto: ${cryptoError}`}
          </p>
        </div>
      )}

      {/* Wallet Connection / Swap Section */}
      {!connected ? (
        <div className="mt-6">
          <div className="text-center mb-4">
            <h3 className="text-white font-medium mb-2">Connect your Solana wallet</h3>
            <p className="text-gray-400 text-sm">
              Connect with Phantom, Solflare, or any Solana wallet
            </p>
          </div>
          <WalletButton className="w-full" />
        </div>
      ) : (
        <div className="mt-6 space-y-3">
          {/* Connected wallet info */}
          <div className="flex items-center justify-between p-3 bg-gray-700 rounded-xl">
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 bg-gradient-to-r from-purple-500 to-blue-500 rounded-full flex items-center justify-center">
                <span className="text-white font-bold text-xs">S</span>
              </div>
              <span className="text-white font-medium">Solana Wallet Connected</span>
            </div>
            <WalletButton />
          </div>
          
          {/* Swap Button */}
          <button 
            onClick={handleSwap}
            disabled={!canSwap}
            className="w-full py-4 bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-700 hover:to-blue-700 disabled:from-gray-600 disabled:to-gray-600 disabled:cursor-not-allowed rounded-2xl font-semibold text-white transition-all flex items-center justify-center gap-2"
          >
            {isSwapping ? (
              <>
                <div className="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                <span>Swapping...</span>
              </>
            ) : canSwap ? (
              `Swap ${fromToken?.symbol} for ${toToken?.symbol}`
            ) : (
              'Select tokens and amount'
            )}
          </button>
          
          {/* Powered by Solana */}
          <div className="text-center text-xs text-gray-500">
            Powered by Solana • RWA Stock Trading
          </div>
        </div>
      )}

      {/* Token Selectors */}
      {showFromSelector && (
        <TokenSelector
          isOpen={showFromSelector}
          onClose={() => setShowFromSelector(false)}
          onSelect={setFromToken}
          tokenType={swapType === 'crypto-to-stock' ? 'crypto' : 'stock'}
          tokens={swapType === 'crypto-to-stock' ? cryptoTokens : dynamicStocks}
          loading={swapType === 'crypto-to-stock' ? cryptoLoading : stocksLoading}
        />
      )}

      {showToSelector && (
        <TokenSelector
          isOpen={showToSelector}
          onClose={() => setShowToSelector(false)}
          onSelect={setToToken}
          tokenType={swapType === 'crypto-to-stock' ? 'stock' : 'crypto'}
          tokens={swapType === 'crypto-to-stock' ? dynamicStocks : cryptoTokens}
          loading={swapType === 'crypto-to-stock' ? stocksLoading : cryptoLoading}
        />
      )}
    </div>
  );
}