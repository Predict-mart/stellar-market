const BASE_URL = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:3001';

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE_URL}${path}`, {
    headers: { 'Content-Type': 'application/json', ...init?.headers },
    ...init,
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText })) as { error: string };
    throw new Error(err.error ?? `API error ${res.status}`);
  }
  return res.json() as Promise<T>;
}

// ── Types ─────────────────────────────────────────────────────────────────────

export interface Market {
  market_id: number;
  question: string;
  outcomes: string[];
  status: 'pending' | 'active' | 'closed' | 'resolved' | 'settled' | 'rejected';
  category: string;
  resolution_date: number;
  winning_outcome: number | null;
  total_pool_usdc: string;
  volume_24h_usdc: string;
  created_at: string;
}

export interface PriceLevel {
  price_bps: number;
  quantity: string;
}

export interface OrderBook {
  market_id: number;
  outcome_id?: number;
  bids: PriceLevel[];
  asks: PriceLevel[];
}

export interface Trade {
  id: number;
  market_id: number;
  outcome_id: number;
  price_bps: number;
  quantity: number;
  executed_at: string;
}

export interface Order {
  order_id: string;
  market_id: number;
  outcome_id: number;
  trader_address: string;
  side: 'buy' | 'sell';
  price_bps: number;
  quantity: number;
  filled_quantity: number;
  status: 'open' | 'filled' | 'cancelled';
  created_at: string;
}

export interface Position {
  trader_address: string;
  market_id: number;
  outcome_id: number;
  token_balance: string;
  avg_buy_price: number | null;
  question: string;
  outcomes: string[];
  market_status: string;
  winning_outcome: number | null;
  unrealised_pnl_usdc?: string;
}

export interface Pagination {
  page: number;
  limit: number;
  total: number;
  pages: number;
}

// ── Markets ───────────────────────────────────────────────────────────────────

export interface ListMarketsParams {
  status?: string;
  category?: string;
  sort?: 'volume' | 'end_date' | 'newest';
  page?: number;
  limit?: number;
  q?: string;
}

export async function listMarkets(params: ListMarketsParams = {}): Promise<{ data: Market[]; pagination: Pagination }> {
  const qs = new URLSearchParams(
    Object.entries(params)
      .filter(([, v]) => v !== undefined)
      .map(([k, v]) => [k, String(v)]),
  ).toString();
  return apiFetch(`/markets${qs ? `?${qs}` : ''}`);
}

export async function getMarket(id: number): Promise<{ data: Market }> {
  return apiFetch(`/markets/${id}`);
}

export async function getOrderBook(id: number, outcomeId?: number): Promise<{ data: OrderBook }> {
  const qs = outcomeId !== undefined ? `?outcome_id=${outcomeId}` : '';
  return apiFetch(`/markets/${id}/orderbook${qs}`);
}

export async function getTrades(id: number, page = 1): Promise<{ data: Trade[] }> {
  return apiFetch(`/markets/${id}/trades?page=${page}`);
}

// ── Orders ────────────────────────────────────────────────────────────────────

export async function submitOrder(xdr: string): Promise<{ data: { tx_hash: string; status: string } }> {
  return apiFetch('/orders', { method: 'POST', body: JSON.stringify({ xdr }) });
}

export async function getOpenOrders(address: string): Promise<{ data: Order[] }> {
  return apiFetch(`/orders/${address}`);
}

export async function cancelOrder(orderId: string, xdr: string): Promise<{ data: unknown }> {
  return apiFetch(`/orders/${orderId}`, { method: 'DELETE', body: JSON.stringify({ xdr }) });
}

// ── Positions ─────────────────────────────────────────────────────────────────

export async function getPositions(address: string): Promise<{ data: Position[] }> {
  return apiFetch(`/positions/${address}`);
}

export async function getPnl(address: string): Promise<{ data: { positions: Position[]; total_unrealised_pnl_usdc: string } }> {
  return apiFetch(`/positions/${address}/pnl`);
}