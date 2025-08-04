import { useState, useCallback } from 'react';
import { stockApi, cryptoApi } from '@/lib/api-client';
import { Token, SwapQuote } from '@/types';

const USDT_ADDRESS = '0xdAC17F958D2ee523a2206206994597C13D831ec7';

export function useSwapQuote() {
  const [loading, setLoading] = useState(false);
  const [quote, setQuote] = useState<SwapQuote | null>(null);

  const getQuote = useCallback(async (
    fromToken: Token,
    toToken: Token,
    amount: string,
    swapType: 'crypto-to-stock' | 'stock-to-crypto'
  ) => {
    if (!amount || parseFloat(amount) === 0) {
      setQuote(null);
      return null;
    }

    setLoading(true);
    try {
      if (swapType === 'crypto-to-stock') {
        // Step 1: Get crypto to USDT quote
        const cryptoQuote = await cryptoApi.getQuote({
          chainId: '1',
          fromTokenAddress: fromToken.address || '',
          toTokenAddress: USDT_ADDRESS,
          amount: (parseFloat(amount) * 10 ** (fromToken.decimals || 18)).toString(),
          slippage: '0.5'
        });

        const usdtAmount = cryptoQuote.data?.[0]?.toTokenAmount || '0';
        const usdtValue = parseFloat(usdtAmount) / 1e6; // USDT has 6 decimals

        // Step 2: Calculate stock amount from USDT
        const stockPrice = await stockApi.getPrice(toToken.symbol);
        const stockAmount = usdtValue / parseFloat(stockPrice.price);

        const quote: SwapQuote = {
          fromToken,
          toToken,
          fromAmount: amount,
          toAmount: stockAmount.toFixed(6),
          priceImpact: '0.1',
          route: [fromToken.symbol, 'USDT', toToken.symbol]
        };

        setQuote(quote);
        return quote;
      } else {
        // Stock to crypto flow
        const stockPrice = await stockApi.getPrice(fromToken.symbol);
        const usdtValue = parseFloat(amount) * parseFloat(stockPrice.price);
        const usdtAmount = (usdtValue * 1e6).toFixed(0); // Convert to USDT decimals

        // Get USDT to crypto quote
        const cryptoQuote = await cryptoApi.getQuote({
          chainId: '1',
          fromTokenAddress: USDT_ADDRESS,
          toTokenAddress: toToken.address || '',
          amount: usdtAmount,
          slippage: '0.5'
        });

        const cryptoAmount = cryptoQuote.data?.[0]?.toTokenAmount || '0';
        const cryptoValue = parseFloat(cryptoAmount) / 10 ** (toToken.decimals || 18);

        const quote: SwapQuote = {
          fromToken,
          toToken,
          fromAmount: amount,
          toAmount: cryptoValue.toFixed(6),
          priceImpact: '0.1',
          route: [fromToken.symbol, 'USDT', toToken.symbol]
        };

        setQuote(quote);
        return quote;
      }
    } catch (error) {
      setQuote(null);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  return { getQuote, quote, loading };
}