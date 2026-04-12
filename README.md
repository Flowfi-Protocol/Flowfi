# FlowFi Protocol

> **Modular DeFi yield and staking infrastructure for the Stellar ecosystem, built on Soroban.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Built on Soroban](https://img.shields.io/badge/Built%20on-Soroban-blueviolet)](https://soroban.stellar.org)
[![Contributions Welcome](https://img.shields.io/badge/Contributions-Welcome-brightgreen)](CONTRIBUTING.md)
[![Network: Testnet](https://img.shields.io/badge/Network-Testnet-orange)](https://soroban-testnet.stellar.org)

---

## What is FlowFi?

FlowFi is an open-source, modular DeFi protocol for Stellar. It lets users deposit Stellar-native assets into a shared vault, earn time-based rewards, and eventually benefit from capital allocated to on-chain yield strategies — all powered by **Soroban smart contracts** written in Rust.

The protocol is intentionally designed as a **contribution-first project**: the codebase has clear extension points, well-labeled TODOs, and a layered issue tracker that welcomes contributors from first-timers to protocol researchers.

---

## Why FlowFi on Stellar?

Stellar is one of the most widely used payment networks in the world, processing billions of dollars in cross-border transactions annually. But its **DeFi ecosystem is nascent**. The launch of Soroban — Stellar's programmable smart contract layer — opens the door for permissionless financial protocols.

FlowFi aims to be the **foundational yield layer** of Stellar DeFi:

- **Stellar's asset diversity**: XLM, CBDCs, stablecoins, and tokenized assets can all flow through FlowFi
- **Low fees**: Soroban transactions cost a fraction of EVM equivalents, making frequent interactions (rewards claims, rebalancing) economically viable
- **Financial inclusion focus**: Stellar's core mission aligns with DeFi that works for underbanked populations, not just crypto-native whales
- **Composability**: FlowFi's modular contracts are designed to integrate with protocols like Blend (lending) and Soroswap (AMM)

---

## Architecture Overview

FlowFi is composed of four smart contract modules, each with a single responsibility:

```
┌──────────────────────────────────────────────────────────┐
│                     FlowFi Protocol                       │
│                                                           │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────┐ │
│  │    Vault    │◄──►│   Rewards    │    │  Strategy   │ │
│  │             │    │   Engine     │    │   Router    │ │
│  │ deposits    │    │              │    │             │ │
│  │ withdrawals │    │ time-based   │    │ allocation  │ │
│  │ shares      │    │ accrual      │    │ weights     │ │
│  └─────────────┘    └──────────────┘    └──────┬──────┘ │
│                                                │         │
│  ┌──────────────────┐              ┌───────────▼──────┐  │
│  │  Access Control  │              │    Strategies    │  │
│  │  admin/strat     │              │  (passthrough,   │  │
│  │  roles           │              │   Blend, etc.)   │  │
│  └──────────────────┘              └──────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

| Module | Responsibility |
|--------|---------------|
| **Vault** | Accepts deposits, issues shares, handles withdrawals |
| **Rewards Engine** | Accrues time-based rewards proportional to share ownership |
| **Strategy Router** | Routes capital to yield-generating strategies |
| **Access Control** | Manages admin and strategist roles across the protocol |

For a deeper dive, see [docs/architecture.md](docs/architecture.md).

---

## Repository Structure

```
flowfi/
├── Cargo.toml                  # Workspace manifest
├── contracts/
│   ├── vault/                  # Core deposit/withdraw/share logic
│   ├── rewards/                # Time-based reward accrual and claiming
│   ├── strategy_router/        # Capital allocation to strategies
│   └── access_control/         # Role-based permission system
├── sdk/                        # TypeScript SDK
│   └── src/
│       ├── client.ts           # Main FlowFiClient class
│       ├── types.ts            # TypeScript type definitions
│       └── index.ts            # Public exports
├── tests/                      # Integration tests (cross-contract)
├── docs/
│   ├── architecture.md         # Protocol architecture deep dive
│   └── contracts.md            # Contract function reference
├── examples/                   # Usage examples
├── scripts/
│   └── deploy.sh               # Deployment automation
├── CONTRIBUTING.md
└── GITHUB_ISSUES.md            # Issue templates for contributors
```

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) with `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/install-cli) (`stellar`)
- [Node.js](https://nodejs.org/) >= 18 (for SDK)

```bash
# Install Rust wasm target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install stellar-cli --features opt

# Verify installations
stellar --version
rustc --version
```

### Clone and Build

```bash
git clone https://github.com/your-org/flowfi.git
cd flowfi

# Build all contracts
cargo build --target wasm32-unknown-unknown --release --workspace

# Run all contract tests
cargo test --workspace
```

### Set Up the SDK

```bash
cd sdk
npm install
npm run build
```

---

## Deploying to Testnet

```bash
# 1. Configure your environment
cp .env.example .env
# Edit .env with your testnet secret key and RPC URL

# 2. Fund your deployer account
stellar keys generate deployer
stellar keys public-key deployer  # Get the public key
# Fund via: https://laboratory.stellar.org/account-creator?network=test

# 3. Deploy
chmod +x scripts/deploy.sh
./scripts/deploy.sh
```

After deployment, contract addresses are saved to `.deployed.json`.

---

## Interacting via CLI

### Deposit

```bash
stellar contract invoke \
  --id <VAULT_CONTRACT_ID> \
  --source <YOUR_KEYPAIR_ALIAS> \
  --network testnet \
  -- deposit \
  --from <YOUR_PUBLIC_KEY> \
  --amount 1000000
```

### Check Pending Rewards

```bash
stellar contract invoke \
  --id <REWARDS_CONTRACT_ID> \
  --source <YOUR_KEYPAIR_ALIAS> \
  --network testnet \
  -- pending_rewards \
  --user <YOUR_PUBLIC_KEY>
```

### Claim Rewards

```bash
stellar contract invoke \
  --id <REWARDS_CONTRACT_ID> \
  --source <YOUR_KEYPAIR_ALIAS> \
  --network testnet \
  -- claim_rewards \
  --user <YOUR_PUBLIC_KEY>
```

### Withdraw

```bash
stellar contract invoke \
  --id <VAULT_CONTRACT_ID> \
  --source <YOUR_KEYPAIR_ALIAS> \
  --network testnet \
  -- withdraw \
  --from <YOUR_PUBLIC_KEY> \
  --shares 500000
```

---

## SDK Usage

```typescript
import { FlowFiClient, Networks } from '@flowfi/sdk';
import { Keypair } from '@stellar/stellar-sdk';

// Load deployed addresses from .deployed.json
import deployed from './.deployed.json';

const client = new FlowFiClient({
  rpcUrl: 'https://soroban-testnet.stellar.org',
  networkPassphrase: Networks.TESTNET,
  contracts: deployed.contracts,
});

const signer = Keypair.fromSecret(process.env.SECRET_KEY!);

// Deposit 1 XLM (in stroops)
const depositResult = await client.deposit(signer, 10_000_000n);
console.log(`Deposited. Shares minted: ${depositResult.sharesMinted}`);

// Check your position
const position = await client.getUserPosition(signer.publicKey());
console.log(`Shares: ${position.shares}`);
console.log(`Pending rewards: ${position.pendingRewards}`);
console.log(`Estimated value: ${position.estimatedValue}`);

// Claim rewards
const claimResult = await client.claimRewards(signer);
console.log(`Claimed: ${claimResult.rewardsClaimed} tokens`);

// Withdraw half your shares
const withdrawResult = await client.withdraw(signer, position.shares / 2n);
console.log(`Withdrew: ${withdrawResult.assetsReturned} tokens`);
```

---

## Roadmap

FlowFi is under active development. The following features are planned and represent excellent contribution opportunities.

### 🟡 In Progress
- Cross-contract reward checkpointing on deposit/withdraw
- Passthrough strategy contract (first real strategy)
- `getStrategies()` SDK implementation

### 📋 Planned
- [ ] Checkpoint-based reward accumulator (Synthetix-style)
- [ ] Multi-token reward support
- [ ] `harvest()` function in Strategy Router
- [ ] Protocol fee mechanism (performance fee on yield)
- [ ] Emergency pause / guardian role
- [ ] Withdrawal queue for illiquid strategies
- [ ] Share price manipulation protection (virtual offset)
- [ ] Proper `FlowFiError` enum (replace string panics)

### 🔬 Research
- [ ] Blend Protocol integration (lending strategy)
- [ ] Soroswap AMM liquidity strategy
- [ ] On-chain governance module (token-weighted voting)
- [ ] Automated rebalancing based on weight targets
- [ ] Protocol-owned liquidity mechanisms

### 🏗️ Infrastructure
- [ ] Full integration test suite (cross-contract)
- [ ] Deployment to Stellar Mainnet
- [ ] Subgraph/indexer for historical data
- [ ] Frontend dashboard (React + wagmi equivalent for Stellar)

---

## Why Contribute?

FlowFi is designed from the ground up as a **contributor-first protocol**. Here's why it's worth your time:

**🌱 Learn Soroban by building something real**
Most Soroban tutorials stop at "hello world." FlowFi gives you a real, multi-contract protocol with meaningful complexity — vault math, reward accrual, cross-contract calls, role-based access. Every part of the codebase has documentation explaining *why* it works the way it does.

**🔍 Clear contribution surface**
Every TODO comment in the code is a potential issue. Every GitHub Issue has a clearly defined scope, expected outcome, and difficulty label. You won't wonder where to start.

**📐 Protocol design challenges**
If you're interested in the deeper questions — how should reward rate changes be handled fairly? What's the right withdrawal queue design? How should governance interact with protocol parameters? — those conversations are explicitly invited.

**🤝 Maintainer support**
Issues are triaged weekly. PRs get reviewed with detailed feedback. If you're blocked, open a discussion and we'll help.

**📜 Portfolio-worthy work**
A merged PR on a live Soroban protocol is a meaningful addition to a Web3 developer portfolio, especially given how few production Soroban contracts exist today.

See [CONTRIBUTING.md](CONTRIBUTING.md) to get started.

---

## Security

FlowFi is in active development and **has not been audited**. Do not use on mainnet with real funds until an audit is completed.

If you discover a security vulnerability, please do **not** open a public GitHub issue. Instead, email [security@flowfi.xyz] with a description of the issue and steps to reproduce.

See our [Security Policy](SECURITY.md) for more details.

---

## License

MIT — see [LICENSE](LICENSE).

---

## Acknowledgements

FlowFi draws architectural inspiration from Yearn Finance (vault shares), Synthetix (staking rewards), and ERC-4626 (share accounting). Built with love for the Stellar ecosystem.
