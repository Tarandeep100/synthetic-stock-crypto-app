'use client';

import { useState, useEffect } from 'react';
import { Token } from '@/types';

interface AlpacaAsset {
  id: string;
  class: string;
  exchange: string;
  symbol: string;
  name: string;
  status: string;
  tradable: boolean;
  marginable: boolean;
  shortable: boolean;
  easy_to_borrow: boolean;
  fractionable: boolean;
}

interface AlpacaResponse {
  data?: AlpacaAsset[];
  assets?: AlpacaAsset[];
}

export function useStockList() {
  const [stocks, setStocks] = useState<Token[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchStockList = async () => {
      try {
        setLoading(true);
        setError(null);

        const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
        
        const response = await fetch(`${API_URL}/api/stock/list`, {
          method: 'GET',
          headers: {
            'Content-Type': 'application/json',
          },
        });

        if (!response.ok) {
          throw new Error(`Failed to fetch stock list: ${response.status} ${response.statusText}`);
        }

        const data: AlpacaResponse = await response.json();

        // Handle different possible response formats from Alpaca
        const assets = data.data || data.assets || (Array.isArray(data) ? data : []);
        
        if (!Array.isArray(assets)) {
          throw new Error('Invalid response format from stock API');
        }

        // Convert Alpaca assets to our Token format
        const stockTokens: Token[] = assets
          .filter((asset: AlpacaAsset) => 
            asset.tradable && 
            asset.status === 'active' && 
            asset.class === 'us_equity' &&
            asset.symbol // Ensure symbol exists
          )
          .slice(0, 50) // Limit to first 50 for better UI performance
          .map((asset: AlpacaAsset) => ({
            symbol: asset.symbol,
            name: asset.name || asset.symbol,
            type: 'stock' as const,
            tradable: asset.tradable,
            fractionable: asset.fractionable,
            exchange: asset.exchange,
          }));

        setStocks(stockTokens);

      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to fetch stock list';
        setError(errorMessage);
        setStocks([]); // Clear stocks on error
      } finally {
        setLoading(false);
      }
    };

    fetchStockList();
  }, []);

  const refetch = async () => {
    setLoading(true);
    setError(null);
    // Re-run the fetch logic
    const fetchStockList = async () => {
      try {
        const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
        
        const response = await fetch(`${API_URL}/api/stock/list`, {
          method: 'GET',
          headers: {
            'Content-Type': 'application/json',
          },
        });

        if (!response.ok) {
          throw new Error(`Failed to fetch stock list: ${response.status} ${response.statusText}`);
        }

        const data: AlpacaResponse = await response.json();
        const assets = data.data || data.assets || (Array.isArray(data) ? data : []);
        
        if (!Array.isArray(assets)) {
          throw new Error('Invalid response format from stock API');
        }

        const stockTokens: Token[] = assets
          .filter((asset: AlpacaAsset) => 
            asset.tradable && 
            asset.status === 'active' && 
            asset.class === 'us_equity' &&
            asset.symbol
          )
          .slice(0, 50)
          .map((asset: AlpacaAsset) => ({
            symbol: asset.symbol,
            name: asset.name || asset.symbol,
            type: 'stock' as const,
            tradable: asset.tradable,
            fractionable: asset.fractionable,
            exchange: asset.exchange,
          }));

        setStocks(stockTokens);
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to fetch stock list';
        setError(errorMessage);
        setStocks([]);
      } finally {
        setLoading(false);
      }
    };
    
    await fetchStockList();
  };

  return {
    stocks,
    loading,
    error,
    refetch,
  };
} 