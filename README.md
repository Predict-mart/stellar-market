# 🌟 StellarMarket

> **Predict. Trade. Settle. Trustlessly.**

A decentralized prediction markets platform built on the Stellar blockchain using Soroban smart contracts — combining the transparency of on-chain settlement with the efficiency of a Central Limit Order Book (CLOB).

---

## Vision

StellarMarket is an open-source, community-governed prediction markets protocol that enables anyone to trade on the outcomes of real-world events — elections, sports, finance, technology, and global news — with near-zero fees, instant finality, and full on-chain settlement.

We believe prediction markets are one of the most powerful tools for aggregating distributed knowledge. StellarMarket makes them accessible to the world.

---

## Why Stellar?

| Feature | Benefit |
|---|---|
| ~3-5s finality | Real-time order matching |
| Sub-cent fees | Micro-trades are viable |
| Native DEX primitives | Deep liquidity synergy |
| Soroban smart contracts | Expressive, auditable logic |
| Global reach | Accessible without traditional finance |

---

## Quick Start

### Prerequisites

- Rust (1.75+) with `wasm32-unknown-unknown` target
- Node.js 18+
- Stellar CLI (`cargo install stellar-cli`)
- Docker & Docker Compose

### 1. Clone the repo

```bash
git clone https://github.com/Predict-mart/stellar-market
cd stellar-market
```

### 2. Install dependencies

```bash
# Install Rust WASM target
rustup target add wasm32-unknown-unknown

# Install frontend deps
cd frontend && npm install

# Install backend deps
cd ../backend && npm install
```

### 3. Start local Stellar network

```bash
docker compose up stellar-node
```

### 4. Deploy contracts (testnet)

```bash
cd scripts
./deploy-testnet.sh
```

### 5. Run the frontend

```bash
cd frontend && npm run dev
```

Open `http://localhost:3000`

---

## Tech Stack

| Layer | Technology |
|---|---|
| Smart Contracts | Rust / Soroban |
| Blockchain | Stellar (Testnet / Mainnet) |
| Backend Indexer | Node.js + TypeScript |
| API | REST + WebSocket |
| Frontend | Next.js + TypeScript |
| Database | PostgreSQL + Redis |
| Infrastructure | Docker, GitHub Actions |

---

## Architecture Overview

```
┌─────────────────────────────────────────────┐
│                  Frontend (Next.js)          │
└──────────────────┬──────────────────────────┘
                   │ REST / WebSocket
┌──────────────────▼──────────────────────────┐
│            Backend API + Indexer             │
│         (Node.js / TypeScript)               │
└──────────────────┬──────────────────────────┘
                   │ Stellar Horizon RPC
┌──────────────────▼──────────────────────────┐
│           Soroban Smart Contracts            │
│  MarketFactory | CLOB | Oracle | Settlement  │
└──────────────────┬──────────────────────────┘
                   │
        ┌──────────▼──────────┐
        │   Stellar Network   │
        └─────────────────────┘
```

---

## Repository Structure

```
/stellarmarket
├── contracts/          # Soroban smart contracts (Rust)
├── backend/            # Indexer + REST API (TypeScript)
├── frontend/           # Web application (Next.js)
├── docs/               # Architecture, guides, specs
├── scripts/            # Deployment + utility scripts
├── tests/              # Integration + E2E tests
└── deployments/        # Contract addresses + configs
```

---

## Contributing

We welcome contributors of all experience levels. See [CONTRIBUTING.md](./CONTRIBUTING.md) to get started.

Browse our [open issues](https://github.com/Predict-mart/stellar-market/issues) — many are labeled `good first issue`.

---

## License

Apache 2.0 — see [LICENSE](./LICENSE)

---

