.PHONY: help dev down clean build test lint fmt contracts deploy-testnet fund-testnet

# ── Default ───────────────────────────────────────────────────────────────────
help:
	@echo ""
	@echo "  StellarMarket — Developer Commands"
	@echo ""
	@echo "  make dev              Start full local dev environment"
	@echo "  make down             Stop and remove containers"
	@echo "  make clean            Remove containers, volumes, and images"
	@echo ""
	@echo "  make build            Build all Docker images"
	@echo "  make test             Run all tests (contracts + backend + frontend)"
	@echo "  make lint             Lint all workspaces"
	@echo "  make fmt              Format all workspaces"
	@echo ""
	@echo "  make contracts        Build and test Soroban contracts"
	@echo "  make deploy-testnet   Deploy contracts to Stellar Testnet"
	@echo "  make fund-testnet     Fund a testnet account via Friendbot"
	@echo ""

# ── Local Dev ─────────────────────────────────────────────────────────────────
dev:
	docker compose up --build

down:
	docker compose down

clean:
	docker compose down -v --rmi local

build:
	docker compose build

# ── Testing ───────────────────────────────────────────────────────────────────
test: test-contracts test-backend test-frontend

test-contracts:
	@echo "▶ Testing contracts..."
	cd contracts && cargo test

test-backend:
	@echo "▶ Testing backend..."
	cd backend && npm test

test-frontend:
	@echo "▶ Testing frontend..."
	cd frontend && npm test

test-integration:
	@echo "▶ Starting test services..."
	docker compose -f docker-compose.test.yml up -d
	@echo "▶ Running integration tests..."
	cd backend && DATABASE_URL=postgresql://stellarmarket:stellarmarket@localhost:5433/stellarmarket_test \
	              REDIS_URL=redis://localhost:6380 \
	              npm run test:integration
	docker compose -f docker-compose.test.yml down

# ── Linting & Formatting ──────────────────────────────────────────────────────
lint:
	cd contracts && cargo clippy -- -D warnings
	cd backend && npm run lint
	cd frontend && npm run lint

fmt:
	cd contracts && cargo fmt
	cd backend && npm run format
	cd frontend && npm run format

# ── Contracts ─────────────────────────────────────────────────────────────────
contracts:
	@echo "▶ Building contracts..."
	cd contracts && cargo build --target wasm32-unknown-unknown --release
	@echo "▶ Running contract tests..."
	cd contracts && cargo test

# ── Deployment ────────────────────────────────────────────────────────────────
deploy-testnet:
	@echo "▶ Deploying to Stellar Testnet..."
	./scripts/deploy-testnet.sh

fund-testnet:
	@if [ -z "$(ADDR)" ]; then echo "Usage: make fund-testnet ADDR=<stellar-address>"; exit 1; fi
	./scripts/fund-testnet-account.sh $(ADDR)

# ── Logs ──────────────────────────────────────────────────────────────────────
logs:
	docker compose logs -f

logs-backend:
	docker compose logs -f backend

logs-frontend:
	docker compose logs -f frontend