'use client';

import { useCallback } from 'react';

export interface TransactionToastProps {
  onSuccess?: (signature: string) => void;
  onError?: (error: Error) => void;
}

export function useTransactionToast() {
  const notifySuccess = useCallback((signature: string, message = 'Transaction successful!') => {
    // TODO: Replace with your preferred toast library (e.g., react-hot-toast, sonner)
    // For now, using browser notification
    if ('Notification' in window && Notification.permission === 'granted') {
      new Notification(message, {
        body: `Signature: ${signature.slice(0, 8)}...${signature.slice(-8)}`,
        icon: '/favicon.ico'
      });
    } else {
      // Fallback to alert if notifications aren't available
      alert(`${message}\nSignature: ${signature.slice(0, 8)}...${signature.slice(-8)}`);
    }
  }, []);

  const notifyError = useCallback((error: Error, message = 'Transaction failed!') => {
    // TODO: Replace with your preferred toast library
    // For now, using browser notification or alert
    if ('Notification' in window && Notification.permission === 'granted') {
      new Notification(message, {
        body: error.message,
        icon: '/favicon.ico'
      });
    } else {
      alert(`${message}\nError: ${error.message}`);
    }
  }, []);

  const notifyLoading = useCallback((message = 'Processing transaction...') => {
    // TODO: Implement loading state management
    // This could integrate with a global loading state or toast library
  }, []);

  return {
    notifySuccess,
    notifyError,
    notifyLoading,
  };
} 