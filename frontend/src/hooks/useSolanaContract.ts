import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { Program, AnchorProvider, web3, BN } from '@project-serum/anchor';
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { useCallback, useMemo } from 'react';

// Your program ID (you'll get this after deployment)
const PROGRAM_ID = new PublicKey(process.env.NEXT_PUBLIC_STOCK_CONTRACTS_PROGRAM_ID || "9MWyubXRFZawmGVE9WqQXCvQnS1YiRx3u35vkeKaNbrL");

export interface PlaceBuyOrderParams {
  stockSymbol: string;
  solAmount: number; // SOL amount
  maxPricePerShare: number; // Max price willing to pay per share
}

export function useSolanaContract() {
  const { connection } = useConnection();
  const wallet = useWallet();

  const provider = useMemo(() => {
    if (!wallet.publicKey) return null;
    
    return new AnchorProvider(
      connection,
      wallet as any,
      { commitment: 'confirmed' }
    );
  }, [connection, wallet]);

  const program = useMemo(() => {
    if (!provider) return null;
    
    // You'll need to import your IDL here
    // const idl = require('../../../stock_contracts/target/idl/stock_contracts.json');
    // return new Program(idl, PROGRAM_ID, provider);
    
    return null; // Placeholder until IDL is available
  }, [provider]);

  // Derive PDAs (Program Derived Addresses)
  const getTradingPoolPDA = useCallback(() => {
    const [tradingPool] = PublicKey.findProgramAddressSync(
      [Buffer.from("trading_pool")],
      PROGRAM_ID
    );
    return tradingPool;
  }, []);

  const getTradingPoolVaultPDA = useCallback(() => {
    const [vault] = PublicKey.findProgramAddressSync(
      [Buffer.from("trading_pool_vault")],
      PROGRAM_ID
    );
    return vault;
  }, []);

  const getBuyOrderPDA = useCallback((user: PublicKey, orderId: number) => {
    const [buyOrder] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("buy_order"),
        user.toBuffer(),
        new BN(orderId).toArrayLike(Buffer, 'le', 8)
      ],
      PROGRAM_ID
    );
    return buyOrder;
  }, []);

  const getStockMintPDA = useCallback((stockSymbol: string) => {
    const [stockMint] = PublicKey.findProgramAddressSync(
      [Buffer.from("stock_mint"), Buffer.from(stockSymbol)],
      PROGRAM_ID
    );
    return stockMint;
  }, []);

  const getStockMintInfoPDA = useCallback((stockSymbol: string) => {
    const [stockMintInfo] = PublicKey.findProgramAddressSync(
      [Buffer.from("stock_mint_info"), Buffer.from(stockSymbol)],
      PROGRAM_ID
    );
    return stockMintInfo;
  }, []);

  const placeBuyOrder = useCallback(async (params: PlaceBuyOrderParams) => {
    if (!program || !wallet.publicKey) {
      throw new Error('Wallet not connected or program not loaded');
    }

    const { stockSymbol, solAmount, maxPricePerShare } = params;
    
    try {
      // Get current order count for PDA derivation
      const tradingPool = getTradingPoolPDA();
      const tradingPoolAccount = await program.account.tradingPool.fetch(tradingPool);
      const currentOrderId = tradingPoolAccount.totalOrders.toNumber();
      
      const buyOrderPDA = getBuyOrderPDA(wallet.publicKey, currentOrderId);
      const tradingPoolVault = getTradingPoolVaultPDA();
      
      const solAmountLamports = new BN(solAmount * LAMPORTS_PER_SOL);
      const maxPriceLamports = new BN(maxPricePerShare * LAMPORTS_PER_SOL);

      const tx = await program.methods
        .placeBuyOrder(
          stockSymbol,
          solAmountLamports,
          maxPriceLamports
        )
        .accounts({
          buyOrder: buyOrderPDA,
          tradingPool: tradingPool,
          tradingPoolVault: tradingPoolVault,
          user: wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      return {
        signature: tx,
        orderId: currentOrderId,
        buyOrderPDA: buyOrderPDA.toString(),
      };
    } catch (error) {
      throw error;
    }
  }, [program, wallet.publicKey, getTradingPoolPDA, getBuyOrderPDA, getTradingPoolVaultPDA]);

  const getUserStockBalance = useCallback(async (stockSymbol: string) => {
    if (!program || !wallet.publicKey) return 0;

    try {
      const stockMint = getStockMintPDA(stockSymbol);
      
      // Get associated token account
      const [associatedTokenAccount] = PublicKey.findProgramAddressSync(
        [
          wallet.publicKey.toBuffer(),
          TOKEN_PROGRAM_ID.toBuffer(),
          stockMint.toBuffer(),
        ],
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      const tokenAccount = await connection.getTokenAccountBalance(associatedTokenAccount);
      return tokenAccount.value.uiAmount || 0;
    } catch (error) {
      return 0;
    }
  }, [program, wallet.publicKey, connection, getStockMintPDA]);

  const getOrderStatus = useCallback(async (orderId: number) => {
    if (!program || !wallet.publicKey) return null;

    try {
      const buyOrderPDA = getBuyOrderPDA(wallet.publicKey, orderId);
      const orderAccount = await program.account.buyOrder.fetch(buyOrderPDA);
      
      return {
        orderId: orderAccount.orderId.toNumber(),
        stockSymbol: orderAccount.stockSymbol,
        solAmount: orderAccount.solAmount.toNumber() / LAMPORTS_PER_SOL,
        status: orderAccount.status,
        sharesReceived: orderAccount.sharesReceived.toNumber(),
        timestamp: new Date(orderAccount.timestamp.toNumber() * 1000),
      };
    } catch (error) {
      return null;
    }
  }, [program, wallet.publicKey, getBuyOrderPDA]);

  return {
    program,
    provider,
    isConnected: !!wallet.connected,
    publicKey: wallet.publicKey,
    
    // Contract interactions
    placeBuyOrder,
    getUserStockBalance,
    getOrderStatus,
    
    // PDA helpers
    getTradingPoolPDA,
    getBuyOrderPDA,
    getStockMintPDA,
  };
} 