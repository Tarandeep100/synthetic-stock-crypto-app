'use client';

import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { Program, AnchorProvider, Idl, setProvider } from '@coral-xyz/anchor';
import { useMemo } from 'react';

// Import your IDL here when smart contracts are deployed
// import { StockContracts } from '@/anchor-idl/stock_contracts';

// TODO: Replace with your actual deployed program ID
const PROGRAM_ID = new PublicKey('11111111111111111111111111111111');

export function useProgram() {
  const { connection } = useConnection();
  const wallet = useWallet();

  const provider = useMemo(() => {
    if (!wallet.publicKey) return null;
    
    return new AnchorProvider(connection, wallet as any, {
      commitment: 'confirmed',
    });
  }, [connection, wallet]);

  const program = useMemo(() => {
    if (!provider) return null;
    
    try {
      setProvider(provider);
      
      // TODO: Initialize actual program with IDL when contracts are deployed
      // return new Program(StockContracts as Idl, PROGRAM_ID, provider);
      
      throw new Error('Smart contracts not yet deployed. Please deploy your Solana program first.');
      
    } catch (error) {
      console.error('Failed to initialize program:', error);
      return null;
    }
  }, [provider]);

  return {
    program,
    provider,
    programId: PROGRAM_ID,
    connected: wallet.connected,
    publicKey: wallet.publicKey,
  };
} 