'use client';

import { useEffect, useRef, useState } from 'react';
import type { OrderBook as OrderBookType, PriceLevel } from '@/lib/api';

interface OrderBookProps {
  initialData: OrderBookType;
  marketId: number;
  outcomeId: number;
}

function bpsToCents(bps: number): string {
  return (bps / 100).toFixed(0) + '¢';
}

function PriceLevelRow({
  level,
  side,
  maxQty,
}: {
  level: PriceLevel;
  side: 'bid' | 'ask';
  maxQty: number;
}) {
  const pct = Math.min(100, (Number(level.quantity) / maxQty) * 100);
  const bg = side === 'bid' ? 'bg-green-500/10' : 'bg-red-500/10';
  const priceColor = side === 'bid' ? 'text-green-400' : 'text-red-400';

  return (
    <div className="relative flex justify-between items-center py-0.5 px-2 text-xs font-mono">
      {/* Background fill */}
      <div
        className={`absolute inset-y-0 ${side === 'bid' ? 'right-0' : 'left-0'} ${bg}`}
        style={{ width: `${pct}%` }}
      />
      <span className={`relative z-10 ${priceColor}`}>{bpsToCents(level.price_bps)}</span>
      <span className="relative z-10 text-gray-400">{Number(level.quantity).toLocaleString()}</span>
    </div>
  );
}

export function OrderBook({ initialData, marketId, outcomeId }: OrderBookProps) {
  const [orderBook, setOrderBook] = useState<OrderBookType>(initialData);
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    const wsUrl = process.env.NEXT_PUBLIC_WS_URL ?? 'ws://localhost:3002';
    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      ws.send(JSON.stringify({ type: 'subscribe', market_id: String(marketId) }));
    };

    ws.onmessage = (event: MessageEvent<string>) => {
      try {
        const msg = JSON.parse(event.data) as { type: string; [key: string]: unknown };
        if (msg.type === 'orderbook_update') {
          // TODO: apply incremental diff rather than full refresh
          setOrderBook((prev) => ({ ...prev }));
        }
      } catch {
        // ignore malformed messages
      }
    };

    return () => {
      ws.send(JSON.stringify({ type: 'unsubscribe', market_id: String(marketId) }));
      ws.close();
    };
  }, [marketId, outcomeId]);

  const maxBidQty = Math.max(...orderBook.bids.map((b) => Number(b.quantity)), 1);
  const maxAskQty = Math.max(...orderBook.asks.map((a) => Number(a.quantity)), 1);

  const bestBid = orderBook.bids[0]?.price_bps;
  const bestAsk = orderBook.asks[0]?.price_bps;
  const spread = bestBid && bestAsk ? bestAsk - bestBid : null;

  return (
    <div className="card">
      <h2 className="text-sm font-semibold text-gray-300 mb-3">Order Book</h2>

      {/* Column headers */}
      <div className="flex justify-between text-xs text-gray-600 px-2 mb-1">
        <span>Price</span>
        <span>Quantity</span>
      </div>

      {/* Asks (sell side) — shown top, ascending → lowest ask at bottom */}
      <div className="space-y-0.5 mb-1">
        {orderBook.asks.length === 0 ? (
          <p className="text-xs text-gray-600 px-2 py-1">No asks</p>
        ) : (
          [...orderBook.asks].reverse().map((level, i) => (
            <PriceLevelRow key={i} level={level} side="ask" maxQty={maxAskQty} />
          ))
        )}
      </div>

      {/* Spread */}
      <div className="flex items-center justify-center py-1.5 border-y border-gray-800 text-xs text-gray-500 my-1">
        {spread !== null ? (
          <>
            <span className="font-mono">Spread: {bpsToCents(spread)}</span>
          </>
        ) : (
          <span>No spread</span>
        )}
      </div>

      {/* Bids (buy side) — highest bid at top */}
      <div className="space-y-0.5 mt-1">
        {orderBook.bids.length === 0 ? (
          <p className="text-xs text-gray-600 px-2 py-1">No bids</p>
        ) : (
          orderBook.bids.map((level, i) => (
            <PriceLevelRow key={i} level={level} side="bid" maxQty={maxBidQty} />
          ))
        )}
      </div>
    </div>
  );
}