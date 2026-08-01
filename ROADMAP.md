# StellarMarket — Development Roadmap

This roadmap describes the planned development phases. Timelines are estimates and may shift based on community velocity and audits.

---

## Phase 0 — Foundation ✅ (Month 0–1)

Bootstrapping the project infrastructure.

- [x] Project identity and documentation
- [x] Monorepo structure
- [x] Local development environment (Docker)
- [x] CI/CD pipeline (GitHub Actions)
- [x] Soroban contract scaffolding
- [x] Backend project setup (Node.js + TypeScript)
- [x] Frontend project setup (Next.js)
- [x] Contributor guidelines and issue templates

---

## Phase 1 — Core Smart Contracts (Month 1–3)

Building the foundational Soroban contracts.

**Deliverables:**
- `MarketFactory` contract with proposal + approval flow
- `MarketContract` with CLOB engine (binary markets)
- `OutcomeToken` contract (fungible, mintable, burnable)
- Full unit test suite for all contracts
- Contract deployment scripts (testnet)
- Basic contract interaction CLI

**Milestones:**
- [ ] MarketFactory: propose/approve/reject lifecycle
- [ ] CLOB: limit order placement + price-time priority matching
- [ ] CLOB: partial fills + cancellation
- [ ] OutcomeToken: mint on buy, burn on redemption
- [ ] All contracts: >90% test coverage

---

## Phase 2 — Oracle & Settlement (Month 3–5)

Connecting real-world data to on-chain resolution.

**Deliverables:**
- `OracleContract` with multi-provider report aggregation
- `SettlementContract` with automated payout distribution
- Oracle provider onboarding framework
- Dispute window mechanism

**Milestones:**
- [ ] Oracle: multi-provider report submission
- [ ] Oracle: 2/3 consensus + finalization
- [ ] Settlement: winner payout calculation
- [ ] Settlement: fee collection
- [ ] Dispute: challenge flow → governance escalation
- [ ] End-to-end market lifecycle test (create → trade → resolve → settle)

---

## Phase 3 — Backend Indexer & API (Month 4–6)

Off-chain infrastructure for performant queries.

**Deliverables:**
- Horizon event indexer (TypeScript)
- PostgreSQL schema for markets, orders, trades, positions
- REST API with full market data endpoints
- WebSocket server for real-time order book streams
- Redis caching layer

**Milestones:**
- [ ] Event listener for all contract events
- [ ] Order book reconstruction from on-chain state
- [ ] REST API: markets, orders, trades, positions
- [ ] WebSocket: live order book depth stream
- [ ] API authentication (Stellar keypair signing)
- [ ] Rate limiting and abuse protection

---

## Phase 4 — Frontend Interface (Month 5–8)

Building the trader-facing web application.

**Deliverables:**
- Market discovery and browsing
- Market detail with live order book visualization
- Order placement interface (limit orders)
- Portfolio / positions dashboard
- Market proposal submission UI
- Wallet integration (Freighter, WalletConnect)

**Milestones:**
- [ ] Market listing page with search + filter
- [ ] Order book depth chart (bids/asks)
- [ ] Trade execution flow (sign + submit)
- [ ] Position and P&L dashboard
- [ ] Market proposal form
- [ ] Freighter wallet integration

---

## Phase 5 — Governance (Month 7–9)

Decentralizing protocol control.

**Deliverables:**
- `GovernanceContract` for council voting
- Governance UI (proposals, voting)
- Council member management
- Parameter change proposals (fees, dispute windows)

**Milestones:**
- [ ] Governance contract: proposal + voting lifecycle
- [ ] Council multisig approval flows
- [ ] Governance UI: browse proposals, vote
- [ ] Emergency pause mechanism
- [ ] Upgrade pathway for contracts

---

## Phase 6 — Security & Audit (Month 8–10)

Hardening before mainnet.

**Deliverables:**
- Internal security review
- External smart contract audit (2 independent firms)
- Penetration testing of backend API
- Bug bounty program launch

**Milestones:**
- [ ] Internal security review complete
- [ ] Audit firm #1 engaged
- [ ] Audit firm #2 engaged
- [ ] All critical/high findings remediated
- [ ] Bug bounty live on Immunefi or equivalent

---

## Phase 7 — Testnet Beta (Month 9–11)

Public beta on Stellar Testnet.

**Deliverables:**
- Full platform on Stellar Testnet
- Onboarding documentation
- Community testing campaign
- Performance benchmarking
- UI/UX refinement based on feedback

**Milestones:**
- [ ] Testnet beta announcement
- [ ] 100 community testers onboarded
- [ ] 10 sample markets live
- [ ] Latency targets met (<500ms order submission)
- [ ] Known bugs resolved

---

## Phase 8 — Mainnet Launch (Month 12)

Production deployment on Stellar Mainnet.

**Deliverables:**
- Mainnet contract deployment
- Production infrastructure (monitoring, alerting)
- Oracle provider agreements
- Initial market set (elections, sports, finance)
- Press and community launch

**Milestones:**
- [ ] Contracts deployed to mainnet
- [ ] ≥3 oracle providers live
- [ ] ≥5 markets open at launch
- [ ] Status page live
- [ ] Community announcement

---

## Beyond v1 — Future Roadmap

- **Multi-outcome markets** (beyond binary YES/NO)
- **Market order type** support in CLOB
- **Mobile app** (React Native)
- **Permissionless oracle submissions** (staked)
- **Cross-chain bridge** for USDC liquidity
- **DAO governance token** and token-weighted voting
- **Liquidity incentive programs**
- **Categorical market templates** (sports, elections)
- **API v2** with GraphQL

---

## How to Influence the Roadmap

- Open a GitHub Discussion under "Roadmap" to propose priorities
- Vote on existing discussions with 👍
- Join the monthly community call (Discord)
- Submit a governance proposal (once governance is live)
