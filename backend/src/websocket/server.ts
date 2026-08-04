import { WebSocketServer, WebSocket } from 'ws';
import { EventEmitter } from 'events';

// Global event bus — indexer publishes here, WS server subscribes
export const marketEvents = new EventEmitter();
marketEvents.setMaxListeners(0);

interface SubscribeMessage {
  type: 'subscribe' | 'unsubscribe';
  market_id: string;
}

interface Client {
  ws: WebSocket;
  subscriptions: Set<string>;
}

const clients = new Set<Client>();

function broadcast(marketId: string, data: unknown): void {
  const payload = JSON.stringify(data);
  for (const client of clients) {
    if (client.subscriptions.has(marketId) && client.ws.readyState === WebSocket.OPEN) {
      client.ws.send(payload);
    }
  }
}

// Forward indexer events to subscribed WebSocket clients
marketEvents.on('orderbook_update', (marketId: string, update: unknown) => {
  broadcast(marketId, { type: 'orderbook_update', market_id: marketId, ...update as object });
});

marketEvents.on('trade', (marketId: string, trade: unknown) => {
  broadcast(marketId, { type: 'trade', market_id: marketId, ...trade as object });
});

marketEvents.on('market_resolved', (marketId: string, outcome: unknown) => {
  broadcast(marketId, { type: 'market_resolved', market_id: marketId, winning_outcome: outcome });
});

export function createWebSocketServer(port: number): WebSocketServer {
  const wss = new WebSocketServer({ port });

  wss.on('connection', (ws: WebSocket) => {
    const client: Client = { ws, subscriptions: new Set() };
    clients.add(client);

    // Heartbeat
    const pingInterval = setInterval(() => {
      if (ws.readyState === WebSocket.OPEN) ws.ping();
    }, 30_000);

    ws.on('message', (raw: Buffer) => {
      try {
        const msg = JSON.parse(raw.toString()) as SubscribeMessage;

        if (msg.type === 'subscribe' && msg.market_id) {
          client.subscriptions.add(String(msg.market_id));
          ws.send(JSON.stringify({ type: 'subscribed', market_id: msg.market_id }));
        } else if (msg.type === 'unsubscribe' && msg.market_id) {
          client.subscriptions.delete(String(msg.market_id));
          ws.send(JSON.stringify({ type: 'unsubscribed', market_id: msg.market_id }));
        } else {
          ws.send(JSON.stringify({ type: 'error', message: 'Unknown message type' }));
        }
      } catch {
        ws.send(JSON.stringify({ type: 'error', message: 'Invalid JSON' }));
      }
    });

    ws.on('close', () => {
      clients.delete(client);
      clearInterval(pingInterval);
    });

    ws.on('error', (err) => {
      console.error('WebSocket client error:', err.message);
    });

    // Welcome message
    ws.send(JSON.stringify({ type: 'connected', message: 'StellarMarket WebSocket' }));
  });

  return wss;
}