import { Router, Request, Response } from 'express';
import { z } from 'zod';
import { db } from '../../db/client';
import { config } from '../../config';

export const ordersRouter = Router();

const submitOrderSchema = z.object({
  xdr: z.string().min(1, 'XDR transaction is required'),
});

// ── POST /orders — submit a pre-signed XDR transaction ───────────────────────

ordersRouter.post('/', async (req: Request, res: Response) => {
  const parse = submitOrderSchema.safeParse(req.body);
  if (!parse.success) {
    return res.status(400).json({ error: 'Invalid request', details: parse.error.flatten() });
  }

  const { xdr } = parse.data;

  try {
    // Submit to Stellar Horizon
    const response = await fetch(`${config.HORIZON_URL}/transactions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({ tx: xdr }),
    });

    const result = await response.json() as { hash?: string; status?: string; extras?: unknown };

    if (!response.ok) {
      return res.status(400).json({
        error: 'Transaction rejected by Stellar network',
        details: result,
      });
    }

    return res.status(201).json({
      data: {
        tx_hash: result.hash,
        status: 'submitted',
      },
    });
  } catch (err) {
    console.error('Error submitting transaction:', err);
    return res.status(502).json({ error: 'Failed to submit transaction to Stellar network' });
  }
});

// ── GET /orders/:address — open orders for a trader ──────────────────────────

ordersRouter.get('/:address', async (req: Request, res: Response) => {
  const { address } = req.params;
  const marketId = req.query.market_id ? Number(req.query.market_id) : undefined;

  const conditions = ['trader_address = $1', "status = 'open'"];
  const params: unknown[] = [address];
  let idx = 2;

  if (marketId !== undefined) {
    conditions.push(`market_id = $${idx++}`);
    params.push(marketId);
  }

  const { rows } = await db.query(
    `SELECT * FROM orders WHERE ${conditions.join(' AND ')} ORDER BY created_at DESC`,
    params,
  );

  return res.json({ data: rows });
});

// ── DELETE /orders/:orderId — cancel a resting order ─────────────────────────

ordersRouter.delete('/:orderId', async (req: Request, res: Response) => {
  const { orderId } = req.params;
  const { xdr } = req.body as { xdr?: string };

  if (!xdr) {
    return res.status(400).json({ error: 'Signed XDR transaction required in request body' });
  }

  try {
    const response = await fetch(`${config.HORIZON_URL}/transactions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({ tx: xdr }),
    });

    const result = await response.json() as { hash?: string };

    if (!response.ok) {
      return res.status(400).json({ error: 'Cancel transaction rejected', details: result });
    }

    return res.json({
      data: {
        order_id: orderId,
        tx_hash: result.hash,
        status: 'cancellation_submitted',
      },
    });
  } catch (err) {
    console.error('Error submitting cancel transaction:', err);
    return res.status(502).json({ error: 'Failed to submit cancellation to Stellar network' });
  }
});