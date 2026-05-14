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

/// Storage keys used by the Rewards Engine contract.
/// 
/// These keys map to different types of data related to reward tracking and distribution.
/// Most keys store global configuration; some are keyed per-user for per-address data.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Admin address (can update reward rates)
    Admin,
    /// Address of the token used as reward payout
    RewardToken,
    /// Address of the vault contract (used for share balance queries)
    VaultContract,
    /// Reward rate: tokens per ledger, scaled by [`REWARD_PRECISION`].
    /// Updated by admin via [`set_reward_rate`](RewardsContract::set_reward_rate).
    RewardRate,
    /// Total shares tracked (mirrors vault's total_shares).
    /// NOTE: This is updated manually and can drift. TODO: Use cross-contract query.
    TotalShares,
    /// Per-user: ledger number when they last interacted with rewards.
    /// Used to calculate elapsed time for reward accrual.
    UserLastLedger(Address),
    /// Per-user: accumulated but unclaimed reward balance.
    /// Includes all rewards accrued up to the last checkpoint.
    UserPendingRewards(Address),
    /// Per-user: snapshot of their share balance at last checkpoint.
    /// Stored to enable accurate reward calculation when shares change.
    UserShareSnapshot(Address),
    /// Total rewards distributed (for accounting and analytics).
    /// Incremented when users claim rewards.
    TotalDistributed,
}

/// Precision multiplier for reward rate calculations.
/// 
/// The reward rate is stored as `tokens_per_ledger * REWARD_PRECISION` to avoid 
/// integer truncation when working with fractional rates. For example, a rate of
/// 0.001 tokens per ledger would be stored as 1_000_000.
/// 
/// # Current Value
/// 
/// Set to `1e9` for balance between precision and overflow risk.
/// 
/// # Notes
/// 
/// For very small reward rates, this precision may be insufficient and lead to 
/// rounding errors. Future versions may increase this to `1e18` for better precision.
/// 
/// TODO: Evaluate whether 1e9 precision is sufficient or if we need 1e18
const REWARD_PRECISION: i128 = 1_000_000_000;

#[contract]
pub struct RewardsContract;

#[contractimpl]
impl RewardsContract {
    /// Initialize the rewards engine with configuration parameters.
    ///
    /// This function must be called exactly once after the contract is deployed.
    /// It sets up the initial reward configuration and vault link.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `admin` - Admin address with governance permissions. Can update the reward rate.
    /// * `reward_token` - Token contract address used for reward payouts.
    /// * `vault` - Associated vault contract address (queried for user share balances).
    /// * `reward_rate` - Initial rewards per ledger per unit of total shares,
    ///   scaled by [`REWARD_PRECISION`]. For example, to distribute 1 token per ledger
    ///   to 1000 total shares: `reward_rate = (1_000_000_000 / 1000)`.
    ///
    /// # Panics
    /// 
    /// * If the contract is already initialized (has admin set)
    ///
    /// # Example
    /// 
    /// ```ignore
    /// let reward_rate = 1_000_000; // 1e6 * 1e9 precision = very small rate
    /// rewards.initialize(&admin, &reward_token, &vault, reward_rate);
    /// ```
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

    /// Checkpoint a user's rewards and record their current share balance.
    ///
    /// This function must be called before any change to the user's vault share balance
    /// (e.g., before a deposit or withdrawal). It calculates and stores pending rewards
    /// accrued since the user's last checkpoint.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `user` - The user address to checkpoint
    ///
    /// # Implementation
    /// 
    /// This function:
    /// 1. Retrieves the user's last checkpoint ledger
    /// 2. Calculates elapsed ledgers since last checkpoint
    /// 3. Computes newly accrued rewards using [`compute_pending`](RewardsContract::compute_pending)
    /// 4. Updates the user's pending reward balance
    /// 5. Updates the user's last interaction ledger
    ///
    /// # Notes
    /// 
    /// If the user has never been checkpointed, this initializes their tracking data.
    /// The implementation is currently a TODO stub.
    ///
    /// TODO: Compute newly accrued rewards since last checkpoint
    /// TODO: Update user's pending reward balance
    /// TODO: Update last interaction ledger
    /// TODO: Integrate automatic checkpointing via vault cross-contract calls
    /// TODO: Handle edge case where total_shares is zero
    pub fn checkpoint(env: Env, user: Address) {
        // TODO: Get user's last checkpoint ledger
        // TODO: Get current ledger number
        // TODO: Calculate elapsed = current_ledger - last_ledger
        // TODO: Get user's share snapshot
        // TODO: Get total shares from vault
        // TODO: Get reward rate
        // TODO: Calculate pending = elapsed * rate * user_shares / total_shares / PRECISION
        // TODO: Update user's pending_rewards storage
        // TODO: Update user's last_ledger to current

        panic!("TODO: checkpoint() implementation needed");
    }

