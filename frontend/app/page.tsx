import { Suspense } from 'react';
import { MarketCard, MarketCardSkeleton } from '@/components/MarketCard';
import { listMarkets, type Market } from '@/lib/api';

const CATEGORIES = ['All', 'Elections', 'Sports', 'Finance', 'Tech', 'Global', 'Other'] as const;

async function MarketGrid({ status, category, sort }: { status?: string; category?: string; sort: string }) {
  let markets: Market[] = [];

  try {
    const res = await listMarkets({
      status: status ?? 'active',
      category: category !== 'All' ? category?.toLowerCase() : undefined,
      sort: sort as 'volume' | 'end_date' | 'newest',
      limit: 24,
    });
    markets = res.data;
  } catch {
    // API not yet running — return empty state gracefully
  }

  if (markets.length === 0) {
    return (
      <div className="col-span-full text-center py-16 text-gray-500">
        <p className="text-4xl mb-3">📊</p>
        <p className="text-lg font-medium">No markets found</p>
        <p className="text-sm mt-1">Be the first to{' '}
          <a href="/propose" className="text-blue-400 underline">propose a market</a>
        </p>
      </div>
    );
  }

  return (
    <>
      {markets.map((market) => (
        <MarketCard key={market.market_id} market={market} />
      ))}
    </>
  );
}

export default function HomePage({
  searchParams,
}: {
  searchParams: Record<string, string | undefined>;
}) {
  const sort     = searchParams.sort ?? 'newest';
  const category = searchParams.category ?? 'All';
  const status   = searchParams.status ?? 'active';

  return (
    <div>
      {/* Hero */}
      <div className="mb-8 text-center">
        <h1 className="text-3xl font-bold text-white mb-2">Prediction Markets</h1>
        <p className="text-gray-400 max-w-xl mx-auto text-sm">
          Trade on real-world outcomes with near-zero fees and on-chain settlement on Stellar.
        </p>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap gap-2 mb-4 items-center justify-between">
        {/* Category tabs */}
        <div className="flex gap-1 flex-wrap">
          {CATEGORIES.map((cat) => (
            <a
              key={cat}
              href={`/?category=${cat}&sort=${sort}`}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                category === cat
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-800 text-gray-400 hover:text-white'
              }`}
            >
              {cat}
            </a>
          ))}
        </div>

        {/* Sort */}
        <div className="flex gap-1">
          {[
            { value: 'newest', label: 'Newest' },
            { value: 'volume', label: 'Volume' },
            { value: 'end_date', label: 'Ending Soon' },
          ].map(({ value, label }) => (
            <a
              key={value}
              href={`/?category=${category}&sort=${value}`}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
                sort === value
                  ? 'bg-gray-700 text-white'
                  : 'text-gray-500 hover:text-white'
              }`}
            >
              {label}
            </a>
          ))}
        </div>
      </div>

      {/* Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        <Suspense fallback={
          <>
            {Array.from({ length: 6 }).map((_, i) => <MarketCardSkeleton key={i} />)}
          </>
        }>
          <MarketGrid status={status} category={category} sort={sort} />
        </Suspense>
      </div>
    </div>
  );
}