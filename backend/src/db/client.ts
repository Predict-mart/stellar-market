import { Pool } from 'pg';
import { config } from '../config';

export const db = new Pool({
  connectionString: config.DATABASE_URL,
  max: 20,
  idleTimeoutMillis: 30_000,
  connectionTimeoutMillis: 5_000,
});

db.on('error', (err) => {
  console.error('Unexpected DB error:', err);
});

export async function checkDbConnection(): Promise<void> {
  const client = await db.connect();
  await client.query('SELECT 1');
  client.release();
}