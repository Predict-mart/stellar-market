# StellarMarket — System Architecture

## Table of Contents

1. [Overview](#overview)
2. [High-Level Architecture](#high-level-architecture)
3. [Smart Contract Layer](#smart-contract-layer)
4. [CLOB Trading Engine](#clob-trading-engine)
5. [Oracle System](#oracle-system)
6. [Backend Indexer & API](#backend-indexer--api)
7. [Frontend](#frontend)
8. [Data Flow](#data-flow)
9. [Market Lifecycle](#market-lifecycle)
10. [Security Model](#security-model)

---

## Overview

StellarMarket is a decentralized prediction markets platform built on Stellar's Soroban smart contract platform. It uses a **Central Limit Order Book (CLOB)** model for price discovery and trade execution — giving traders precise control over fill prices, unlike AMM-based alternatives.

Markets follow a **Hybrid creation model**: any user may propose a market, but a multisig maintainer council must approve it before it goes live. This balances openness with quality control and spam prevention.

---

## High-Level Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                         Users / Traders                              │
└────────────────────────────┬─────────────────────────────────────────┘
                             │ HTTPS / WebSocket
┌────────────────────────────▼─────────────────────────────────────────┐
│                      Frontend (Next.js)                              │
│   Market Discovery | Order Placement | Portfolio | Governance UI     │
└────────────────────────────┬─────────────────────────────────────────┘
                             │ REST + WebSocket
┌────────────────────────────▼─────────────────────────────────────────┐
│                    Backend API Server (Node.js)                      │
│         Validates & Submits XDR Transactions to Stellar RPC          │
└──────┬────────────────────┬────────────────────────┬─────────────────┘
       │                    │                        │
┌──────▼──────┐   ┌─────────▼────────┐   ┌──────────▼──────────┐
│  Indexer    │   │  Oracle Relay    │   │  Settlement Worker  │
│  (Horizon   │   │  (Feeds external │   │  (Monitors resolved │
│  events)    │   │   data on-chain) │   │   markets, triggers │
│             │   │                  │   │   payouts)          │
└──────┬──────┘   └─────────┬────────┘   └──────────┬──────────┘
       │                    │                        │
┌──────▼────────────────────▼────────────────────────▼──────────┐
│                    PostgreSQL + Redis                          │
│   Markets | Orders | Trades | Positions | Oracle Reports      │
└───────────────────────────────┬───────────────────────────────┘
                                │
┌───────────────────────────────▼───────────────────────────────┐
│                  Stellar Network (Soroban)                     │
│                                                               │
│  ┌─────────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │MarketFactory│  │  Market  │  │  Oracle  │  │Settlement│  │
│  │  Contract   │  │ Contract │  │ Contract │  │ Contract │  │
│  │             │  │  (CLOB)  │  │          │  │          │  │
│  └─────────────┘  └──────────┘  └──────────┘  └──────────┘  │
└───────────────────────────────────────────────────────────────┘
```

---

## Smart Contract Layer

All business logic lives in Soroban contracts written in Rust. Contracts are immutable once deployed; upgrades require a governance vote.

### MarketFactory Contract

**Purpose:** Registry of all markets. Manages the proposal → approval → live lifecycle.

**Storage:**
```
market_count: u64
markets: Map<u64, MarketMetadata>
pending_proposals: Map<u64, MarketProposal>
maintainers: Vec<Address>      // multisig council
approval_threshold: u32        // e.g. 3-of-5
```

**Key Methods:**
- `propose_market(question, outcomes, resolution_date, oracle_id)` — any user
- `approve_market(proposal_id)` — maintainer only
- `reject_market(proposal_id, reason)` — maintainer only
- `get_market(id)` — read
- `list_markets(status, page)` — read

**Events emitted:**
- `MarketProposed { proposal_id, proposer, question }`
- `MarketApproved { market_id, proposal_id }`
- `MarketRejected { proposal_id, reason }`

---

### Market Contract (CLOB Engine)

**Purpose:** Hosts a Central Limit Order Book for a single market. Manages YES/NO outcome token trading.

**Storage:**
```
market_id: u64
question: String
outcomes: Vec<String>           // ["YES", "NO"] or multi-outcome
status: MarketStatus            // Pending | Active | Resolved | Settled
resolution_date: u64            // Unix timestamp
resolved_outcome: Option<u32>
order_book: Map<OutcomeId, OrderBook>
positions: Map<Address, Position>
```

**Order Book Structure:**
```
OrderBook {
  bids: BTreeMap<Price, Vec<Order>>,   // sorted descending
  asks: BTreeMap<Price, Vec<Order>>,   // sorted ascending
}

Order {
  order_id: u128,
  trader: Address,
  outcome_id: u32,
  side: Side,       // Buy | Sell
  price: u64,       // in basis points, e.g. 6500 = $0.65
  quantity: u64,    // number of shares
  timestamp: u64,
}
```

**Key Methods:**
- `place_order(outcome_id, side, price, quantity)` — price-time priority matching
- `cancel_order(order_id)`
- `get_order_book(outcome_id)` — returns bids/asks snapshot
- `get_position(trader)` — returns holdings
- `get_best_bid(outcome_id)` / `get_best_ask(outcome_id)`

**Matching Logic:**
Orders are matched at price-time priority. When a new order arrives:
1. Check opposite side of the book for crossing orders
2. Fill at the resting order's price (maker gets their price)
3. Partial fills are supported; remainder stays in book
4. Emit `TradeExecuted` for each fill

**Events:**
- `OrderPlaced { order_id, trader, outcome_id, side, price, quantity }`
- `OrderCancelled { order_id }`
- `TradeExecuted { buy_order_id, sell_order_id, outcome_id, price, quantity }`
- `MarketResolved { outcome_id }`

---

### OutcomeToken Contract

**Purpose:** Fungible token representing a share in a specific outcome. Winners redeem 1 share = 1 USDC at settlement.

**Storage:**
```
market_id: u64
outcome_id: u32
total_supply: u128
balances: Map<Address, u128>
```

**Key Methods:**
- `mint(to, amount)` — called by Market contract on buy
- `burn(from, amount)` — called by Settlement on redemption
- `transfer(from, to, amount)` — standard token transfer
- `balance_of(address)` — read

---

### Oracle Contract

**Purpose:** Receives resolution reports from approved oracle providers, aggregates them, and finalizes market outcomes.

**Storage:**
```
oracle_providers: Vec<Address>
reports: Map<MarketId, Vec<OracleReport>>
finalized: Map<MarketId, u32>     // winning outcome_id
dispute_window: u64               // seconds after first report before finalization
```

**OracleReport:**
```
OracleReport {
  provider: Address,
  market_id: u64,
  outcome_id: u32,
  timestamp: u64,
  confidence: u8,     // 0–100
}
```

**Key Methods:**
- `submit_report(market_id, outcome_id, confidence)`
- `finalize_resolution(market_id)` — callable after dispute_window passes
- `dispute_resolution(market_id, rationale)` — triggers governance vote
- `get_resolution(market_id)` — read

**Resolution Logic:**
- Minimum 2/3 of registered oracle providers must agree
- If consensus is not reached within the dispute window, governance decides
- Reports are weighted by provider reputation score

---

### Settlement Contract

**Purpose:** Distributes winnings to holders of the resolved outcome token.

**Storage:**
```
settled_markets: Map<u64, bool>
total_pool: Map<u64, u128>      // total USDC in market
fee_rate: u32                   // basis points, e.g. 100 = 1%
fee_recipient: Address
```

**Key Methods:**
- `settle_market(market_id)` — callable after oracle finalizes
- `claim_winnings(market_id)` — winner redeems outcome tokens for USDC
- `get_payout_amount(market_id, address)` — read

**Payout Formula:**
```
payout = (outcome_token_balance / total_winning_supply) * total_pool * (1 - fee_rate)
```

---

### Governance Contract

**Purpose:** Manages maintainer council, parameter changes, and dispute resolution votes.

**Storage:**
```
council: Vec<Address>
proposals: Map<u64, GovernanceProposal>
votes: Map<(ProposalId, Address), Vote>
quorum: u32
```

**Key Methods:**
- `create_proposal(type, payload)`
- `vote(proposal_id, in_favor)`
- `execute_proposal(proposal_id)` — after vote passes
- `add_council_member(address)`
- `remove_council_member(address)`

---

## CLOB Trading Engine

The CLOB is the heart of StellarMarket's price discovery. Unlike AMMs, a CLOB allows:

- **Limit orders** — trade only at your specified price or better
- **No slippage** for resting orders — makers get exactly their price
- **Transparent order book** — full depth visible on-chain and via API
- **Price-time priority** — fair queue ordering

### Why CLOB over AMM for Prediction Markets?

| | CLOB (StellarMarket) | AMM |
|---|---|---|
| Price precision | Exact | Slippage-dependent |
| Liquidity efficiency | Capital efficient | Requires LP deposits |
| Market making | Professional MMs | Passive LPs |
| Transparency | Full book depth | Formula-based only |
| Late-market efficiency | Excellent (wide spreads = thin book) | Poor (always prices at curve) |

### Order Types (v1)

- **Limit orders** — place at specific price, rest in book
- **Cancel** — remove resting order

### Order Types (planned v2)

- Market orders
- Fill-or-kill (FOK)
- Immediate-or-cancel (IOC)
- Stop orders

---

## Oracle System

### Oracle Providers (Tiered)

**Tier 1 — Decentralized Data Feeds**
- Band Protocol / DIA (on-chain price feeds)
- Custom Stellar oracle contracts

**Tier 2 — API-based Oracles**
- Approved human reporters with staked reputation
- Off-chain API aggregators (Chainlink-compatible adapters)

**Tier 3 — Governance Fallback**
- If oracle consensus fails, council votes to resolve
- Emergency backstop only

### Resolution Flow

```
Market closes
     │
Oracle providers submit reports (24h window)
     │
Consensus check: ≥ 2/3 agree?
     ├── YES → Finalize after dispute window (48h)
     └── NO  → Open dispute → Governance vote (7 days)
```

---

## Backend Indexer & API

The indexer listens to Stellar Horizon events and maintains a queryable database of all on-chain activity.

### Services

| Service | Purpose |
|---|---|
| `EventListener` | Polls Horizon for contract events |
| `OrderBookService` | Maintains real-time CLOB state |
| `MarketService` | Market metadata + status |
| `OracleService` | Aggregates oracle reports |
| `SettlementService` | Monitors & triggers payouts |

### API Endpoints

```
GET  /markets                   # List all active markets
GET  /markets/:id               # Market detail + order book
GET  /markets/:id/orderbook     # Live order book depth
GET  /markets/:id/trades        # Trade history
POST /markets/:id/orders        # Submit signed XDR transaction
GET  /positions/:address        # User portfolio
WS   /markets/:id/stream        # Real-time order book stream
```

---

## Market Lifecycle

```
[User proposes market] → Pending
        │
[Maintainer council approves]
        │
[Market goes live] → Active
        │
[Orders flow, CLOB matching]
        │
[Resolution date reached] → Closed
        │
[Oracle providers report]
        │
[Consensus + dispute window] → Resolved
        │
[Settlement contract distributes winnings] → Settled
```

---

## Security Model

- All user actions are signed client-side; the backend never holds private keys
- Contract storage uses scoped data with address authorization
- Oracle reports require registered provider signature
- Governance actions require multisig threshold
- All contracts are open-source and audited before mainnet

See [SECURITY.md](./SECURITY.md) for vulnerability reporting.
