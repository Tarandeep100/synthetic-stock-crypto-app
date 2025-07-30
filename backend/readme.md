# Stock to Crypto Exchange API

A simple Rust REST API for stock and crypto price display and crypto-to-stock swapping using Alpaca and OKX DEX APIs.

## Setup

1. **Install dependencies:**
   ```bash
   cargo build
   ```

2. **Set environment variables:**
   ```bash
   cp .env.example .env
   # Edit .env with your API credentials
   ```

3. **Run the server:**
   ```bash
   cargo run
   ```

   Server starts at `http://127.0.0.1:8080`

## API Endpoints

### Stock Endpoints

#### Get Stock Price
```http
GET /api/stock/price/{symbol}
```
Example:
```bash
curl http://localhost:8080/api/stock/price/AAPL
```

#### List All Stocks
```http
GET /api/stock/list
```

#### Buy Stock with USDT
```http
POST /api/stock/buy
Content-Type: application/json

{
  "symbol": "AAPL",
  "notional": "100.00"  // USD amount
}
```

#### Sell Stock to USDT
```http
POST /api/stock/sell
Content-Type: application/json

{
  "symbol": "AAPL",
  "notional": "100.00"  // USD amount
}
```

#### Get Account Info
```http
GET /api/account
```

#### Get Positions
```http
GET /api/positions
```

### Crypto Endpoints

#### Get Crypto Price
```http
GET /api/crypto/price?chainId=1&tokenAddress=0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
```
Parameters:
- `chainId`: Blockchain ID (1 for Ethereum, 56 for BSC, etc.)
- `tokenAddress`: Token contract address

#### Get Swap Quote
```http
GET /api/crypto/quote?chainId=1&fromTokenAddress=0x...&toTokenAddress=0x...&amount=1000000&slippage=0.5
```
Parameters:
- `chainId`: Blockchain ID
- `fromTokenAddress`: Source token address
- `toTokenAddress`: Destination token address (use USDT address for crypto-to-stock flow)
- `amount`: Amount in token's smallest unit
- `slippage`: Slippage tolerance (0.5 = 0.5%)

#### Execute Crypto Swap
```http
POST /api/crypto/swap
Content-Type: application/json

{
  "chainId": "1",
  "fromTokenAddress": "0x...",
  "toTokenAddress": "0x...",
  "amount": "1000000",
  "slippage": "0.5",
  "userWalletAddress": "0x..."
}
```

#### Buy Crypto with USDT
```http
POST /api/crypto/buy
Content-Type: application/json

{
  "chainId": "1",
  "fromTokenAddress": "0xdAC17F958D2ee523a2206206994597C13D831ec7",  // USDT address
  "toTokenAddress": "0x...",  // Target crypto token
  "amount": "100000000",  // USDT amount (with decimals)
  "slippage": "0.5",
  "userWalletAddress": "0x..."
}
```

#### List Tokens
```http
GET /api/crypto/tokens?chainId=1
```

### Health Check
```http
GET /health
```

## Crypto to Stock Swap Flow

1. **Get crypto quote to USDT**
   ```bash
   GET /api/crypto/quote?chainId=1&fromTokenAddress={TOKEN}&toTokenAddress={USDT}&amount={AMOUNT}
   ```

2. **Execute crypto to USDT swap**
   ```bash
   POST /api/crypto/swap
   ```

3. **Buy stock with USDT amount**
   ```bash
   POST /api/stock/buy
   {
     "symbol": "AAPL",
     "notional": "100.00"
   }
   ```

4. **Your event handler triggers synthetic stock release**

## Stock to Crypto Swap Flow (Reverse)

1. **Sell synthetic stock to USDT**
   ```bash
   POST /api/stock/sell
   {
     "symbol": "AAPL",
     "notional": "100.00"
   }
   ```

2. **Get quote for USDT to target crypto**
   ```bash
   GET /api/crypto/quote?chainId=1&fromTokenAddress={USDT}&toTokenAddress={TARGET_CRYPTO}&amount={USDT_AMOUNT}
   ```

3. **Buy crypto with USDT**
   ```bash
   POST /api/crypto/buy
   {
     "chainId": "1",
     "fromTokenAddress": "0xdAC17F958D2ee523a2206206994597C13D831ec7",
     "toTokenAddress": "0x...",
     "amount": "100000000",
     "slippage": "0.5",
     "userWalletAddress": "0x..."
   }
   ```

4. **Your event handler manages the synthetic stock burn**

## Common Token Addresses

### Ethereum (chainId: 1)
- USDT: `0xdAC17F958D2ee523a2206206994597C13D831ec7`
- USDC: `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48`
- WETH: `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2`

### BSC (chainId: 56)
- USDT: `0x55d398326f99059fF775485246999027B3197955`
- BUSD: `0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56`

## Response Format

All endpoints return JSON responses:

### Success Response
```json
{
  "code": "0",
  "data": [...],
  "msg": ""
}
```

### Error Response
```json
{
  "error": "Error description"
}
```

## Notes

- CORS is enabled for all origins (adjust for production)
- All crypto amounts are in the token's smallest unit (wei for ETH)
- Stock prices are in USD
- The API uses paper trading by default (change ALPACA_API_BASE_URL for live trading)