    /// Claim all pending rewards for the calling user.
    ///
    /// Transfers all accumulated reward tokens to the user and clears their pending balance.
    /// This function must be authorized by the user (checked via `require_auth()`).
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `user` - The user address claiming rewards (must authorize this call)
    ///
    /// # Returns
    /// 
    /// The amount of reward tokens transferred to the user.
    ///
    /// # Panics
    /// 
    /// * If the user does not authorize the call
    /// * If implementation is incomplete (TODO)
    ///
    /// # Implementation
    /// 
    /// This function:
    /// 1. Checkpoints the user to account for latest accrual
    /// 2. Retrieves the user's pending reward balance
    /// 3. Transfers reward tokens to the user via the reward token contract
    /// 4. Clears the user's pending balance from storage
    /// 5. Updates the total distributed counter
    /// 6. Emits a claim event
    ///
    /// # Notes
    /// 
    /// The implementation is currently a TODO stub.
    ///
    /// TODO: Implement reward token transfers
    /// TODO: Add a minimum claim threshold to avoid dust transfers
    /// TODO: Emit a richer event with reward rate snapshot
    pub fn claim_rewards(env: Env, user: Address) -> i128 {
        user.require_auth();

        // TODO: Call checkpoint(user) to calculate any new accrual
        // TODO: Get pending_rewards from storage
        // TODO: If pending > 0:
        //   - Clear pending balance
        //   - Update total_distributed
        //   - Transfer reward tokens to user
        //   - Emit claim event
        // TODO: Return claimed amount

        panic!("TODO: claim_rewards() implementation needed");
    }

    /// Query the pending reward balance for a user without claiming.
    ///
    /// This is a read-only view function that returns the total pending rewards
    /// for a user, including any rewards not yet checkpointed.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `user` - The user address to query
    ///
    /// # Returns
    /// 
    /// The total pending rewards (already-accrued + newly-computed amounts).
    /// Returns `0` if the user has never interacted with the rewards engine.
    ///
    /// # Notes
    /// 
    /// This function does not update storage; it only reads and computes values.
    /// Users should call [`claim_rewards`](RewardsContract::claim_rewards) to actually
    /// receive their rewards.
    pub fn pending_rewards(env: Env, user: Address) -> i128 {
        let stored_pending: i128 = env
            .storage()
            .instance()
            .get(&DataKey::UserPendingRewards(user.clone()))
            .unwrap_or(0);

        let newly_accrued = Self::compute_pending(&env, &user);

        stored_pending + newly_accrued
    }

    /// Update the total shares tracked by the rewards engine.
    ///
    /// This function allows the vault (or another authorized caller) to synchronize
    /// the total shares count with the vault's current state. This is necessary because
    /// reward calculations depend on knowing the total number of shares outstanding.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `caller` - The address making the call (must be the vault contract)
    /// * `total_shares` - The vault's current total shares outstanding
    ///
    /// # Panics
    /// 
    /// * If the caller is not the vault contract
    ///
    /// # Notes
    /// 
    /// This is a manual synchronization mechanism and can drift if the vault's total shares
    /// change without calling this function. TODO: Replace with an automatic cross-contract
    /// query to `vault.total_shares()` so synchronization happens automatically.
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
    /// Changes the global reward rate (tokens per ledger) used for all reward calculations.
    /// Only the admin can call this function.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `caller` - The address making the call (must be the admin)
    /// * `new_rate` - The new reward rate, scaled by [`REWARD_PRECISION`]
    ///
    /// # Panics
    /// 
    /// * If the caller is not the admin
    ///
    /// # Notes
    /// 
    /// The reward rate change takes effect immediately for future accruals.
    /// Users' already-accumulated pending rewards are not affected.
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

    /// Update a user's share snapshot for accurate reward calculation.
    ///
    /// This function should be called after a user's vault share balance changes
    /// (e.g., after a deposit or withdrawal). It checkpoints the user's current
    /// rewards and updates their share balance snapshot for future calculations.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `user` - The user whose share snapshot to update
    /// * `shares` - The user's new share balance
    ///
    /// # Implementation
    /// 
    /// This function:
    /// 1. Calls [`checkpoint`](RewardsContract::checkpoint) to finalize accrual up to now
    /// 2. Stores the new share balance as the user's snapshot
    ///
    /// # Notes
    /// 
    /// This is a clunky interface. Ideally, the vault would call this atomically
    /// via a cross-contract invocation whenever the user's shares change.
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

    /// Returns the total amount of rewards distributed to all users since genesis.
    ///
    /// This is a read-only counter used for analytics and accounting.
    /// It increments whenever users claim rewards.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    ///
    /// # Returns
    /// 
    /// The total amount of reward tokens distributed. Returns `0` if no rewards have been claimed.
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
    /// This is a private helper function used by [`checkpoint`](RewardsContract::checkpoint)
    /// and [`pending_rewards`](RewardsContract::pending_rewards).
    ///
    /// # Calculation (Naive)
    /// 
    /// ```text
    /// elapsed_ledgers = current_ledger - last_ledger
    /// user_pending = (elapsed * rate * user_shares / total_shares) / PRECISION
    /// ```
    ///
    /// # Arguments
    /// 
    /// * `env` - The contract environment (borrowed)
    /// * `user` - The user address to compute rewards for (borrowed)
    ///
    /// # Returns
    /// 
    /// The newly accrued rewards since last checkpoint. Returns `0` if no time has elapsed.
    ///
    /// # Known Issues
    /// 
    /// This implementation is intentionally simple and has known limitations:
    /// - Linear accrual doesn't account for compounding effects
    /// - Can lose precision on very small reward rates
    /// - Doesn't handle the case where `total_shares = 0` gracefully
    /// - Does not account for changing share balances within a period
    ///
    /// # Future Improvements
    /// 
    /// TODO: Implement a cumulative index approach (like Synthetix) for better precision
    /// TODO: Handle zero total_shares case without panicking
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
