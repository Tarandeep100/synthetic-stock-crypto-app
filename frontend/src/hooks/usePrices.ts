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
              return { symbol: token.symbol, price: 0 };
            }
          } else {
            try {
              // For crypto tokens, fetch from crypto API
              const data = await cryptoApi.getPrice('1', token.address || '');
              
              const priceData = data.data?.[0];
              const price = priceData ? parseFloat(priceData.price) : 0;
              return { symbol: token.symbol, price };
            } catch (error) {
              return { symbol: token.symbol, price: 0 };
            }
          }
        });

        const results = await Promise.all(pricePromises);
        const newPrices = results.reduce((acc, { symbol, price }) => {
          acc[symbol] = price;
          return acc;
        }, {} as Record<string, number>);

        setPrices(newPrices);
      } catch (error) {
        // Silently handle errors, individual token errors are already handled above
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