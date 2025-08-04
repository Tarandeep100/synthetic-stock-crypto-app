'use client';

import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';

interface WalletButtonProps {
  className?: string;
}

export default function WalletButton({ className }: WalletButtonProps) {
  const { connected, connecting, publicKey } = useWallet();

  return (
    <div className={className}>
      <WalletMultiButton className="!bg-blue-600 hover:!bg-blue-700 !rounded-2xl !font-semibold !transition-colors" />
      
      {/* Optional: Show connection status */}
      {connected && publicKey && (
        <div className="mt-2 text-sm text-gray-400 text-center">
          Connected: {publicKey.toString().slice(0, 8)}...{publicKey.toString().slice(-8)}
        </div>
      )}
      
      {connecting && (
        <div className="mt-2 text-sm text-yellow-400 text-center">
          Connecting...
        </div>
      )}
    </div>
  );
} 