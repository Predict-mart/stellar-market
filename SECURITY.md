# Security Policy

## Reporting a Vulnerability

The StellarMarket team takes security seriously. If you discover a vulnerability, please follow responsible disclosure:

**Do NOT open a public GitHub issue for security vulnerabilities.**

### How to Report

Email: **security@stellarmarket.io**

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- Your contact information (for follow-up)

We will acknowledge your report within **48 hours** and provide a timeline for resolution within **7 days**.

---

## Bug Bounty

Once the platform launches on mainnet, a formal bug bounty program will be active. Rewards are based on severity:

| Severity | Reward Range |
|---|---|
| Critical (fund loss possible) | $10,000 – $50,000 |
| High (major protocol disruption) | $2,500 – $10,000 |
| Medium (partial data corruption) | $500 – $2,500 |
| Low (minor issues) | Up to $500 |

---

## Known Risk Areas

The following areas carry elevated risk and should be considered carefully:

### Oracle Manipulation
Oracle providers could submit false reports to resolve markets incorrectly. Mitigations:
- Multi-provider consensus (2/3 required)
- 48-hour dispute window
- Governance fallback for contested resolutions
- Provider staking and slashing (planned)

### CLOB Front-Running
Stellar's ~5s block times reduce front-running risk vs Ethereum, but it still exists. Mitigations:
- Commit-reveal schemes under consideration for v2
- On-chain sequencing is deterministic and auditable

### Smart Contract Exploits
Soroban contracts are immutable post-deployment. Mitigations:
- External audit by two independent firms before mainnet
- Formal verification of core matching logic (planned)
- Emergency governance pause mechanism

### Governance Attacks
A compromised council majority could alter protocol parameters. Mitigations:
- Time-locked execution of governance proposals (48h delay)
- Council size minimum (5 members)
- Community veto mechanism (planned)

### Price Manipulation
Thin markets are susceptible to price manipulation. Mitigations:
- Minimum liquidity requirements for market approval
- Circuit breakers for extreme price movements (planned)

---

## Secure Development Practices

- All smart contract changes require 2 maintainer approvals
- Contracts are deployed from a hardware-wallet-controlled deployer key
- No private keys are stored server-side
- All user transactions are signed client-side (Freighter wallet)
- Backend API does not require users to share keys

---

## Disclosure Timeline

| Stage | Timeframe |
|---|---|
| Report received | Day 0 |
| Acknowledgement | Within 48 hours |
| Initial assessment | Within 7 days |
| Patch development | Depends on severity |
| Critical: hotfix | ASAP (typically <7 days) |
| High: next release | Within 30 days |
| Public disclosure | 90 days after fix (or sooner with reporter's consent) |

---

## Hall of Fame

We publicly thank responsible disclosers (with their permission) in our Security Hall of Fame.
