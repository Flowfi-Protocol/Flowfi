//! FlowFi Strategy Router
//!
//! Routes vault capital to yield-generating strategies. The router maintains
//! a registry of approved strategies and tracks how much capital each holds.
//!
//! Currently, only one mock strategy is implemented (PassthroughStrategy).
//! The architecture is designed to support many strategies with configurable
//! allocation weights.
//!
//! Design goals:
//! - Vault delegates allocation decisions to the router
//! - Strategist role configures which strategies are active and their weights
//! - Admin can pause or remove strategies in emergencies
//!
//! Current state (intentionally incomplete):
//! - Only mock/passthrough strategy exists
//! - Allocation weights are stored but not enforced during rebalancing
//! - No actual on-chain rebalancing logic implemented
//!
//! TODO: Implement real rebalancing: allocate() and withdraw_from_strategy()
//! TODO: Add a strategy interface (trait-like) that all strategies must conform to
//! TODO: Implement harvest() to collect yield from strategies into the vault
//! TODO: Add strategy performance tracking (APY, TVL per strategy)
//! TODO: Add weight validation (sum of weights must equal 100%)
//! TODO: Consider supporting Soroban's AMM or Blend Protocol as strategies
//! TODO: Add emergency exit (pull all funds from all strategies)

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Vec,
};

/// A registered strategy with its allocation configuration
#[contracttype]
#[derive(Clone)]
pub struct StrategyInfo {
    /// The strategy contract address
    pub address: Address,
    /// Allocation weight (basis points, 0-10000)
    /// TODO: Enforce that weights across all strategies sum to 10000
    pub weight_bps: u32,
    /// Whether this strategy is currently active
    pub active: bool,
    /// Total assets currently allocated to this strategy
    /// TODO: Replace with live cross-contract query to strategy contract
    pub allocated: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Strategist,
    Vault,
    /// List of registered strategy addresses
    StrategyList,
    /// Per-strategy info, keyed by strategy address
    Strategy(Address),
    /// Total assets under management across all strategies
    TotalAllocated,
}

#[contract]
pub struct StrategyRouterContract;

