import { Router, Request, Response } from 'express';
import { db } from '../../db/client';

export const positionsRouter = Router();

// ── GET /positions/:address — all positions for a trader ──────────────────────

positionsRouter.get('/:address', async (req: Request, res: Response) => {
  const { address } = req.params;

  const { rows } = await db.query(
    `SELECT
       p.*,
       m.question,
       m.outcomes,
       m.status AS market_status,
       m.winning_outcome
     FROM positions p
     JOIN markets m ON m.market_id = p.market_id
     WHERE p.trader_address = $1
       AND p.token_balance > 0
     ORDER BY p.updated_at DESC`,
    [address],
  );

  return res.json({ data: rows });
});

// ── GET /positions/:address/pnl — unrealised P&L ─────────────────────────────

positionsRouter.get('/:address/pnl', async (req: Request, res: Response) => {
  const { address } = req.params;

  // Join positions with the current best mid-price from the order book
  const { rows } = await db.query(
    `WITH mid_prices AS (
       SELECT
         o.market_id,
         o.outcome_id,
         (
           COALESCE(
             (SELECT price_bps FROM orders
              WHERE market_id = o.market_id AND outcome_id = o.outcome_id
                AND side = 'buy' AND status = 'open'
              ORDER BY price_bps DESC LIMIT 1),
             0
           )
           +
           COALESCE(
             (SELECT price_bps FROM orders
              WHERE market_id = o.market_id AND outcome_id = o.outcome_id
                AND side = 'sell' AND status = 'open'
              ORDER BY price_bps ASC LIMIT 1),
             0
           )
         ) / 2.0 AS mid_price_bps
       FROM positions o
       WHERE o.trader_address = $1
     )
     SELECT
       p.*,
       m.question,
       mp.mid_price_bps,
       CASE
         WHEN p.avg_buy_price IS NOT NULL AND mp.mid_price_bps > 0
         THEN (mp.mid_price_bps - p.avg_buy_price) * p.token_balance::numeric / 10000.0
         ELSE 0
       END AS unrealised_pnl_usdc
     FROM positions p
     JOIN markets m ON m.market_id = p.market_id
     LEFT JOIN mid_prices mp
       ON mp.market_id = p.market_id AND mp.outcome_id = p.outcome_id
     WHERE p.trader_address = $1
       AND p.token_balance > 0`,
    [address],
  );

  const totalPnl = rows.reduce(
    (sum, row) => sum + Number(row.unrealised_pnl_usdc ?? 0),
    0,
  );

  return res.json({
    data: {
      positions: rows,
      total_unrealised_pnl_usdc: totalPnl.toFixed(7),
    },
  });
});