import { useState, useEffect, useRef } from 'react';
import { Token } from '@/types';
import { apiClient } from '@/lib/api-client';

interface OKXToken {
  chainId: string;
  tokenContractAddress: string;
  tokenSymbol: string;
  tokenName: string;
  tokenLogoUrl?: string;
  decimals: string;
}

interface OKXTokenResponse {
  code: string;
  msg: string;
  data: OKXToken[];
}

// Global cache and request deduplication
const tokenCache = new Map<string, { tokens: Token[]; timestamp: number }>();
const activeRequests = new Map<string, Promise<Token[]>>();
const CACHE_DURATION = 5 * 60 * 1000; // 5 minutes

async function fetchCryptoTokens(): Promise<Token[]> {
  const cacheKey = 'crypto-tokens-1';
  const now = Date.now();
  
  // Check cache first
  const cached = tokenCache.get(cacheKey);
  if (cached && (now - cached.timestamp) < CACHE_DURATION) {
    return cached.tokens;
  }
  
  // Check if there's already an active request
  const activeRequest = activeRequests.get(cacheKey);
  if (activeRequest) {
    return activeRequest;
  }
  
  // Create new request with deduplication
  const requestPromise = (async () => {
    try {
      const response = await apiClient.get('/api/crypto/tokens', {
        params: { chainId: '1' }
      });
      
      const data: OKXTokenResponse = response.data;
      
      if (data.code === '0' && data.data) {
        const cryptoTokens: Token[] = data.data
          .slice(0, 20) // Limit to first 20 popular tokens
          .map((token: OKXToken) => ({
            symbol: token.tokenSymbol,
            name: token.tokenName,
            type: 'crypto' as const,
            address: token.tokenContractAddress,
            decimals: parseInt(token.decimals),
          }));
        
        // Cache the result
        tokenCache.set(cacheKey, { tokens: cryptoTokens, timestamp: now });
        
        return cryptoTokens;
      } else {
        throw new Error('Invalid response from crypto API');
      }
    } catch (error) {
      throw error;
    } finally {
      // Remove from active requests
      activeRequests.delete(cacheKey);
    }
  })();
  
  // Store the promise to prevent duplicate requests
  activeRequests.set(cacheKey, requestPromise);
  
  return requestPromise;
}

export function useTokenList(tokenType: 'crypto' | 'stock') {
  const [tokens, setTokens] = useState<Token[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const isMountedRef = useRef(true);

  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  useEffect(() => {
    const fetchTokens = async () => {
      if (!isMountedRef.current) return;
      
      try {
        setLoading(true);
        setError(null);

        if (tokenType === 'crypto') {
          const cryptoTokens = await fetchCryptoTokens();
          if (isMountedRef.current) {
            setTokens(cryptoTokens);
          }
        } else {
          // For stocks, return empty array since stocks are handled by useStockList
          if (isMountedRef.current) {
            setTokens([]);
          }
        }
      } catch (error) {
        if (isMountedRef.current) {
          const errorMessage = error instanceof Error ? error.message : 'Failed to fetch tokens';
          setError(errorMessage);
          setTokens([]);
        }
      } finally {
        if (isMountedRef.current) {
          setLoading(false);
        }
      }
    };

    fetchTokens();
  }, [tokenType]);

  return { tokens, loading, error };
}