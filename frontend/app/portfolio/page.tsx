'use client';

import { useEffect, useState } from 'react';
import { useWallet } from '@/lib/wallet';
import { getOpenOrders, getPnl, type Order, type Position } from '@/lib/api';
import { clsx } from 'clsx';

function bpsToCents(bps: number): string {
  return `${(bps / 100).toFixed(0)}¢`;
}

export default function PortfolioPage() {
  const { address, isConnected, connect } = useWallet();
  const [orders, setOrders] = useState<Order[]>([]);
  const [positions, setPositions] = useState<Position[]>([]);
  const [totalPnl, setTotalPnl] = useState<string>('0.0000000');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!address) { setLoading(false); return; }

    const load = async () => {
      try {
        const [ordersRes, pnlRes] = await Promise.all([
          getOpenOrders(address),
          getPnl(address),
        ]);
        setOrders(ordersRes.data);
        setPositions(pnlRes.data.positions);
        setTotalPnl(pnlRes.data.total_unrealised_pnl_usdc);
      } catch (err) {
        console.error(err);
      } finally {
        setLoading(false);
      }
    };

    load();
  }, [address]);

  if (!isConnected) {
    return (
      <div className="text-center py-20">
        <p className="text-gray-400 mb-4">Connect your wallet to view your portfolio</p>
        <button onClick={connect} className="btn-primary">Connect Wallet</button>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="space-y-4 animate-pulse">
        {[1,2,3].map(i => <div key={i} className="h-20 bg-gray-800 rounded-xl" />)}
      </div>
    );
  }

  const pnlNum = Number(totalPnl);

  return (
    <div className="max-w-4xl mx-auto space-y-8">
      <div>
        <h1 className="text-2xl font-bold text-white mb-1">Portfolio</h1>
        <p className="text-sm text-gray-500">
          Unrealised P&L:{' '}
          <span className={clsx('font-medium', pnlNum >= 0 ? 'text-green-400' : 'text-red-400')}>
            {pnlNum >= 0 ? '+' : ''}{pnlNum.toFixed(2)} USDC
          </span>
        </p>
      </div>

      {/* Open Orders */}
      <section>
        <h2 className="text-lg font-semibold mb-3">Open Orders</h2>
        {orders.length === 0 ? (
          <div className="card text-center text-gray-500 text-sm py-8">
            No open orders.{' '}
            <a href="/" className="text-blue-400 underline">Browse markets</a> to place one.
          </div>
        ) : (
          <div className="card overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-xs text-gray-500 border-b border-gray-800">
                  <th className="text-left pb-2 pr-4">Market</th>
                  <th className="text-left pb-2 pr-4">Outcome</th>
                  <th className="text-left pb-2 pr-4">Side</th>
                  <th className="text-right pb-2 pr-4">Price</th>
                  <th className="text-right pb-2 pr-4">Qty</th>
                  <th className="text-right pb-2">Filled</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-800">
                {orders.map((o) => (
                  <tr key={o.order_id} className="text-gray-300">
                    <td className="py-2 pr-4">
                      <a href={`/markets/${o.market_id}`} className="text-blue-400 hover:underline">
                        #{o.market_id}
                      </a>
                    </td>
                    <td className="py-2 pr-4">{o.outcome_id === 0 ? 'YES' : 'NO'}</td>
                    <td className="py-2 pr-4">
                      <span className={clsx(
                        'capitalize text-xs font-medium',
                        o.side === 'buy' ? 'text-green-400' : 'text-red-400',
                      )}>
                        {o.side}
                      </span>
                    </td>
                    <td className="py-2 pr-4 text-right font-mono">{bpsToCents(o.price_bps)}</td>
                    <td className="py-2 pr-4 text-right font-mono">{o.quantity}</td>
                    <td className="py-2 text-right font-mono">{o.filled_quantity}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {/* Positions */}
      <section>
        <h2 className="text-lg font-semibold mb-3">Positions</h2>
        {positions.length === 0 ? (
          <div className="card text-center text-gray-500 text-sm py-8">
            No positions yet.
          </div>
        ) : (
          <div className="space-y-3">
            {positions.map((p) => {
              const pnl = Number(p.unrealised_pnl_usdc ?? 0);
              const isWinner = p.market_status === 'resolved' && p.winning_outcome === p.outcome_id;
              const isLoser = p.market_status === 'resolved' && p.winning_outcome !== p.outcome_id;

              return (
                <div key={`${p.market_id}-${p.outcome_id}`} className="card">
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 min-w-0">
                      <a href={`/markets/${p.market_id}`} className="text-sm text-white hover:text-blue-400 font-medium line-clamp-1">
                        {p.question}
                      </a>
                      <p className="text-xs text-gray-500 mt-0.5">
                        {p.outcomes[p.outcome_id]} · {Number(p.token_balance).toLocaleString()} shares
                      </p>
                    </div>
                    <div className="text-right shrink-0">
                      {isWinner && (
                        <span className="text-xs text-green-400 font-medium">
                          Winner 🎉
                        </span>
                      )}
                      {isLoser && (
                        <span className="text-xs text-red-400 font-medium">Resolved</span>
                      )}
                      {p.market_status === 'active' && (
                        <span className={clsx(
                          'text-xs font-medium',
                          pnl >= 0 ? 'text-green-400' : 'text-red-400',
                        )}>
                          {pnl >= 0 ? '+' : ''}{pnl.toFixed(2)} USDC
                        </span>
                      )}
                      {isWinner && (
                        <button className="block mt-1 btn-primary text-xs py-1 px-3">
                          Claim
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </section>
    </div>
  );
}