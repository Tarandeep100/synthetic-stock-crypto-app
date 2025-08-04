# StockSwap Frontend - Solana RWA Trading

A Next.js frontend for trading tokenized stocks on the Solana blockchain. Built with Anchor, Solana Wallet Adapter, and modern React patterns.

## 🌟 Features

- **Solana Wallet Integration**: Support for Phantom, Solflare, and other Solana wallets
- **Real World Assets (RWA)**: Trade tokenized stocks on Solana
- **Modern UI**: Clean, responsive interface built with Tailwind CSS  
- **Real-time Quotes**: Live price feeds for crypto and stock tokens
- **Multi-token Support**: Swap between SOL, USDC, USDT and tokenized stocks
- **Transaction Management**: Toast notifications and error handling

## 🚀 Quick Start

### Prerequisites

- Node.js 18+
- A Solana wallet (Phantom, Solflare, etc.)
- Backend API running on port 8080

### Installation

```bash
# Install dependencies
npm install

# Start development server
npm run dev
```

Visit `http://localhost:3000` to see the app.

## 🔗 Wallet Connection

The app supports multiple Solana wallets:

- **Phantom** - Most popular Solana wallet
- **Solflare** - Feature-rich web wallet  
- **Math Wallet** - Multi-chain support
- **Ledger** - Hardware wallet integration
- **Trust Wallet** - Mobile-first wallet

## 🏗 Architecture

### Core Components

- `SwapInterface.tsx` - Main trading interface
- `WalletButton.tsx` - Solana wallet connection
- `SolanaProvider.tsx` - Wallet adapter configuration

### Hooks

- `useProgram.ts` - Anchor program interaction
- `useTransactionToast.ts` - Transaction notifications
- `usePrices.ts` - Real-time price feeds
- `useSwapQuote.ts` - Swap quote calculations

### Configuration

- **Network**: Solana Devnet (configurable)
- **RPC**: Uses `clusterApiUrl` by default
- **Program ID**: `9MWyubXRFZawmGVE9WqQXCvQnS1YiRx3u35vkeKaNbrL`

## 📝 Environment Variables

Create a `.env.local` file:

```env
# Backend API
NEXT_PUBLIC_API_URL=http://localhost:8080

# Solana Configuration
NEXT_PUBLIC_STOCK_CONTRACTS_PROGRAM_ID=9MWyubXRFZawmGVE9WqQXCvQnS1YiRx3u35vkeKaNbrL
NEXT_PUBLIC_SOLANA_RPC_URL=https://api.devnet.solana.com

# Development
NEXT_PUBLIC_ENVIRONMENT=development
```

## 🔧 Development

### Supported Tokens

**Crypto Tokens:**
- SOL (native Solana)
- USDC (`EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`)
- USDT (`Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB`) 
- RAY (`4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R`)

**Stock Tokens:**
- AAPL (Apple Inc.)
- GOOGL (Alphabet Inc.)
- MSFT (Microsoft Corporation)
- TSLA (Tesla Inc.)
- NVDA (NVIDIA Corporation)
- AMZN (Amazon.com Inc.)

### Project Structure

```
src/
├── app/                    # Next.js app router
├── components/
│   ├── providers/          # Solana wallet providers
│   ├── solana/            # Solana-specific components  
│   └── swap/              # Trading interface
├── hooks/                 # Custom React hooks
├── lib/                   # Utilities and configuration
├── types/                 # TypeScript type definitions
└── anchor-idl/            # Solana program IDL files
```

## 🔐 Security

- All transactions require wallet approval
- No private keys stored in the application
- Environment variables for sensitive configuration
- Input validation and error handling

## 🌐 Integration

### Backend API

The frontend expects these API endpoints:

- `GET /api/stock/:symbol` - Stock price data
- `GET /api/crypto/:symbol` - Crypto price data  
- `POST /api/swap/quote` - Swap quote calculation
- `POST /api/swap/execute` - Execute swap transaction

### Solana Program

Uses Anchor framework with these instructions:

- `initializeTradingPool` - Setup trading pool
- `createStockMint` - Create stock token
- `placeBuyOrder` - Place stock buy order
- `fulfillBuyOrder` - Fulfill buy order

## 📚 Next Steps

1. **Deploy Smart Contracts**: Deploy your Anchor program to devnet
2. **Update Program ID**: Replace placeholder with actual program ID
3. **Add Real IDL**: Replace mock IDL with generated one
4. **Implement Transactions**: Connect swap logic to smart contracts
5. **Add Token Accounts**: Implement SPL token account management

## 🔗 Links

- [Solana Developers](https://github.com/solana-developers/anchor-web3js-nextjs)
- [Anchor Framework](https://anchor-lang.com/)
- [Solana Wallet Adapter](https://github.com/solana-labs/wallet-adapter)

Built for the RWA (Real World Assets) track - tokenizing traditional stocks on Solana! 🚀
