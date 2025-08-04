'use client';

import { useAccount, useConnect } from 'wagmi';
import { Token } from '@/types';

interface SwapButtonProps {
  onClick: () => void;
  disabled: boolean;
  loading: boolean;
  fromToken: Token | null;
  toToken: Token | null;
}

export default function SwapButton({
  onClick,
  disabled,
  loading,
  fromToken,
  toToken
}: SwapButtonProps) {
  const { address, isConnected } = useAccount();
  const { connect, connectors } = useConnect();

  if (!isConnected) {
    return (
      <button
        onClick={() => connect({ connector: connectors[0] })}
        className="w-full py-4 mt-4 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 rounded-2xl font-semibold text-white transition-colors"
      >
        Connect Wallet!
      </button>
    );
  }

  const buttonText = () => {
    if (loading) return 'Swapping...';
    if (!fromToken || !toToken) return 'Select tokens';
    return `Swap ${fromToken.symbol} for ${toToken.symbol}`;
  };

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className="w-full py-4 mt-4 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed rounded-2xl font-semibold text-white transition-colors flex items-center justify-center gap-2"
    >
      {loading && (
        <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-white"></div>
      )}
      {buttonText()}
    </button>
  );
}