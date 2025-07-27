StockSwap Frontend
A Next.js-based frontend for the Stock-to-Crypto exchange application with a Uniswap-style interface.

Features
🔄 Swap between stocks and cryptocurrencies
💰 Real-time price updates
🦊 MetaMask and WalletConnect integration
📊 Price impact and slippage protection
🎨 Dark mode UI similar to Uniswap
📱 Fully responsive design
Setup Instructions
1. Install Dependencies
bash
cd frontend
npm install
2. Configure Environment Variables
bash
cp .env.local.example .env.local
# Edit .env.local with your configuration
3. Run Development Server
bash
npm run dev
Visit http://localhost:3000

Project Structure
frontend/
├── app/                    # Next.js 13+ app directory
│   ├── page.tsx           # Home page with swap interface
│   ├── layout.tsx         # Root layout
│   └── providers.tsx      # Web3 providers
├── components/            
│   ├── swap/              # Swap-related components
│   │   ├── SwapInterface.tsx
│   │   ├── TokenInput.tsx
│   │   ├── TokenSelector.tsx
│   │   ├── SwapButton.tsx
│   │   └── SwapDetails.tsx
│   └── layout/            # Layout components
│       └── Header.tsx
├── hooks/                 # Custom React hooks
│   ├── useSwap.ts
│   ├── useTokenList.ts
│   └── usePrices.ts
├── lib/                   # Utilities and configs
│   ├── api-client.ts
│   └── wagmi.ts
└── types/                 # TypeScript definitions
    └── index.ts
Key Components
SwapInterface
The main swap component that handles:

Token selection
Amount input
Swap execution
Price quotes
TokenSelector
Modal for selecting tokens with:

Search functionality
Popular tokens list
Balance display
SwapButton
Handles wallet connection and swap execution with proper states:

Connect wallet
Insufficient balance
Loading states
Swap confirmation
Customization
Adding New Tokens
Edit hooks/useTokenList.ts:

typescript
const POPULAR_CRYPTO: Token[] = [
  {
    symbol: 'NEW',
    name: 'New Token',
    address: '0x...',
    decimals: 18,
    chainId: 1
  },
  // ... more tokens
];
Styling
The app uses Tailwind CSS. Main color scheme:

Background: Gray 900-800
Cards: Gray 800-700
Primary: Blue 600
Text: White/Gray
API Integration
All API calls go through lib/api-client.ts. The flow:

Crypto → Stock
Get quote for crypto → USDT
Swap crypto to USDT
Buy stock with USDT
Stock → Crypto
Sell stock to USDT
Get quote for USDT → crypto
Buy crypto with USDT
Production Build
bash
npm run build
npm start
Environment Variables
NEXT_PUBLIC_API_URL: Backend API URL
NEXT_PUBLIC_WALLET_CONNECT_PROJECT_ID: WalletConnect project ID
NEXT_PUBLIC_ENABLE_TESTNETS: Enable test networks
Notes
The interface is optimized for desktop and mobile
All amounts are handled in their smallest units (wei for ETH, cents for stocks)
Slippage protection is built-in with configurable tolerance
The app uses RainbowKit for wallet connections
