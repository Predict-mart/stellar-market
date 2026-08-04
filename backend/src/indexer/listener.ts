import { config } from '../config';
import { db } from '../db/client';
import { marketEvents } from '../websocket/server';

// ── Known contract event types ────────────────────────────────────────────────
const CONTRACT_EVENTS = [
  'MarketProposed',
  'MarketApproved',
  'MarketRejected',
  'OrderPlaced',
  'OrderCancelled',
  'TradeExecuted',
  'OracleReportSubmitted',
  'MarketResolved',
  'MarketSettled',
  'WinningsClaimed',
] as const;

type ContractEventType = typeof CONTRACT_EVENTS[number];

interface HorizonEvent {
  type: string;
  ledger: number;
  ledger_closed_at: string;
  paging_token: string;
  id: string;
  contract_id?: string;
  topic?: string[];
  value?: unknown;
}

let running = false;

// ── Get/set indexer cursor ────────────────────────────────────────────────────

async function getCursor(): Promise<string> {
  const { rows } = await db.query(
    "SELECT value FROM indexer_state WHERE key = 'last_cursor'",
  );
  return rows[0]?.value ?? '0';
}

async function saveCursor(cursor: string): Promise<void> {
  await db.query(
    "UPDATE indexer_state SET value = $1, updated_at = NOW() WHERE key = 'last_cursor'",
    [cursor],
  );
}

// ── Fetch events from Horizon ─────────────────────────────────────────────────

async function fetchEvents(cursor: string): Promise<HorizonEvent[]> {
  const params = new URLSearchParams({
    cursor,
    limit: '200',
    order: 'asc',
  });

  const response = await fetch(
    `${config.HORIZON_URL}/effects?${params.toString()}`,
  );

  if (!response.ok) {
    if (response.status === 429) {
      // Rate limited — back off
      await sleep(5_000);
      return [];
    }
    console.error(`Horizon returned ${response.status}`);
    return [];
  }

  const data = await response.json() as { _embedded?: { records: HorizonEvent[] } };
  return data._embedded?.records ?? [];
}

// ── Process a single event ────────────────────────────────────────────────────

async function processEvent(event: HorizonEvent): Promise<void> {
  const topic = event.topic?.[0];
  if (!topic || !CONTRACT_EVENTS.includes(topic as ContractEventType)) return;

  const eventType = topic as ContractEventType;
  const payload = event.value ?? {};

  // Persist raw event
  await db.query(
    `INSERT INTO events (contract_address, event_type, payload, ledger_seq, tx_hash)
     VALUES ($1, $2, $3, $4, $5)
     ON CONFLICT DO NOTHING`,
    [
      event.contract_id ?? '',
      eventType,
      JSON.stringify(payload),
      event.ledger,
      event.id,
    ],
  );

  // ── Dispatch to specific handlers ───────────────────────────────────────────
  switch (eventType) {
    case 'MarketApproved':
      await handleMarketApproved(payload as MarketApprovedPayload);
      break;
    case 'OrderPlaced':
      await handleOrderPlaced(payload as OrderPlacedPayload);
      break;
    case 'TradeExecuted':
      await handleTradeExecuted(payload as TradeExecutedPayload);
      break;
    case 'OrderCancelled':
      await handleOrderCancelled(payload as OrderCancelledPayload);
      break;
    case 'MarketResolved':
      await handleMarketResolved(payload as MarketResolvedPayload);
      break;
    default:
      break;
  }
}

// ── Event payload types ───────────────────────────────────────────────────────

interface MarketApprovedPayload { market_id: number; proposal_id: number }
interface OrderPlacedPayload { order_id: string; trader: string; market_id: number; outcome_id: number; side: string; price: number; quantity: number }
interface TradeExecutedPayload { buy_order_id: string; sell_order_id: string; market_id: number; outcome_id: number; price: number; quantity: number }
interface OrderCancelledPayload { order_id: string; trader: string }
interface MarketResolvedPayload { market_id: number; outcome_id: number }

// ── Handlers ─────────────────────────────────────────────────────────────────

async function handleMarketApproved(payload: MarketApprovedPayload): Promise<void> {
  await db.query(
    `UPDATE markets SET status = 'active', approved_at = NOW() WHERE market_id = $1`,
    [payload.market_id],
  );
}

async function handleOrderPlaced(payload: OrderPlacedPayload): Promise<void> {
  await db.query(
    `INSERT INTO orders (order_id, market_id, outcome_id, trader_address, side, price_bps, quantity)
     VALUES ($1, $2, $3, $4, $5, $6, $7)
     ON CONFLICT (order_id) DO NOTHING`,
    [payload.order_id, payload.market_id, payload.outcome_id, payload.trader, payload.side, payload.price, payload.quantity],
  );

  marketEvents.emit('orderbook_update', String(payload.market_id), {
    outcome_id: payload.outcome_id,
    side: payload.side,
    price_bps: payload.price,
    quantity: payload.quantity,
  });
}

async function handleTradeExecuted(payload: TradeExecutedPayload): Promise<void> {
  await db.query(
    `INSERT INTO trades (market_id, outcome_id, buy_order_id, sell_order_id, price_bps, quantity)
     VALUES ($1, $2, $3, $4, $5, $6)`,
    [payload.market_id, payload.outcome_id, payload.buy_order_id, payload.sell_order_id, payload.price, payload.quantity],
  );

  marketEvents.emit('trade', String(payload.market_id), {
    outcome_id: payload.outcome_id,
    price_bps: payload.price,
    quantity: payload.quantity,
    executed_at: new Date().toISOString(),
  });
}

async function handleOrderCancelled(payload: OrderCancelledPayload): Promise<void> {
  await db.query(
    `UPDATE orders SET status = 'cancelled', updated_at = NOW() WHERE order_id = $1`,
    [payload.order_id],
  );
}

async function handleMarketResolved(payload: MarketResolvedPayload): Promise<void> {
  await db.query(
    `UPDATE markets SET status = 'resolved', winning_outcome = $1, resolved_at = NOW()
     WHERE market_id = $2`,
    [payload.outcome_id, payload.market_id],
  );

  marketEvents.emit('market_resolved', String(payload.market_id), payload.outcome_id);
}

// ── Main poll loop ────────────────────────────────────────────────────────────

async function poll(): Promise<void> {
  const cursor = await getCursor();
  const events = await fetchEvents(cursor);

  if (events.length === 0) return;

  for (const event of events) {
    await processEvent(event);
  }

  const lastEvent = events[events.length - 1];
  if (lastEvent?.paging_token) {
    await saveCursor(lastEvent.paging_token);
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

export function startIndexer(): void {
  if (running) return;
  running = true;

  const run = async () => {
    while (running) {
      try {
        await poll();
      } catch (err) {
        console.error('Indexer error:', err);
      }
      await sleep(config.POLL_INTERVAL_MS);
    }
  };

  run().catch(console.error);
}

export function stopIndexer(): void {
  running = false;
}