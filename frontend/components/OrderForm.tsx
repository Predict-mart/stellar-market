'use client';

import { useState, type FormEvent } from 'react';
import { clsx } from 'clsx';
import { useWallet } from '@/lib/wallet';
import { submitOrder } from '@/lib/api';

interface OrderFormProps {
  marketId: number;
  outcomes: string[];
}

type Side = 'buy' | 'sell';

export function OrderForm({ marketId, outcomes }: OrderFormProps) {
  const { isConnected, connect } = useWallet();

  const [outcomeId, setOutcomeId] = useState(0);
  const [side, setSide] = useState<Side>('buy');
  const [price, setPrice] = useState('');
  const [quantity, setQuantity] = useState('');
  const [status, setStatus] = useState<'idle' | 'signing' | 'submitting' | 'success' | 'error'>('idle');
  const [txHash, setTxHash] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const priceNum = parseFloat(price);
  const quantityNum = parseInt(quantity, 10);
  const priceValid = !isNaN(priceNum) && priceNum > 0 && priceNum < 1;
  const quantityValid = !isNaN(quantityNum) && quantityNum > 0;
  const formValid = priceValid && quantityValid;

  const estimatedCost = priceValid && quantityValid
    ? (priceNum * quantityNum).toFixed(2)
    : null;

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (!formValid || !isConnected) return;

    setStatus('signing');
    setErrorMsg(null);

    try {
      // TODO: Build real Soroban XDR transaction here
      // For now, this is a placeholder that demonstrates the full UI flow
      // See issue #26: Build order placement interface
      const placeholderXdr = 'PLACEHOLDER_XDR';

      setStatus('submitting');
      const result = await submitOrder(placeholderXdr);
      setTxHash(result.data.tx_hash);
      setStatus('success');
      setPrice('');
      setQuantity('');
    } catch (err) {
      setErrorMsg(err instanceof Error ? err.message : 'Submission failed');
      setStatus('error');
    }
  }

  if (!isConnected) {
    return (
      <div className="card text-center space-y-3">
        <p className="text-sm text-gray-400">Connect your wallet to trade</p>
        <button onClick={connect} className="btn-primary w-full">
          Connect Wallet
        </button>
      </div>
    );
  }

  return (
    <div className="card">
      <h2 className="text-sm font-semibold text-gray-300 mb-4">Place Order</h2>

      <form onSubmit={handleSubmit} className="space-y-4">
        {/* Outcome selector */}
        <div>
          <label className="block text-xs text-gray-500 mb-1">Outcome</label>
          <div className="flex gap-2">
            {outcomes.map((label, idx) => (
              <button
                key={idx}
                type="button"
                onClick={() => setOutcomeId(idx)}
                className={clsx(
                  'flex-1 py-2 rounded-lg text-sm font-medium border transition-colors',
                  outcomeId === idx
                    ? idx === 0
                      ? 'bg-green-900/50 border-green-600 text-green-300'
                      : 'bg-red-900/50 border-red-600 text-red-300'
                    : 'bg-gray-800 border-gray-700 text-gray-400 hover:border-gray-500',
                )}
              >
                {label}
              </button>
            ))}
          </div>
        </div>

        {/* Side selector */}
        <div>
          <label className="block text-xs text-gray-500 mb-1">Side</label>
          <div className="flex gap-2">
            {(['buy', 'sell'] as Side[]).map((s) => (
              <button
                key={s}
                type="button"
                onClick={() => setSide(s)}
                className={clsx(
                  'flex-1 py-2 rounded-lg text-sm font-medium border transition-colors capitalize',
                  side === s
                    ? 'bg-blue-900/50 border-blue-600 text-blue-300'
                    : 'bg-gray-800 border-gray-700 text-gray-400 hover:border-gray-500',
                )}
              >
                {s}
              </button>
            ))}
          </div>
        </div>

        {/* Price */}
        <div>
          <label className="block text-xs text-gray-500 mb-1">
            Price (0.01 – 0.99)
          </label>
          <div className="relative">
            <input
              type="number"
              value={price}
              onChange={(e) => setPrice(e.target.value)}
              step="0.01"
              min="0.01"
              max="0.99"
              placeholder="0.65"
              className={clsx(
                'w-full bg-gray-800 border rounded-lg px-3 py-2 text-sm font-mono',
                'focus:outline-none focus:ring-1 focus:ring-blue-500',
                price && !priceValid ? 'border-red-600' : 'border-gray-700',
              )}
            />
            {priceValid && (
              <span className="absolute right-3 top-2 text-xs text-gray-500">
                {(priceNum * 100).toFixed(0)}% probability
              </span>
            )}
          </div>
        </div>

        {/* Quantity */}
        <div>
          <label className="block text-xs text-gray-500 mb-1">Shares</label>
          <input
            type="number"
            value={quantity}
            onChange={(e) => setQuantity(e.target.value)}
            min="1"
            step="1"
            placeholder="100"
            className={clsx(
              'w-full bg-gray-800 border rounded-lg px-3 py-2 text-sm font-mono',
              'focus:outline-none focus:ring-1 focus:ring-blue-500',
              quantity && !quantityValid ? 'border-red-600' : 'border-gray-700',
            )}
          />
        </div>

        {/* Estimated cost */}
        {estimatedCost && (
          <div className="bg-gray-800/50 rounded-lg px-3 py-2 text-xs flex justify-between text-gray-400">
            <span>Estimated {side === 'buy' ? 'cost' : 'proceeds'}</span>
            <span className="font-mono text-white">${estimatedCost} USDC</span>
          </div>
        )}

        {/* Submit */}
        <button
          type="submit"
          disabled={!formValid || status === 'signing' || status === 'submitting'}
          className="btn-primary w-full"
        >
          {status === 'signing' ? 'Waiting for signature...' :
           status === 'submitting' ? 'Submitting...' :
           `Place ${side.charAt(0).toUpperCase() + side.slice(1)} Order`}
        </button>

        {/* Status feedback */}
        {status === 'success' && txHash && (
          <p className="text-xs text-green-400 text-center">
            ✓ Order submitted!{' '}
            <a
              href={`https://stellar.expert/explorer/testnet/tx/${txHash}`}
              target="_blank"
              rel="noreferrer"
              className="underline"
            >
              View transaction
            </a>
          </p>
        )}
        {status === 'error' && (
          <p className="text-xs text-red-400 text-center">{errorMsg}</p>
        )}
      </form>
    </div>
  );
}