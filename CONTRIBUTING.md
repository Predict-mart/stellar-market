# Contributing to StellarMarket

Thank you for your interest in contributing! StellarMarket is an open-source protocol and we welcome contributions from developers of all experience levels.

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Setup](#development-setup)
4. [How to Contribute](#how-to-contribute)
5. [Pull Request Process](#pull-request-process)
6. [Coding Standards](#coding-standards)
7. [Testing Requirements](#testing-requirements)
8. [Issue Labels](#issue-labels)

---

## Code of Conduct

We are committed to a welcoming, inclusive community. All contributors must follow our [Code of Conduct](./CODE_OF_CONDUCT.md). Be respectful, assume good faith, and help each other grow.

---

## Getting Started

1. Browse [open issues](https://github.com/stellarmarket/stellarmarket/issues)
2. Issues labeled `good first issue` are great for newcomers
3. Comment on an issue to claim it before starting work
4. Fork the repo, create a branch, and open a PR

---

## Development Setup

### Requirements

- Rust 1.75+ with `wasm32-unknown-unknown` target
- Node.js 18+
- Docker + Docker Compose
- Stellar CLI: `cargo install stellar-cli`

### Local Environment

```bash
# Clone your fork
git clone https://github.com/<your-username>/stellarmarket.git
cd stellarmarket

# Add upstream remote
git remote add upstream https://github.com/stellarmarket/stellarmarket.git

# Start local Stellar node (Futurenet)
docker compose up -d stellar-node

# Build contracts
cd contracts
cargo build --target wasm32-unknown-unknown --release

# Run contract tests
cargo test

# Start backend
cd ../backend
npm install
npm run dev

# Start frontend
cd ../frontend
npm install
npm run dev
```

### Environment Variables

Copy `.env.example` to `.env` in `backend/` and `frontend/`:

```bash
cp backend/.env.example backend/.env
cp frontend/.env.example frontend/.env
```

---

## How to Contribute

### Reporting Bugs

File a GitHub issue with:
- Clear title describing the bug
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version, Node version)
- Relevant logs or screenshots

### Suggesting Features

Open a GitHub Discussion under "Ideas" before filing an issue for large features. For small improvements, an issue is fine.

### Picking Up Issues

1. Check that the issue is not already assigned
2. Comment "I'd like to work on this" to claim it
3. Issues unclaimed for 14 days are re-opened to others
4. Ask questions in the issue thread before starting

---

## Pull Request Process

### Branch Naming

```
feat/short-description       # New features
fix/short-description        # Bug fixes
docs/short-description       # Documentation
test/short-description       # Tests
chore/short-description      # Maintenance
```

### Before Opening a PR

- [ ] All existing tests pass (`cargo test`, `npm test`)
- [ ] New code is covered by tests
- [ ] Linting passes (`cargo clippy`, `npm run lint`)
- [ ] Formatting applied (`cargo fmt`, `npm run format`)
- [ ] Documentation updated if API changed
- [ ] CHANGELOG.md updated (for notable changes)

### PR Description Template

```markdown
## Summary
What does this PR do?

## Related Issue
Closes #<issue-number>

## Changes
- List of specific changes

## Testing
How was this tested?

## Screenshots (if UI change)
```

### Review Process

- At least 1 approval required from a core maintainer
- For smart contract changes: 2 approvals required
- CI must pass before merge
- Squash merge preferred for feature branches

---

## Coding Standards

### Rust (Contracts)

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo clippy` and fix all warnings
- Use `cargo fmt` for formatting
- Document all public functions with `///` doc comments
- Avoid `unwrap()` — use proper error handling
- Emit events for all state-changing operations

```rust
/// Places a limit order on the order book.
///
/// # Arguments
/// * `outcome_id` - The outcome being traded
/// * `side` - Buy or Sell
/// * `price` - Price in basis points (0–10000)
/// * `quantity` - Number of shares
///
/// # Errors
/// Returns `Error::InvalidPrice` if price is out of range.
pub fn place_order(
    env: Env,
    trader: Address,
    outcome_id: u32,
    side: Side,
    price: u64,
    quantity: u64,
) -> Result<u128, Error> { ... }
```

### TypeScript (Backend/Frontend)

- Strict TypeScript (`"strict": true` in tsconfig)
- ESLint + Prettier for formatting
- No `any` types without explicit justification
- Async/await over raw promises
- Error handling with typed errors, not string messages

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(contracts): add partial fill support to CLOB
fix(indexer): handle missing ledger sequence gracefully
docs(architecture): update oracle resolution diagram
test(settlement): add edge case for zero-supply outcome
```

---

## Testing Requirements

### Contracts

- Unit tests for every public function
- Edge case coverage: zero balances, price boundaries, full book scenarios
- Integration tests using `soroban-sdk`'s test environment

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_place_order_matches_crossing_order() {
        let env = Env::default();
        // ...
    }
}
```

### Backend

- Unit tests with Jest
- Integration tests against local Stellar node
- API contract tests (request/response schema)

### Frontend

- Component tests with React Testing Library
- E2E tests with Playwright for critical flows

---

## Issue Labels

| Label | Meaning |
|---|---|
| `good first issue` | Suitable for newcomers, well-scoped |
| `help wanted` | Extra attention needed |
| `contracts` | Soroban smart contract work |
| `backend` | Indexer / API work |
| `frontend` | UI / UX work |
| `testing` | Test coverage |
| `docs` | Documentation |
| `infrastructure` | DevOps / CI / Docker |
| `security` | Security-related |
| `oracle` | Oracle integration |
| `governance` | Governance mechanism |
| `bug` | Something broken |
| `enhancement` | Improvement to existing feature |
| `discussion` | Needs architectural discussion |

---

## Questions?

- Open a GitHub Discussion
- Join our [Discord](https://discord.gg/stellarmarket)
- Tag `@core-team` in your issue for urgent questions

We're happy to help you make your first contribution!
