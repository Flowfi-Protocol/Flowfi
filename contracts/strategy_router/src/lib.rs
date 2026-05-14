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

/// Configuration and state for a registered yield strategy.
/// 
/// Each strategy is tracked with its address, allocation weight, active status,
/// and the amount of capital currently allocated to it.
/// 
/// # Fields
/// 
/// * `address` - The strategy contract address. Must implement the strategy interface.
/// * `weight_bps` - Allocation weight in basis points (0-10000).
///   Represents the percentage of vault capital intended for this strategy.
/// * `active` - Whether this strategy is currently active and eligible for allocations.
/// * `allocated` - Current amount of underlying assets allocated to this strategy.
#[contracttype]
#[derive(Clone)]
pub struct StrategyInfo {
    /// The strategy contract address.
    /// This address should implement the standard strategy interface
    /// (deposit, withdraw, harvest, etc.).
    pub address: Address,
    /// Allocation weight (basis points, 0-10000).
    /// 100 bps = 1%. Total weights across all strategies should ideally sum to 10000.
    /// TODO: Enforce that weights across all strategies sum to 10000
    pub weight_bps: u32,
    /// Whether this strategy is currently active.
    /// Inactive strategies cannot receive new allocations but may still hold capital.
    pub active: bool,
    /// Total assets currently allocated to this strategy.
    /// Updated when capital is allocated or withdrawn.
    /// TODO: Replace with live cross-contract query to strategy contract
    pub allocated: i128,
}

/// Storage keys used by the Strategy Router contract.
/// 
/// Keys store configuration (admin, strategist, vault), a registry of strategies,
/// and per-strategy information.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Admin address (can add/remove strategies)
    Admin,
    /// Strategist address (can adjust weights and trigger rebalances)
    Strategist,
    /// Associated vault contract address (can allocate/withdraw capital)
    Vault,
    /// List of registered strategy addresses.
    /// Stored as a `Vec<Address>` for enumeration of all strategies.
    StrategyList,
    /// Per-strategy info, keyed by strategy address.
    /// Stores a [`StrategyInfo`] struct with weight, active status, and allocation.
    Strategy(Address),
    /// Total assets under management across all strategies.
    /// Sum of all strategy allocations. Updated during allocate/withdraw.
    TotalAllocated,
}

#[contract]
pub struct StrategyRouterContract;

#[contractimpl]
impl StrategyRouterContract {
    /// Initialize the strategy router with administrative roles.
    ///
    /// This function must be called exactly once after contract deployment.
    /// It sets up the admin and strategist roles and links to the vault contract.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `admin` - Admin address with permissions to add/remove strategies
    /// * `strategist` - Strategist address with permissions to adjust weights and rebalance
    /// * `vault` - The associated vault contract (the only contract that can allocate capital)
    ///
    /// # Panics
    /// 
    /// * If the router is already initialized (has admin set)
    ///
    /// # Example
    /// 
    /// ```ignore
    /// let admin = Address::generate(&env);
    /// let strategist = Address::generate(&env);
    /// let vault = Address::generate(&env);
    /// router.initialize(&admin, &strategist, &vault);
    /// ```
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

    /// Register a new yield strategy. Admin only.
    ///
    /// Adds a new strategy to the router's registry, enabling the vault to allocate
    /// capital to it. The strategy is initialized as active and can be adjusted later.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `caller` - The address making the call (must be the admin)
    /// * `strategy` - The strategy contract address to register
    /// * `weight_bps` - Initial allocation weight in basis points (0-10000).
    ///   100 = 1% of vault capital. Sum of all weights should be <= 10000.
    ///
    /// # Panics
    /// 
    /// * If the caller is not the admin
    ///
    /// # Implementation
    /// 
    /// This function should:
    /// 1. Verify caller is admin
    /// 2. Validate `weight_bps` is in valid range (0-10000)
    /// 3. Check strategy is not already registered
    /// 4. Validate total weights don't exceed 10000 bps
    /// 5. (Future) Call strategy.validate() to check interface compatibility
    /// 6. Create a [`StrategyInfo`] struct with weight and active status
    /// 7. Store it in [`DataKey::Strategy`] keyed by strategy address
    /// 8. Append strategy address to [`DataKey::StrategyList`]
    /// 9. Emit an event for transparency
    ///
    /// # Notes
    /// 
    /// The implementation is currently a TODO stub.
    ///
    /// TODO: Implement all validation and registration logic
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
    ///
    /// Marks a strategy as inactive, preventing new allocations to it.
    /// The strategy is not removed from the registry; historical data is preserved.
    /// Any capital already allocated to the strategy remains there.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `caller` - The address making the call (must be the admin)
    /// * `strategy` - The strategy address to deactivate
    ///
    /// # Panics
    /// 
    /// * If the caller is not the admin
    /// * If the strategy is not registered
    ///
    /// # Implementation
    /// 
    /// This function should:
    /// 1. Verify caller is admin
    /// 2. Retrieve the strategy from storage
    /// 3. (Future) If strategy has allocated capital, trigger withdrawal before deactivating
    /// 4. Set the strategy's `active` flag to `false`
    /// 5. Update storage
    ///
    /// # Notes
    /// 
    /// The implementation is currently a partial stub. Withdrawing capital from
    /// inactive strategies is TODO.
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

