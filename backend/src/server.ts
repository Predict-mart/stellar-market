import { createApp } from './app';
import { config } from './config';
import { checkDbConnection } from './db/client';
import { createWebSocketServer } from './websocket/server';
import { startIndexer } from './indexer/listener';

async function main(): Promise<void> {
  console.log(`🌟 Starting StellarMarket backend (${config.NODE_ENV})...`);

  // ── Database connectivity check ──────────────────────────────
  try {
    await checkDbConnection();
    console.log('✅ Database connected');
  } catch (err) {
    console.error('❌ Database connection failed:', err);
    process.exit(1);
  }

  // ── Start HTTP server ────────────────────────────────────────
  const app = createApp();
  const httpServer = app.listen(config.PORT, () => {
    console.log(`✅ API server listening on http://localhost:${config.PORT}`);
  });

  // ── Start WebSocket server ────────────────────────────────────
  createWebSocketServer(config.WS_PORT);
  console.log(`✅ WebSocket server listening on ws://localhost:${config.WS_PORT}`);

  // ── Start Horizon event indexer ───────────────────────────────
  startIndexer();
  console.log(`✅ Indexer started (polling ${config.HORIZON_URL} every ${config.POLL_INTERVAL_MS}ms)`);

  // ── Graceful shutdown ─────────────────────────────────────────
  const shutdown = () => {
    console.log('\n⏳ Shutting down gracefully...');
    httpServer.close(() => {
      console.log('✅ HTTP server closed');
      process.exit(0);
    });
  };

  process.on('SIGTERM', shutdown);
  process.on('SIGINT', shutdown);
}

main().catch((err) => {
  console.error('Fatal startup error:', err);
  process.exit(1);
});