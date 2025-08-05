export interface Token {
  symbol: string;
  name: string;
  type: 'crypto' | 'stock';
  address?: string; // For crypto tokens
  decimals?: number; // For crypto tokens
  tradable?: boolean; // For stock tokens
  fractionable?: boolean; // For stock tokens
  exchange?: string; // For stock tokens
  volume?: number; // Trading volume
  price?: number; // Current price
  change?: number; // Price change
  change_percent?: number; // Price change percentage
}

export type SwapType = 'crypto-to-stock' | 'stock-to-crypto';

export interface SwapQuote {
  fromToken: Token;
  toToken: Token;
  fromAmount: string;
  toAmount: string;
  route: string[];
  priceImpact: string;
  estimatedGas?: string;
  minimumReceived?: string;
}

export interface SwapParams {
  fromToken: Token;
  toToken: Token;
  amount: string;
  slippage?: number;
}

export interface StockData {
  symbol: string;
  name: string;
  price: number;
  change: number;
  changePercent: number;
  volume?: number;
  marketCap?: number;
}

export interface CryptoData {
  symbol: string;
  name: string;
  price: number;
  change24h: number;
  changePercent24h: number;
  volume24h?: number;
  marketCap?: number;
}

export interface PriceData {
  [symbol: string]: number;
}