    /// Update the allocation weight of a strategy. Strategist only.
    ///
    /// Adjusts the percentage of vault capital that should be allocated to a specific strategy.
    /// Weight changes take effect on the next rebalancing.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `caller` - The address making the call (must be the strategist)
    /// * `strategy` - The strategy address to update
    /// * `new_weight_bps` - New allocation weight in basis points (0-10000).
    ///   100 = 1%. Sum across all strategies should be <= 10000.
    ///
    /// # Panics
    /// 
    /// * If the caller is not the strategist
    /// * If the strategy is not registered
    /// * If the new weight would cause total weights to exceed 10000
    ///
    /// # Implementation
    /// 
    /// This function should:
    /// 1. Verify caller is strategist
    /// 2. Retrieve strategy (panic if not found)
    /// 3. Validate new weight is in valid range (0-10000)
    /// 4. Check that new total weights don't exceed 10000 bps
    /// 5. Update strategy weight in storage
    /// 6. Emit an event for transparency
    /// 7. (Future) Automatically trigger rebalancing
    ///
    /// # Notes
    /// 
    /// The implementation is currently a TODO stub. Automatic rebalancing is not yet implemented.
    ///
    /// TODO: Implement validation and weight update logic
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

    /// Allocate capital to a strategy. Vault only.
    ///
    /// Transfers underlying assets from the vault to a strategy contract.
    /// Only the vault can call this function (enforced via caller verification).
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `caller` - The address making the call (must be the vault contract)
    /// * `strategy` - The strategy contract address to allocate to
    /// * `amount` - The amount of underlying tokens to allocate
    ///
    /// # Panics
    /// 
    /// * If the caller is not the vault contract
    /// * If the strategy is not registered or is inactive
    /// * If allocation violates weight constraints
    ///
    /// # Implementation
    /// 
    /// This function should:
    /// 1. Verify caller is the vault contract
    /// 2. Retrieve strategy info (panic if not found or inactive)
    /// 3. Validate allocation doesn't exceed strategy weight limits
    /// 4. Call strategy.deposit(amount) via cross-contract invocation to transfer funds
    /// 5. Update strategy's allocated balance
    /// 6. Update total_allocated counter
    /// 7. Emit an event for transparency
    ///
    /// # Notes
    /// 
    /// The implementation is currently a TODO stub. Cross-contract token transfers are not yet implemented.
    ///
    /// TODO: Implement cross-contract token transfers
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

    /// Retrieve configuration and state for a given strategy.
    ///
    /// This is a read-only query function that returns the [`StrategyInfo`] struct
    /// containing the strategy's address, weight, active status, and allocated capital.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `strategy` - The strategy address to query
    ///
    /// # Returns
    /// 
    /// The [`StrategyInfo`] struct for the strategy.
    ///
    /// # Panics
    /// 
    /// * If the strategy is not registered
    pub fn strategy_info(env: Env, strategy: Address) -> StrategyInfo {
        env.storage()
            .instance()
            .get(&DataKey::Strategy(strategy))
            .expect("strategy not found")
    }

    /// Retrieve the list of all registered strategy addresses.
    ///
    /// This is a read-only query function that returns a vector of all strategy
    /// contract addresses registered with the router. Includes both active and
    /// inactive strategies.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    ///
    /// # Returns
    /// 
    /// A vector of all registered strategy addresses. Returns an empty vector if
    /// no strategies have been registered.
    pub fn strategy_list(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::StrategyList)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Retrieve the total amount of capital allocated across all strategies.
    ///
    /// This is a read-only counter that sums the allocated balances of all strategies.
    /// It represents the total vault capital deployed to yield-generating strategies.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    ///
    /// # Returns
    /// 
    /// The total amount of capital allocated across all strategies. Returns `0` if
    /// no capital has been allocated.
    pub fn total_allocated(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalAllocated)
            .unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Internal helper: Assert that the caller is the admin.
    /// 
    /// Verifies that the calling address matches the stored admin address.
    /// Panics if the caller is not authorized.
    ///
    /// # Arguments
    /// 
    /// * `env` - The contract environment (borrowed)
    /// * `caller` - The calling address (borrowed)
    ///
    /// # Panics
    /// 
    /// * If the caller is not the admin
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

    /// Internal helper: Assert that the caller is the strategist.
    /// 
    /// Verifies that the calling address matches the stored strategist address.
    /// Panics if the caller is not authorized.
    ///
    /// # Arguments
    /// 
    /// * `env` - The contract environment (borrowed)
    /// * `caller` - The calling address (borrowed)
    ///
    /// # Panics
    /// 
    /// * If the caller is not the strategist
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

    /// Internal helper: Assert that the caller is the vault contract.
    /// 
    /// Verifies that the calling address matches the stored vault contract address.
    /// Panics if the caller is not authorized.
    ///
    /// # Arguments
    /// 
    /// * `env` - The contract environment (borrowed)
    /// * `caller` - The calling address (borrowed)
    ///
    /// # Panics
    /// 
    /// * If the caller is not the vault contract
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
