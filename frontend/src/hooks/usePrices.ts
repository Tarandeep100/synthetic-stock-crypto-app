import { useState, useEffect } from 'react';
import { stockApi, cryptoApi } from '@/lib/api-client';
import { Token } from '@/types';

export function usePrices(tokens: (Token | null)[]) {
  const [prices, setPrices] = useState<Record<string, number>>({});
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const fetchPrices = async () => {
      const validTokens = tokens.filter(Boolean) as Token[];
      if (validTokens.length === 0) return;

      setLoading(true);
      try {
        const pricePromises = validTokens.map(async (token) => {
          if (token.type === 'stock') {
            try {
              const data = await stockApi.getPrice(token.symbol);
              return { symbol: token.symbol, price: parseFloat(data.price) };
            } catch (error) {
              console.error(`Failed to fetch price for ${token.symbol}:`, error);
              return { symbol: token.symbol, price: 0 };
            }
          } else {
            // For crypto, we would need the token address
            // For now, return mock price
            return { symbol: token.symbol, price: token.symbol === 'USDC' ? 1 : 2000 };
          }
        });

        const results = await Promise.all(pricePromises);
        const newPrices = results.reduce((acc, { symbol, price }) => {
          acc[symbol] = price;
          return acc;
        }, {} as Record<string, number>);

        setPrices(newPrices);
      } catch (error) {
        console.error('Failed to fetch prices:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchPrices();
    
    // Refresh prices every 30 seconds
    const interval = setInterval(fetchPrices, 30000);
    return () => clearInterval(interval);
  }, [tokens.map(t => t?.symbol).join(',')]);

  return { prices, loading };
}