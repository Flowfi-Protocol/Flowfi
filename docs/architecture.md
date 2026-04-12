# FlowFi Protocol — Architecture

## Overview

FlowFi is a modular DeFi protocol built on Stellar's Soroban smart contract platform. The protocol enables users to deposit Stellar-native assets, earn time-based rewards, and benefit from capital deployed into yield strategies.

The system is intentionally modular: each concern is handled by a dedicated contract. This design makes the system easier to audit, upgrade, and extend by contributors.

---

## System Diagram

```
                         ┌──────────────────┐
                         │   Access Control  │
                         │   (admin, strat)  │
                         └────────┬─────────┘
                                  │ governs
          ┌───────────────────────┼───────────────────────┐
          │                       │                       │
          ▼                       ▼                       ▼
  ┌───────────────┐      ┌────────────────┐    ┌─────────────────────┐
  │     Vault     │◄────►│ Rewards Engine │    │  Strategy Router    │
  │               │      │                │    │                     │
  │ - deposit()   │      │ - checkpoint() │    │ - add_strategy()    │
  │ - withdraw()  │      │ - claim()      │    │ - set_weight()      │
  │ - shares      │      │ - pending()    │    │ - allocate()        │
  └───────┬───────┘      └────────────────┘    └──────────┬──────────┘
          │                                               │
          │ pushes capital                    ┌───────────┤
          └──────────────────────────────────►│           │
                                             │ Strategy A │
                                             │ (mock/pass)│
                                             └───────────┘
                                        (more strategies: future)
```

---

## Contract Responsibilities

### Vault (`contracts/vault`)

The Vault is the primary user-facing contract. It:

- Accepts deposits of the underlying token
- Issues "shares" to depositors proportional to their contribution
- Burns shares on withdrawal and returns underlying assets
- Maintains the share/asset accounting invariant

The vault uses a simple share-price model where:

```
shares_to_mint = amount * total_shares / total_assets
assets_to_return = shares * total_assets / total_shares
```

If the vault is empty (bootstrap state), the ratio defaults to 1:1.

**Extension points:**
- `total_assets()` should query strategy allocations for accurate TVL
- Deposits/withdrawals should notify the Rewards Engine via cross-contract call

---

### Rewards Engine (`contracts/rewards`)

The Rewards Engine tracks and distributes yield to vault depositors. It:

- Accrues rewards linearly over time (per ledger)
- Distributes proportional to each user's share of total_shares
- Provides `pending_rewards(user)` for read-only reward queries
- Transfers reward tokens to users on `claim_rewards()`

**Current implementation:**
Reward accrual uses a simple formula:
```
pending = elapsed_ledgers * reward_rate * user_shares / total_shares / PRECISION
```

This is intentionally naive. A checkpoint-based accumulator (Synthetix-style) would be more robust and gas-efficient.

**Extension points:**
- Multi-token rewards (multiple reward streams)
- Compounding rewards (auto-reinvesting into the vault)
- Vesting schedules for claimed rewards

---

### Strategy Router (`contracts/strategy_router`)

The Strategy Router is the yield allocation layer. It:

- Maintains a registry of approved yield strategies
- Stores allocation weights (in basis points) per strategy
- Tracks assets deployed per strategy

**Current state:** The router stores weights and allocation data, but no actual cross-contract capital movement is implemented. This is a deliberate open contribution surface.

**Extension points:**
- Cross-contract `allocate()` that calls `strategy.deposit(amount)`
- Harvest function to collect and route yield back to the vault
- Strategy performance metrics (realized APY, historical TVL)

---

### Access Control (`contracts/access_control`)

Provides role-based permission primitives used by all other contracts:

- **Admin**: Full protocol control (upgrade strategies, change rates)
- **Strategist**: Can adjust strategy weights and trigger rebalances

**Extension points:**
- Guardian role for emergency pauses
- Multi-sig or DAO-controlled admin
- Timelocked role transitions

---

## Data Flow: Deposit

```
User calls vault.deposit(amount)
│
├── Vault transfers `amount` tokens from user to itself
├── Vault calculates shares: shares = amount * total_shares / total_assets
├── Vault increments user's share balance
├── Vault increments total_shares and total_assets
│
│  (TODO: Vault should then call:)
├── rewards_engine.update_user_shares(user, new_shares)
└── strategy_router.allocate(vault_address, mock_strategy, amount)
```

---

## Data Flow: Claim Rewards

```
User calls rewards.claim_rewards()
│
├── Rewards checkpoints the user:
│   │  elapsed = current_ledger - last_ledger
│   └─ pending += elapsed * rate * user_shares / total_shares / PRECISION
│
├── Pending balance is transferred to user (reward token)
├── Pending balance is cleared
└── TotalDistributed counter incremented
```

---

## Storage Model

All contracts use Soroban's instance storage (persistent across ledger TTL). Key patterns:

- Enum-based `DataKey` for type-safe storage access
- Per-user keys use `DataKey::FieldName(Address)` pattern
- No inter-contract storage sharing — each contract manages its own state

---

## Security Considerations

- All state mutations require `address.require_auth()`
- Checks-Effects-Interactions pattern is followed in vault (state updated before transfer)
- No reentrancy guards yet (Soroban's model makes this less critical, but worth auditing)
- Overflow uses `checked_mul` / `saturating_mul` to prevent panics

**Known open issues:**
- Reward rate changes mid-period don't checkpoint all users
- Strategy accounting can diverge if allocate() fails mid-way
- No pausing mechanism for emergency situations

---

## Upgrade Path

Soroban does not support proxy upgrades natively in the same way EVM does. The planned upgrade approach:

1. Deploy new contract version
2. Admin triggers migration via access-controlled migration entry point
3. State is migrated or users are prompted to re-interact

This is a significant open design problem. Contributions welcome.
