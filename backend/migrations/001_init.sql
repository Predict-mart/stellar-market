-- ============================================================
-- StellarMarket — Initial Database Schema
-- Migration: 001_init.sql
-- ============================================================

-- ── Markets ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS markets (
    id              BIGSERIAL PRIMARY KEY,
    market_id       BIGINT UNIQUE NOT NULL,           -- on-chain ID
    question        TEXT NOT NULL,
    outcomes        JSONB NOT NULL DEFAULT '[]',       -- ["YES", "NO"]
    status          TEXT NOT NULL DEFAULT 'pending'    -- pending|active|closed|resolved|settled|rejected
                    CHECK (status IN ('pending','active','closed','resolved','settled','rejected')),
    category        TEXT NOT NULL DEFAULT 'other'
                    CHECK (category IN ('elections','sports','finance','tech','global','other')),
    resolution_date BIGINT NOT NULL,                  -- Unix timestamp
    oracle_id       TEXT,                             -- oracle contract address
    proposer        TEXT NOT NULL,                    -- Stellar address
    winning_outcome INTEGER,                          -- set on resolution
    total_pool_usdc NUMERIC(20,7) DEFAULT 0,
    volume_24h_usdc NUMERIC(20,7) DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    approved_at     TIMESTAMPTZ,
    resolved_at     TIMESTAMPTZ,
    settled_at      TIMESTAMPTZ
);

CREATE INDEX idx_markets_status    ON markets (status);
CREATE INDEX idx_markets_category  ON markets (category);
CREATE INDEX idx_markets_created   ON markets (created_at DESC);
CREATE INDEX idx_markets_res_date  ON markets (resolution_date);

-- ── Orders ────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS orders (
    id              BIGSERIAL PRIMARY KEY,
    order_id        NUMERIC(39,0) UNIQUE NOT NULL,    -- u128 on-chain ID
    market_id       BIGINT NOT NULL REFERENCES markets(market_id),
    outcome_id      INTEGER NOT NULL,
    trader_address  TEXT NOT NULL,
    side            TEXT NOT NULL CHECK (side IN ('buy','sell')),
    price_bps       INTEGER NOT NULL CHECK (price_bps BETWEEN 1 AND 9999),
    quantity        BIGINT NOT NULL,
    filled_quantity BIGINT NOT NULL DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'open'
                    CHECK (status IN ('open','filled','cancelled')),
    ledger_seq      BIGINT,
    tx_hash         TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_orders_market     ON orders (market_id, status);
CREATE INDEX idx_orders_trader     ON orders (trader_address, status);
CREATE INDEX idx_orders_price      ON orders (market_id, outcome_id, side, price_bps);
CREATE INDEX idx_orders_created    ON orders (created_at DESC);

-- ── Trades ────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS trades (
    id              BIGSERIAL PRIMARY KEY,
    market_id       BIGINT NOT NULL REFERENCES markets(market_id),
    outcome_id      INTEGER NOT NULL,
    buy_order_id    NUMERIC(39,0) NOT NULL,
    sell_order_id   NUMERIC(39,0) NOT NULL,
    price_bps       INTEGER NOT NULL,
    quantity        BIGINT NOT NULL,
    ledger_seq      BIGINT,
    tx_hash         TEXT,
    executed_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_trades_market     ON trades (market_id, outcome_id);
CREATE INDEX idx_trades_executed   ON trades (executed_at DESC);

-- ── Positions (outcome token balances) ───────────────────────────────────────
CREATE TABLE IF NOT EXISTS positions (
    id              BIGSERIAL PRIMARY KEY,
    trader_address  TEXT NOT NULL,
    market_id       BIGINT NOT NULL REFERENCES markets(market_id),
    outcome_id      INTEGER NOT NULL,
    token_balance   NUMERIC(39,0) NOT NULL DEFAULT 0,
    avg_buy_price   INTEGER,                          -- basis points
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (trader_address, market_id, outcome_id)
);

CREATE INDEX idx_positions_trader  ON positions (trader_address);
CREATE INDEX idx_positions_market  ON positions (market_id);

-- ── Oracle Reports ────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS oracle_reports (
    id              BIGSERIAL PRIMARY KEY,
    market_id       BIGINT NOT NULL REFERENCES markets(market_id),
    outcome_id      INTEGER NOT NULL,
    provider_address TEXT NOT NULL,
    confidence      SMALLINT NOT NULL CHECK (confidence BETWEEN 0 AND 100),
    ledger_seq      BIGINT,
    submitted_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (market_id, provider_address)
);

CREATE INDEX idx_oracle_market     ON oracle_reports (market_id);

-- ── Raw Contract Events ───────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS events (
    id              BIGSERIAL PRIMARY KEY,
    contract_address TEXT NOT NULL,
    event_type      TEXT NOT NULL,
    payload         JSONB NOT NULL DEFAULT '{}',
    ledger_seq      BIGINT NOT NULL,
    tx_hash         TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_events_contract   ON events (contract_address);
CREATE INDEX idx_events_type       ON events (event_type);
CREATE INDEX idx_events_ledger     ON events (ledger_seq DESC);

-- ── Indexer Cursor ────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS indexer_state (
    key             TEXT PRIMARY KEY,
    value           TEXT NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed initial cursor state
INSERT INTO indexer_state (key, value)
VALUES ('last_cursor', '0')
ON CONFLICT (key) DO NOTHING;