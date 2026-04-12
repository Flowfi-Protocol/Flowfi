# FlowFi — Smart Contract Reference

## Overview

This document provides a function-level reference for each FlowFi contract. For architecture context, see [architecture.md](./architecture.md).

All contracts are deployed to Stellar Testnet for development. Mainnet deployment is pending security review.

---

## Vault Contract

**Path:** `contracts/vault/src/lib.rs`

### `initialize(admin: Address, token: Address)`
Sets up the vault with an admin and the underlying token. Must be called once after deployment.

| Param | Type | Description |
|-------|------|-------------|
| admin | Address | Protocol admin |
| token | Address | Stellar token contract address |

---

### `deposit(from: Address, amount: i128) → i128`
Deposits `amount` of the underlying token and mints shares to `from`.

Returns the number of shares minted.

Requires `from` to authorize the call.

| Param | Type | Description |
|-------|------|-------------|
| from | Address | Depositing user |
| amount | i128 | Token amount (in token's smallest unit) |

**Events emitted:** `("deposit", from)` → `(amount, shares_minted)`

---

### `withdraw(from: Address, shares: i128) → i128`
Burns `shares` and returns the proportional underlying asset amount to `from`.

Returns the number of underlying tokens returned.

| Param | Type | Description |
|-------|------|-------------|
| from | Address | Withdrawing user |
| shares | i128 | Number of shares to burn |

**Events emitted:** `("withdraw", from)` → `(shares, assets_out)`

---

### `balance_of(user: Address) → i128`
Returns the share balance of a given user.

---

### `total_assets() → i128`
Returns total assets held in the vault. **Note:** Does not include assets deployed to strategies yet.

---

### `total_shares() → i128`
Returns total shares outstanding.

---

### `share_price() → i128`
Returns the current share price in 6-decimal fixed point (1_000_000 = 1.0).

---

## Rewards Engine Contract

**Path:** `contracts/rewards/src/lib.rs`

### `initialize(admin, reward_token, vault, reward_rate)`
Initializes the rewards engine.

| Param | Type | Description |
|-------|------|-------------|
| admin | Address | Admin address |
| reward_token | Address | Token to pay out as rewards |
| vault | Address | Associated vault address |
| reward_rate | i128 | Rewards per ledger, scaled by 1e9 |

---

### `checkpoint(user: Address)`
Calculates and stores newly accrued rewards for `user`. Should be called before any balance change.

---

### `claim_rewards(user: Address) → i128`
Claims all pending rewards for `user`. Transfers reward tokens and returns the amount claimed.

Requires `user` to authorize.

**Events emitted:** `("CLAIM", user)` → `amount_claimed`

---

### `pending_rewards(user: Address) → i128`
Read-only. Returns total pending rewards including unchecked accrual.

---

### `update_total_shares(caller: Address, total_shares: i128)`
Updates the total share count tracked by the rewards engine. Only callable by the vault.

---

### `set_reward_rate(caller: Address, new_rate: i128)`
Updates the reward rate. Admin only.

---

### `total_distributed() → i128`
Returns the total rewards paid out since genesis.

---

## Strategy Router Contract

**Path:** `contracts/strategy_router/src/lib.rs`

### `initialize(admin, strategist, vault)`
Sets up the router with role addresses.

---

### `add_strategy(caller, strategy, weight_bps)`
Registers a new strategy. Admin only. Weight is in basis points (100 bps = 1%).

---

### `deactivate_strategy(caller, strategy)`
Marks a strategy as inactive. Admin only. Does not currently pull funds.

---

### `set_weight(caller, strategy, new_weight_bps)`
Updates a strategy's allocation weight. Strategist only.

---

### `allocate(caller, strategy, amount)`
Records capital allocation to a strategy. Currently updates accounting only — no token movement.

---

### `strategy_info(strategy) → StrategyInfo`
Returns metadata for a registered strategy.

```rust
pub struct StrategyInfo {
    pub address: Address,
    pub weight_bps: u32,
    pub active: bool,
    pub allocated: i128,
}
```

---

### `strategy_list() → Vec<Address>`
Returns all registered strategy addresses.

---

### `total_allocated() → i128`
Returns sum of all strategy allocations.

---

## Access Control Contract

**Path:** `contracts/access_control/src/lib.rs`

### `initialize(admin: Address)`
Sets the initial admin.

### `admin() → Address`
Returns current admin.

### `strategist() → Option<Address>`
Returns current strategist, if assigned.

### `transfer_admin(new_admin: Address)`
Transfers admin role. Current admin must authorize.

### `set_strategist(strategist: Address)`
Assigns strategist role. Admin only.

### `is_admin(address: Address) → bool`
Checks if an address is admin.

### `is_strategist(address: Address) → bool`
Checks if an address is strategist.

### `assert_admin(caller: Address)`
Panics if caller is not admin. Used internally by other contracts.

### `assert_strategist(caller: Address)`
Panics if caller is not strategist.

---

## Error Handling

Currently, errors are expressed as panics with string messages. This is intentionally naive.

**Planned improvement:** Define a unified `FlowFiError` enum and use it across all contracts. This is a great intermediate-level contribution.

---

## Contract Addresses (Testnet)

> Addresses will be updated after testnet deployment. See `scripts/deploy.sh` for deployment steps.

| Contract | Address |
|----------|---------|
| Vault | TBD |
| Rewards | TBD |
| Strategy Router | TBD |
| Access Control | TBD |
