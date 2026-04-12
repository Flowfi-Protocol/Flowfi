//! FlowFi Rewards Engine
//!
//! Handles time-based reward accrual and distribution to vault depositors.
//! Rewards are proportional to share ownership at the time of accrual.
//!
//! Current Implementation (naive / intentionally simple):
//! - A fixed reward rate (tokens per ledger) is set by the admin
//! - Users accumulate rewards based on their share balance × elapsed ledgers
//! - No compounding; rewards are stored separately from principal
//!
//! Known Limitations (good first contributions):
//! - Reward rate is global, not per-strategy
//! - No support for multiple reward tokens
//! - Reward calculation can drift if user balances change mid-period
//! - No checkpoint system — users must claim before balance changes
//!
//! TODO: Implement a checkpoint-based accumulator (like Synthetix staking rewards)
//! TODO: Support streaming multiple reward tokens simultaneously
//! TODO: Add reward vesting / lockup schedules
//! TODO: Integrate with Vault share changes to auto-checkpoint on deposit/withdraw
//! TODO: Add a governance-controlled reward rate adjustment mechanism

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Admin address
    Admin,
    /// Address of the token used as reward
    RewardToken,
    /// Address of the vault contract (for share balance queries)
    VaultContract,
    /// Reward rate: tokens per ledger, scaled by REWARD_PRECISION
    RewardRate,
    /// Total shares tracked (mirrors vault's total_shares)
    /// NOTE: This is updated manually and can drift. TODO: Use cross-contract query.
    TotalShares,
    /// Per-user: ledger number when they last interacted with rewards
    UserLastLedger(Address),
    /// Per-user: accumulated but unclaimed reward balance
    UserPendingRewards(Address),
    /// Per-user: snapshot of their share balance at last checkpoint
    UserShareSnapshot(Address),
    /// Total rewards distributed (for accounting / analytics)
    TotalDistributed,
}

/// Precision multiplier for reward rate calculations.
/// Reward rate is stored as tokens_per_ledger * REWARD_PRECISION.
/// This avoids integer truncation for small rates.
///
/// TODO: Evaluate whether 1e9 precision is sufficient or if we need 1e18
const REWARD_PRECISION: i128 = 1_000_000_000;

#[contract]
pub struct RewardsContract;

