import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import morgan from 'morgan';
import { marketsRouter } from './api/routes/markets';
import { ordersRouter } from './api/routes/orders';
import { positionsRouter } from './api/routes/positions';
import { config } from './config';

export function createApp(): express.Application {
  const app = express();

  // ── Security middleware ──────────────────────────────────────
  app.use(helmet());
  app.use(cors({
    origin: process.env.NODE_ENV === 'production'
      ? ['https://stellarmarket.io']
      : true,
    credentials: true,
  }));

  // ── Parsing ─────────────────────────────────────────────────
  app.use(express.json({ limit: '1mb' }));
  app.use(express.urlencoded({ extended: false }));

  // ── Logging ─────────────────────────────────────────────────
  if (config.NODE_ENV !== 'test') {
    app.use(morgan('combined'));
  }

  // ── Health check ────────────────────────────────────────────
  app.get('/health', (_req, res) => {
    res.json({
      status: 'ok',
      version: process.env.npm_package_version ?? '0.1.0',
      network: config.STELLAR_NETWORK,
      timestamp: new Date().toISOString(),
    });
  });

  // ── API routes ───────────────────────────────────────────────
  app.use('/markets', marketsRouter);
  app.use('/orders', ordersRouter);
  app.use('/positions', positionsRouter);

  // ── 404 ─────────────────────────────────────────────────────
  app.use((_req, res) => {
    res.status(404).json({ error: 'Not found' });
  });

  // ── Error handler ────────────────────────────────────────────
  app.use((err: Error, _req: express.Request, res: express.Response, _next: express.NextFunction) => {
    console.error(err);
    res.status(500).json({ error: 'Internal server error' });
  });

  return app;
}