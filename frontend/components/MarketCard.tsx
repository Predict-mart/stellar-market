import Link from 'next/link';
import { clsx } from 'clsx';
import type { Market } from '@/lib/api';

const STATUS_BADGE: Record<string, string> = {
  active:   'badge-green',
  pending:  'badge-yellow',
  closed:   'badge-gray',
  resolved: 'badge-blue',
  settled:  'badge-gray',
  rejected: 'badge-red',
};

function formatVolume(usdc: string): string {
  const n = Number(usdc);
  if (n >= 1_000_000) return `$${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `$${(n / 1_000).toFixed(1)}K`;
  return `$${n.toFixed(0)}`;
}

function formatTimeLeft(unixTs: number): string {
  const diff = unixTs * 1000 - Date.now();
  if (diff <= 0) return 'Closed';
  const days = Math.floor(diff / 86_400_000);
  if (days > 0) return `${days}d left`;
  const hours = Math.floor(diff / 3_600_000);
  return `${hours}h left`;
}

interface MarketCardProps {
  market: Market;
}

export function MarketCard({ market }: MarketCardProps) {
  const yesPricePct = 50; // TODO: pull from order book
  const noPricePct  = 50;

  return (
    <Link
      href={`/markets/${market.market_id}`}
      className="card hover:border-gray-600 transition-all hover:shadow-lg hover:shadow-blue-950/30 block"
    >
      {/* Header */}
      <div className="flex items-start justify-between gap-2 mb-3">
        <span className={clsx('badge capitalize', STATUS_BADGE[market.status] ?? 'badge-gray')}>
          {market.status}
        </span>
        <span className="badge badge-gray capitalize">{market.category}</span>
      </div>

      {/* Question */}
      <p className="text-white font-medium text-sm leading-snug line-clamp-2 mb-4">
        {market.question}
      </p>

      {/* Prices */}
      <div className="flex gap-2 mb-4">
        <div className="flex-1 bg-green-950/30 border border-green-900/40 rounded-lg p-2 text-center">
          <div className="text-green-400 font-bold text-lg">{yesPricePct}¢</div>
          <div className="text-green-600 text-xs">YES</div>
        </div>
        <div className="flex-1 bg-red-950/30 border border-red-900/40 rounded-lg p-2 text-center">
          <div className="text-red-400 font-bold text-lg">{noPricePct}¢</div>
          <div className="text-red-600 text-xs">NO</div>
        </div>
      </div>

      {/* Footer */}
      <div className="flex items-center justify-between text-xs text-gray-500">
        <span>Vol: {formatVolume(market.volume_24h_usdc)}</span>
        <span>{formatTimeLeft(market.resolution_date)}</span>
      </div>
    </Link>
  );
}

export function MarketCardSkeleton() {
  return (
    <div className="card animate-pulse">
      <div className="flex gap-2 mb-3">
        <div className="h-5 w-16 bg-gray-800 rounded-full" />
        <div className="h-5 w-16 bg-gray-800 rounded-full" />
      </div>
      <div className="h-4 bg-gray-800 rounded mb-2" />
      <div className="h-4 bg-gray-800 rounded w-3/4 mb-4" />
      <div className="flex gap-2 mb-4">
        <div className="flex-1 h-16 bg-gray-800 rounded-lg" />
        <div className="flex-1 h-16 bg-gray-800 rounded-lg" />
      </div>
      <div className="flex justify-between">
        <div className="h-3 w-16 bg-gray-800 rounded" />
        <div className="h-3 w-16 bg-gray-800 rounded" />
      </div>
    </div>
  );
}