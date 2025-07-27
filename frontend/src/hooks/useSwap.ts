import { useState, useCallback } from 'react';
import { useAccount } from 'wagmi';
import { toast } from 'react-hot-toast';
import { SwapParams, SwapQuote, Token } from '@/types';
import { apiClient } from '@/lib/api-client';

export function useSwap() {
  const { address } = useAccount();
  const [loading, setLoading] = useState(false);

  const getQuote = useCallback(async ({
    fromToken,
    toToken,
    amount,
    swapType
  }: Omit<SwapParams, 'slippage'>): Promise<SwapQuote> => {
    try {
      if (swapType === 'crypto-to-stock') {
        // Get crypto to USDT quote
        const response = await apiClient.get('/api/crypto/quote', {
          params: {
            chainId: '1',
            fromTokenAddress: fromToken.address,
            toTokenAddress: '0xdAC17F958D2ee523a2206206994597C13D831ec7', // USDT
            amount: amount,
            slippage: '0.5'
          }
        });
        
        const usdtAmount = response.data.data[0].toTokenAmount;
        // Convert USDT to stock price
        const stockPrice = await apiClient.get(`/api/stock/price/${toToken.symbol}`);
        const stockAmount = (parseFloat(usdtAmount) / 1e6 / stockPrice.data.price).toFixed(6);
        
        return {
          fromToken,
          toToken,
          fromAmount: amount,
          outputAmount: stockAmount,
          priceImpact: '0.01',
          route: [fromToken.symbol, 'USDT', toToken.symbol],
          gas: '0.005'
        };
      } else {
        // Stock to crypto flow
        const stockPrice = await apiClient.get(`/api/stock/price/${fromToken.symbol}`);
        const usdtAmount = (parseFloat(amount) * stockPrice.data.price * 1e6).toFixed(0);
        
        // Get USDT to crypto quote
        const response = await apiClient.get('/api/crypto/quote', {
          params: {
            chainId: '1',
            fromTokenAddress: '0xdAC17F958D2ee523a2206206994597C13D831ec7', // USDT
            toTokenAddress: toToken.address,
            amount: usdtAmount,
            slippage: '0.5'
          }
        });
        
        return {
          fromToken,
          toToken,
          fromAmount: amount,
          outputAmount: response.data.data[0].toTokenAmount,
          priceImpact: '0.01',
          route: [fromToken.symbol, 'USDT', toToken.symbol],
          gas: '0.005'
        };
      }
    } catch (error) {
      console.error('Failed to get quote:', error);
      throw error;
    }
  }, []);

  const executeSwap = useCallback(async ({
    fromToken,
    toToken,
    amount,
    slippage,
    swapType
  }: SwapParams) => {
    if (!address) {
      toast.error('Please connect your wallet');
      return;
    }

    setLoading(true);
    const toastId = toast.loading('Preparing swap...');

    try {
      if (swapType === 'crypto-to-stock') {
        // Step 1: Swap crypto to USDT
        toast.loading('Swapping to USDT...', { id: toastId });
        const swapResponse = await apiClient.post('/api/crypto/swap', {
          chainId: '1',
          fromTokenAddress: fromToken.address,
          toTokenAddress: '0xdAC17F958D2ee523a2206206994597C13D831ec7', // USDT
          amount: amount,
          slippage: slippage,
          userWalletAddress: address
        });

        // Step 2: Buy stock with USDT
        toast.loading('Buying stock...', { id: toastId });
        const usdtAmount = swapResponse.data.data[0].toTokenAmount;
        const notional = (parseFloat(usdtAmount) / 1e6).toFixed(2);
        
        await apiClient.post('/api/stock/buy', {
          symbol: toToken.symbol,
          notional: notional
        });

        toast.success('Swap completed successfully!', { id: toastId });
      } else {
        // Step 1: Sell stock for USDT
        toast.loading('Selling stock...', { id: toastId });
        const stockPrice = await apiClient.get(`/api/stock/price/${fromToken.symbol}`);
        const notional = (parseFloat(amount) * stockPrice.data.price).toFixed(2);
        
        await apiClient.post('/api/stock/sell', {
          symbol: fromToken.symbol,
          notional: notional
        });

        // Step 2: Buy crypto with USDT
        toast.loading('Buying crypto...', { id: toastId });
        const usdtAmount = (parseFloat(notional) * 1e6).toFixed(0);
        
        await apiClient.post('/api/crypto/buy', {
          chainId: '1',
          fromTokenAddress: '0xdAC17F958D2ee523a2206206994597C13D831ec7', // USDT
          toTokenAddress: toToken.address,
          amount: usdtAmount,
          slippage: slippage,
          userWalletAddress: address
        });

        toast.success('Swap completed successfully!', { id: toastId });
      }
    } catch (error) {
      console.error('Swap failed:', error);
      toast.error('Swap failed. Please try again.', { id: toastId });
    } finally {
      setLoading(false);
    }
  }, [address]);

  return {
    getQuote,
    executeSwap,
    loading
  };
}