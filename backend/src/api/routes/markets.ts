import { Router, Request, Response } from 'express';
import { z } from 'zod';
import { db } from '../../db/client';

export const marketsRouter = Router();

// ── Query schemas ────────────────────────────────────────────────────────────

const listMarketsSchema = z.object({
  status: z.enum(['pending','active','closed','resolved','settled']).optional(),
  category: z.enum(['elections','sports','finance','tech','global','other']).optional(),
  sort: z.enum(['volume','end_date','newest']).default('newest'),
  page: z.coerce.number().int().min(1).default(1),
  limit: z.coerce.number().int().min(1).max(100).default(20),
  q: z.string().max(200).optional(),
});

// ── GET /markets ──────────────────────────────────────────────────────────────

marketsRouter.get('/', async (req: Request, res: Response) => {
  const parse = listMarketsSchema.safeParse(req.query);
  if (!parse.success) {
    return res.status(400).json({ error: 'Invalid query params', details: parse.error.flatten() });
  }

  const { status, category, sort, page, limit, q } = parse.data;
  const offset = (page - 1) * limit;

  const conditions: string[] = [];
  const params: unknown[] = [];
  let idx = 1;

  if (status) { conditions.push(`status = $${idx++}`); params.push(status); }
  if (category) { conditions.push(`category = $${idx++}`); params.push(category); }
  if (q) { conditions.push(`question ILIKE $${idx++}`); params.push(`%${q}%`); }

  const where = conditions.length ? `WHERE ${conditions.join(' AND ')}` : '';

  const orderMap: Record<string, string> = {
    volume: 'volume_24h_usdc DESC',
    end_date: 'resolution_date ASC',
    newest: 'created_at DESC',
  };
  const orderBy = orderMap[sort];

  const [{ rows: markets }, { rows: [{ count }] }] = await Promise.all([
    db.query(
      `SELECT * FROM markets ${where} ORDER BY ${orderBy} LIMIT $${idx++} OFFSET $${idx++}`,
      [...params, limit, offset],
    ),
    db.query(`SELECT COUNT(*) FROM markets ${where}`, params),
  ]);

  return res.json({
    data: markets,
    pagination: {
      page,
      limit,
      total: Number(count),
      pages: Math.ceil(Number(count) / limit),
    },
  });
});

// ── GET /markets/:id ──────────────────────────────────────────────────────────

marketsRouter.get('/:id', async (req: Request, res: Response) => {
  const marketId = Number(req.params.id);
  if (isNaN(marketId)) return res.status(400).json({ error: 'Invalid market ID' });

  const { rows } = await db.query('SELECT * FROM markets WHERE market_id = $1', [marketId]);
  if (!rows.length) return res.status(404).json({ error: 'Market not found' });

  return res.json({ data: rows[0] });
});

// ── GET /markets/:id/orderbook ────────────────────────────────────────────────

marketsRouter.get('/:id/orderbook', async (req: Request, res: Response) => {
  const marketId = Number(req.params.id);
  if (isNaN(marketId)) return res.status(400).json({ error: 'Invalid market ID' });

  const outcomeId = req.query.outcome_id !== undefined ? Number(req.query.outcome_id) : undefined;
  const outcomeFilter = outcomeId !== undefined ? 'AND outcome_id = $2' : '';
  const params: unknown[] = outcomeId !== undefined ? [marketId, outcomeId] : [marketId];

  const bidsQuery = db.query(
    `SELECT price_bps, SUM(quantity - filled_quantity) AS quantity
     FROM orders
     WHERE market_id = $1 AND side = 'buy' AND status = 'open' ${outcomeFilter}
     GROUP BY price_bps ORDER BY price_bps DESC LIMIT 20`,
    params,
  );

  const asksQuery = db.query(
    `SELECT price_bps, SUM(quantity - filled_quantity) AS quantity
     FROM orders
     WHERE market_id = $1 AND side = 'sell' AND status = 'open' ${outcomeFilter}
     GROUP BY price_bps ORDER BY price_bps ASC LIMIT 20`,
    params,
  );

  const [bids, asks] = await Promise.all([bidsQuery, asksQuery]);

  return res.json({
    data: {
      market_id: marketId,
      outcome_id: outcomeId,
      bids: bids.rows,
      asks: asks.rows,
    },
  });
});

// ── GET /markets/:id/trades ───────────────────────────────────────────────────

marketsRouter.get('/:id/trades', async (req: Request, res: Response) => {
  const marketId = Number(req.params.id);
  if (isNaN(marketId)) return res.status(400).json({ error: 'Invalid market ID' });

  const page = Math.max(1, Number(req.query.page) || 1);
  const limit = Math.min(100, Number(req.query.limit) || 50);
  const offset = (page - 1) * limit;

  const { rows } = await db.query(
    `SELECT * FROM trades WHERE market_id = $1 ORDER BY executed_at DESC LIMIT $2 OFFSET $3`,
    [marketId, limit, offset],
  );

  return res.json({ data: rows });
});

// ── GET /markets/:id/stats ────────────────────────────────────────────────────

marketsRouter.get('/:id/stats', async (req: Request, res: Response) => {
  const marketId = Number(req.params.id);
  if (isNaN(marketId)) return res.status(400).json({ error: 'Invalid market ID' });

  const { rows: volumeRows } = await db.query(
    `SELECT outcome_id,
            SUM(quantity * price_bps / 10000.0) AS volume_usdc,
            MAX(price_bps) AS last_price_bps,
            COUNT(*) AS trade_count
     FROM trades
     WHERE market_id = $1 AND executed_at > NOW() - INTERVAL '24 hours'
     GROUP BY outcome_id`,
    [marketId],
  );

  const { rows: openInterestRows } = await db.query(
    `SELECT outcome_id, SUM(quantity - filled_quantity) AS open_quantity
     FROM orders
     WHERE market_id = $1 AND status = 'open'
     GROUP BY outcome_id`,
    [marketId],
  );

  return res.json({
    data: {
      market_id: marketId,
      volume_24h: volumeRows,
      open_interest: openInterestRows,
    },
  });
});