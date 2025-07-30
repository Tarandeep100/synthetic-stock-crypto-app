import { useState, useEffect } from 'react';
import { Token } from '@/types';
import { apiClient } from '@/lib/api-client';

// Popular tokens and stocks for demo
const POPULAR_CRYPTO: Token[] = [
  {
    symbol: 'ETH',
    name: 'Ethereum',
    address: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2',
    decimals: 18,
    chainId: 1
  },
  {
    symbol: 'USDC',
    name: 'USD Coin',
    address: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
    decimals: 6,
    chainId: 1
  },
  {
    symbol: 'WBTC',
    name: 'Wrapped Bitcoin',
    address: '0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599',
    decimals: 8,
    chainId: 1
  },
  {
    symbol: 'DAI',
    name: 'Dai Stablecoin',
    address: '0x6B175474E89094C44Da98b954EedeAC495271d0F',
    decimals: 18,
    chainId: 1
  },
  {
    symbol: 'LINK',
    name: 'Chainlink',
    address: '0x514910771AF9Ca656af840dff83E8264EcF986CA',
    decimals: 18,
    chainId: 1
  }
];

const POPULAR_STOCKS: Token[] = [
  {
    symbol: 'AAPL',
    name: 'Apple Inc.',
    decimals: 2
  },
  {
    symbol: 'GOOGL',
    name: 'Alphabet Inc.',
    decimals: 2
  },
  {
    symbol: 'MSFT',
    name: 'Microsoft Corporation',
    decimals: 2
  },
  {
    symbol: 'AMZN',
    name: 'Amazon.com Inc.',
    decimals: 2
  },
  {
    symbol: 'TSLA',
    name: 'Tesla Inc.',
    decimals: 2
  },
  {
    symbol: 'META',
    name: 'Meta Platforms Inc.',
    decimals: 2
  },
  {
    symbol: 'NVDA',
    name: 'NVIDIA Corporation',
    decimals: 2
  },
  {
    symbol: 'SPY',
    name: 'SPDR S&P 500 ETF',
    decimals: 2
  }
];

export function useTokenList(tokenType: 'crypto' | 'stock') {
  const [tokens, setTokens] = useState<Token[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchTokens = async () => {
      try {
        if (tokenType === 'crypto') {
          // For demo, use popular tokens
          // In production, fetch from API
          setTokens(POPULAR_CRYPTO);
        } else {
          // For demo, use popular stocks
          // In production, fetch from API
          setTokens(POPULAR_STOCKS);
        }
      } catch (error) {
        console.error('Failed to fetch tokens:', error);
        // Fallback to hardcoded lists
        setTokens(tokenType === 'crypto' ? POPULAR_CRYPTO : POPULAR_STOCKS);
      } finally {
        setLoading(false);
      }
    };

    fetchTokens();
  }, [tokenType]);

  return { tokens, loading };
}