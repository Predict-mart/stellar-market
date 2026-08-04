import dotenv from 'dotenv';
import { z } from 'zod';

dotenv.config();

const configSchema = z.object({
  NODE_ENV: z.enum(['development', 'test', 'production']).default('development'),
  PORT: z.coerce.number().default(3001),
  WS_PORT: z.coerce.number().default(3002),
  DATABASE_URL: z.string().min(1),
  REDIS_URL: z.string().default('redis://localhost:6379'),
  STELLAR_NETWORK: z.enum(['local', 'testnet', 'mainnet']).default('local'),
  HORIZON_URL: z.string().url(),
  POLL_INTERVAL_MS: z.coerce.number().default(2000),
  MARKET_FACTORY_ADDRESS: z.string().optional(),
  ORACLE_ADDRESS: z.string().optional(),
  SETTLEMENT_ADDRESS: z.string().optional(),
  GOVERNANCE_ADDRESS: z.string().optional(),
  KEEPER_SECRET_KEY: z.string().optional(),
  FEE_RECIPIENT_ADDRESS: z.string().optional(),
  ALERT_WEBHOOK_URL: z.string().url().optional(),
  RATE_LIMIT_WINDOW_MS: z.coerce.number().default(60_000),
  RATE_LIMIT_MAX: z.coerce.number().default(100),
});

const parsed = configSchema.safeParse(process.env);

if (!parsed.success) {
  console.error('❌ Invalid environment configuration:');
  console.error(parsed.error.format());
  process.exit(1);
}

export const config = parsed.data;
export type Config = typeof config;