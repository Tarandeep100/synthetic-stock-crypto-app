export interface Token {
    symbol: string;
    name: string;
    address?: string;
    decimals?: number;
    chainId?: number;
    logoURI?: string;
    balance?: string;
    balanceUSD?: string;
  }
  
  export type SwapType = 'crypto-to-stock' | 'stock-to-crypto';
  
  export interface SwapQuote {
    fromToken: Token;
    toToken: Token;
    fromAmount: string;
    outputAmount: string;
    priceImpact: string;
    route: string[];
    gas?: string;
  }
  
  export interface SwapParams {
    fromToken: Token;
    toToken: Token;
    amount: string;
    slippage: string;
    swapType: SwapType;
  }
  
  export interface StockData {
    symbol: string;
    name: string;
    price: number;
    change: number;
    changePercent: number;
    volume: number;
    marketCap: number;
  }
  
  export interface CryptoData {
    symbol: string;
    name: string;
    address: string;
    price: number;
    change24h: number;
    volume24h: number;
    marketCap: number;
  }

  export interface Token {
    symbol: string;
    name: string;
    type: 'crypto' | 'stock';
    address?: string;
    decimals?: number;
    price?: number;
  }
  
  export interface SwapQuote {
    fromAmount: string;
    toAmount: string;
    price: number;
    priceImpact: string;
    route: string[];
    gas?: string;
  }
  
  export interface PriceData {
    symbol: string;
    price: number;
    timestamp: number;
  }