#[contractimpl]
impl StrategyRouterContract {
    /// Initialize the strategy router.
    ///
    /// # Arguments
    /// - `admin`: Can add/remove strategies
    /// - `strategist`: Can adjust weights and trigger rebalances
    /// - `vault`: The associated vault contract (only vault can push/pull funds)
    pub fn initialize(env: Env, admin: Address, strategist: Address, vault: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Strategist, &strategist);
        env.storage().instance().set(&DataKey::Vault, &vault);
        env.storage()
            .instance()
            .set(&DataKey::StrategyList, &Vec::<Address>::new(&env));
        env.storage().instance().set(&DataKey::TotalAllocated, &0_i128);
    }

    /// Register a new strategy. Admin only.
    ///
    /// # Arguments
    /// - `strategy`: The strategy contract address
    /// - `weight_bps`: Initial allocation weight in basis points
    ///
    /// TODO: Verify caller is admin
    /// TODO: Validate weight_bps is within valid range (0-10000)
    /// TODO: Check strategy is not already registered
    /// TODO: Validate total weights don't exceed 10000 bps
    /// TODO: Call strategy.validate() to check interface compatibility
    /// TODO: Add strategy to registry with weight and allocated balance
    /// TODO: Emit ADD_STRATEGY event
    pub fn add_strategy(env: Env, caller: Address, strategy: Address, weight_bps: u32) {
        // TODO: Assert caller is admin
        caller.require_auth();

        // TODO: Validate weight_bps <= 10000
        // TODO: Check not already registered
        // TODO: Create StrategyInfo struct
        // TODO: Store in DataKey::Strategy(strategy)
        // TODO: Append to StrategyList
        // TODO: Emit event

        panic!("TODO: add_strategy() implementation needed");
    }

    /// Deactivate a strategy. Admin only.
    /// Does not remove — keeps historical data.
    ///
    /// TODO: Before deactivating, trigger withdrawal of all funds from strategy
    /// TODO: Redistribute weight to remaining active strategies
    pub fn deactivate_strategy(env: Env, caller: Address, strategy: Address) {
        Self::assert_admin(&env, &caller);
        caller.require_auth();

        let mut info: StrategyInfo = env
            .storage()
            .instance()
            .get(&DataKey::Strategy(strategy.clone()))
            .expect("strategy not found");

        // TODO: If info.allocated > 0, pull funds before deactivating
        if info.allocated > 0 {
            // For now, just warn via event. Should block in production.
            env.events().publish(
                (symbol_short!("WARN"), symbol_short!("ALLOC")),
                (strategy.clone(), info.allocated),
            );
        }

        info.active = false;
        env.storage()
            .instance()
            .set(&DataKey::Strategy(strategy.clone()), &info);
    }

    /// Update allocation weights. Strategist only.
    ///
    /// TODO: Verify caller is strategist
    /// TODO: Verify strategy is registered
    /// TODO: Validate new weight is within valid range (0-10000)
    /// TODO: Check new total weights don't exceed 10000 bps across all strategies
    /// TODO: Update strategy weight
    /// TODO: Emit SET_WEIGHT event
    /// TODO: Automatically trigger rebalance after weight update
    pub fn set_weight(env: Env, caller: Address, strategy: Address, new_weight_bps: u32) {
        // TODO: Assert caller is strategist
        caller.require_auth();

        // TODO: Get strategy info (panic if not found)
        // TODO: Validate new_weight_bps <= 10000
        // TODO: Update weight in storage
        // TODO: Emit event

        panic!("TODO: set_weight() implementation needed");
    }

    /// Allocate `amount` tokens to a given strategy.
    /// Only the vault can call this.
    ///
    /// TODO: Verify caller is the vault contract
    /// TODO: Verify strategy is registered and active
    /// TODO: Validate allocation doesn't exceed strategy weight limits
    /// TODO: Implement cross-contract token transfer to strategy contract
    /// TODO: Update strategy's allocated balance
    /// TODO: Update total_allocated counter
    /// TODO: Emit ALLOCATE event
    pub fn allocate(env: Env, caller: Address, strategy: Address, amount: i128) {
        // TODO: Assert caller is vault
        // TODO: Get strategy info (panic if not found)
        // TODO: Check strategy is active
        // TODO: Call strategy.deposit(amount) cross-contract to actually move funds
        // TODO: Update strategy.allocated += amount
        // TODO: Update total_allocated += amount
        // TODO: Emit event

        panic!("TODO: allocate() implementation needed");
    }

    /// Returns info for a given strategy.
    pub fn strategy_info(env: Env, strategy: Address) -> StrategyInfo {
        env.storage()
            .instance()
            .get(&DataKey::Strategy(strategy))
            .expect("strategy not found")
    }

    /// Returns the list of all registered strategy addresses.
    pub fn strategy_list(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::StrategyList)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Returns total assets allocated across all strategies.
    pub fn total_allocated(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalAllocated)
            .unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn assert_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        if &admin != caller {
            panic!("only admin");
        }
    }

    fn assert_strategist(env: &Env, caller: &Address) {
        let strategist: Address = env
            .storage()
            .instance()
            .get(&DataKey::Strategist)
            .expect("not initialized");
        if &strategist != caller {
            panic!("only strategist");
        }
    }

    fn assert_vault(env: &Env, caller: &Address) {
        let vault: Address = env
            .storage()
            .instance()
            .get(&DataKey::Vault)
            .expect("not initialized");
        if &vault != caller {
            panic!("only vault");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_add_and_query_strategy() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, StrategyRouterContract);
        let client = StrategyRouterContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let strategist = Address::generate(&env);
        let vault = Address::generate(&env);
        let strategy = Address::generate(&env);

        client.initialize(&admin, &strategist, &vault);
        client.add_strategy(&admin, &strategy, &5000); // 50%

        let info = client.strategy_info(&strategy);
        assert_eq!(info.weight_bps, 5000);
        assert!(info.active);
        assert_eq!(info.allocated, 0);
    }

    // TODO: Test deactivate_strategy
    // TODO: Test set_weight with unauthorized caller
    // TODO: Test allocate from vault
    // TODO: Test strategy_list returns all strategies
}
