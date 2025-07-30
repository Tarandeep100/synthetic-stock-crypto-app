import axios from 'axios';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

export const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// API functions
export const stockApi = {
  getPrice: async (symbol: string) => {
    const response = await apiClient.get(`/api/stock/price/${symbol}`);
    return response.data;
  },
  
  getList: async () => {
    const response = await apiClient.get('/api/stock/list');
    return response.data;
  },
  
  buy: async (symbol: string, notional: string) => {
    const response = await apiClient.post('/api/stock/buy', { symbol, notional });
    return response.data;
  },
  
  sell: async (symbol: string, notional: string) => {
    const response = await apiClient.post('/api/stock/sell', { symbol, notional });
    return response.data;
  },
};

export const cryptoApi = {
  getPrice: async (chainId: string, tokenAddress: string) => {
    const response = await apiClient.get('/api/crypto/price', {
      params: { chainId, tokenAddress }
    });
    return response.data;
  },
  
  getQuote: async (params: {
    chainId: string;
    fromTokenAddress: string;
    toTokenAddress: string;
    amount: string;
    slippage: string;
  }) => {
    const response = await apiClient.get('/api/crypto/quote', { params });
    return response.data;
  },
  
  swap: async (data: any) => {
    const response = await apiClient.post('/api/crypto/swap', data);
    return response.data;
  },
  
  buy: async (data: any) => {
    const response = await apiClient.post('/api/crypto/buy', data);
    return response.data;
  },
  
  getTokens: async (chainId: string) => {
    const response = await apiClient.get('/api/crypto/tokens', {
      params: { chainId }
    });
    return response.data;
  },
};