#[contractimpl]
impl RewardsContract {
    /// Initialize the rewards engine.
    ///
    /// # Arguments
    /// - `admin`: Admin address for governance
    /// - `reward_token`: Token paid out as rewards
    /// - `vault`: Address of the associated vault (used for share balance queries)
    /// - `reward_rate`: Rewards per ledger per total share, scaled by REWARD_PRECISION
    pub fn initialize(
        env: Env,
        admin: Address,
        reward_token: Address,
        vault: Address,
        reward_rate: i128,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::RewardToken, &reward_token);
        env.storage().instance().set(&DataKey::VaultContract, &vault);
        env.storage().instance().set(&DataKey::RewardRate, &reward_rate);
        env.storage().instance().set(&DataKey::TotalShares, &0_i128);
        env.storage().instance().set(&DataKey::TotalDistributed, &0_i128);
    }

    /// Checkpoint a user's rewards.
    /// Must be called before any share balance change (deposit or withdraw).
    ///
    /// This calculates pending rewards since the user's last interaction
    /// and adds them to their pending balance.
    ///
    /// TODO: This should be called automatically by the vault via cross-contract call.
    ///       Currently requires manual triggering, which is error-prone.
    pub fn checkpoint(env: Env, user: Address) {
        let pending = Self::compute_pending(&env, &user);

        let existing_pending: i128 = env
            .storage()
            .instance()
            .get(&DataKey::UserPendingRewards(user.clone()))
            .unwrap_or(0);

        env.storage().instance().set(
            &DataKey::UserPendingRewards(user.clone()),
            &(existing_pending + pending),
        );

        // Update last ledger to now
        let current_ledger = env.ledger().sequence();
        env.storage()
            .instance()
            .set(&DataKey::UserLastLedger(user.clone()), &current_ledger);
    }

    /// Claim all pending rewards for the calling user.
    ///
    /// Transfers accumulated reward tokens to the user.
    ///
    /// TODO: Add a minimum claim threshold to avoid dust transfers
    /// TODO: Emit a richer event with reward rate at time of claim
    pub fn claim_rewards(env: Env, user: Address) -> i128 {
        user.require_auth();

        // Checkpoint first to include any new accrual
        Self::checkpoint(env.clone(), user.clone());

        let pending: i128 = env
            .storage()
            .instance()
            .get(&DataKey::UserPendingRewards(user.clone()))
            .unwrap_or(0);

        if pending == 0 {
            return 0;
        }

        // Clear pending rewards
        env.storage()
            .instance()
            .set(&DataKey::UserPendingRewards(user.clone()), &0_i128);

        // Update total distributed counter
        let distributed: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalDistributed)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalDistributed, &(distributed + pending));

        // Transfer reward tokens to user
        let reward_token: Address = env
            .storage()
            .instance()
            .get(&DataKey::RewardToken)
            .expect("not initialized");

        let token_client = token::Client::new(&env, &reward_token);
        token_client.transfer(&env.current_contract_address(), &user, &pending);

        env.events().publish(
            (symbol_short!("CLAIM"), user.clone()),
            pending,
        );

        pending
    }

    /// Returns the pending reward balance for a user (without claiming).
    ///
    /// This is a view function and includes any unchecked accrual since last interaction.
    pub fn pending_rewards(env: Env, user: Address) -> i128 {
        let stored_pending: i128 = env
            .storage()
            .instance()
            .get(&DataKey::UserPendingRewards(user.clone()))
            .unwrap_or(0);

        let newly_accrued = Self::compute_pending(&env, &user);

        stored_pending + newly_accrued
    }

    /// Update the total shares tracked by this contract.
    /// Must be called by the vault when deposits/withdrawals occur.
    ///
    /// TODO: Replace this with a cross-contract query to vault.total_shares()
    ///       so this stays in sync automatically without manual updates.
    pub fn update_total_shares(env: Env, caller: Address, total_shares: i128) {
        // Only vault contract can call this
        let vault: Address = env
            .storage()
            .instance()
            .get(&DataKey::VaultContract)
            .expect("not initialized");

        if caller != vault {
            panic!("only vault can update shares");
        }

        env.storage()
            .instance()
            .set(&DataKey::TotalShares, &total_shares);
    }

    /// Update the reward rate. Admin only.
    ///
    /// TODO: Require a governance vote before rate changes
    /// TODO: Emit an event with old vs new rate for transparency
    pub fn set_reward_rate(env: Env, caller: Address, new_rate: i128) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");

        if caller != admin {
            panic!("only admin");
        }
        caller.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::RewardRate, &new_rate);
    }

    /// Update a user's share snapshot. Should be called after deposit/withdraw.
    ///
    /// TODO: This is a clunky interface — ideally the vault calls checkpoint
    ///       and updates the snapshot atomically via cross-contract invocation.
    pub fn update_user_shares(env: Env, user: Address, shares: i128) {
        // First checkpoint the user to prevent reward manipulation
        Self::checkpoint(env.clone(), user.clone());

        env.storage()
            .instance()
            .set(&DataKey::UserShareSnapshot(user), &shares);
    }

    /// Returns total rewards distributed since genesis.
    pub fn total_distributed(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalDistributed)
            .unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Compute newly accrued rewards for a user since their last checkpoint.
    ///
    /// Formula (naive):
    ///   pending = (current_ledger - last_ledger) * reward_rate * user_shares / total_shares / PRECISION
    ///
    /// This is intentionally simple. Known issues:
    /// - Linear accrual doesn't account for compounding
    /// - Can lose precision on very small reward rates
    /// - Doesn't handle the case where total_shares = 0 gracefully
    ///
    /// TODO: Implement a cumulative index approach for better precision
    fn compute_pending(env: &Env, user: &Address) -> i128 {
        let current_ledger = env.ledger().sequence();

        let last_ledger: u32 = env
            .storage()
            .instance()
            .get(&DataKey::UserLastLedger(user.clone()))
            .unwrap_or(current_ledger);

        if current_ledger <= last_ledger {
            return 0;
        }

        let elapsed = (current_ledger - last_ledger) as i128;

        let user_shares: i128 = env
            .storage()
            .instance()
            .get(&DataKey::UserShareSnapshot(user.clone()))
            .unwrap_or(0);

        if user_shares == 0 {
            return 0;
        }

        let total_shares: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalShares)
            .unwrap_or(0);

        if total_shares == 0 {
            return 0;
        }

        let reward_rate: i128 = env
            .storage()
            .instance()
            .get(&DataKey::RewardRate)
            .unwrap_or(0);

        // rewards = elapsed * rate * user_shares / total_shares / PRECISION
        elapsed
            .saturating_mul(reward_rate)
            .saturating_mul(user_shares)
            / total_shares
            / REWARD_PRECISION
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_pending_rewards_zero_initially() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, RewardsContract);
        let client = RewardsContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let reward_token = Address::generate(&env);
        let vault = Address::generate(&env);
        let user = Address::generate(&env);

        client.initialize(&admin, &reward_token, &vault, &1_000_000_000);

        assert_eq!(client.pending_rewards(&user), 0);
    }

    // TODO: Test that rewards accrue correctly over multiple ledgers
    // TODO: Test claim_rewards transfers tokens and clears balance
    // TODO: Test set_reward_rate with unauthorized caller (expect panic)
    // TODO: Test checkpoint before and after share changes
    // TODO: Test rewards with multiple users (proportional distribution)
}
