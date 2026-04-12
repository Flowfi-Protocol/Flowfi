//! FlowFi Access Control Contract
//!
//! Manages role-based permissions across the FlowFi protocol.
//! Currently supports two roles: Admin and Strategist.
//!
//! TODO: Add more granular roles (e.g., guardian, fee collector)
//! TODO: Support multi-sig admin via threshold signatures
//! TODO: Add time-locked role transitions for security

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

/// Storage keys used by the access control contract
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Strategist,
    // TODO: Add Guardian role key
    // TODO: Add FeeCollector role key
}

/// Role identifiers used in events
const ROLE_ADMIN: Symbol = symbol_short!("ADMIN");
const ROLE_STRATEGIST: Symbol = symbol_short!("STRAT");

#[contract]
pub struct AccessControlContract;

#[contractimpl]
impl AccessControlContract {
    /// Initialize access control with an admin address.
    /// Must be called once immediately after deployment.
    ///
    /// TODO: Consider supporting an initial strategist address at init time
    pub fn initialize(env: Env, admin: Address) {
        // Prevent re-initialization
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);

        env.events().publish(
            (symbol_short!("INIT"), ROLE_ADMIN),
            admin,
        );
    }

    /// Returns the current admin address.
    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized")
    }

    /// Returns the current strategist address, if set.
    pub fn strategist(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Strategist)
    }

    /// Transfers the admin role to a new address.
    /// Only the current admin can perform this.
    ///
    /// TODO: Consider a two-step transfer (propose + accept) for safety
    pub fn transfer_admin(env: Env, new_admin: Address) {
        let current_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");

        current_admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &new_admin);

        env.events().publish(
            (symbol_short!("XFER"), ROLE_ADMIN),
            (current_admin, new_admin),
        );
    }

    /// Sets or updates the strategist role.
    /// Only the admin can assign strategists.
    ///
    /// TODO: Allow revoking the strategist role (set to None)
    pub fn set_strategist(env: Env, strategist: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");

        admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Strategist, &strategist);

        env.events().publish(
            (symbol_short!("SET"), ROLE_STRATEGIST),
            strategist,
        );
    }

    /// Checks whether an address holds the admin role.
    pub fn is_admin(env: Env, address: Address) -> bool {
        let admin: Option<Address> = env.storage().instance().get(&DataKey::Admin);
        admin.map_or(false, |a| a == address)
    }

    /// Checks whether an address holds the strategist role.
    pub fn is_strategist(env: Env, address: Address) -> bool {
        let strategist: Option<Address> = env.storage().instance().get(&DataKey::Strategist);
        strategist.map_or(false, |s| s == address)
    }

    /// Helper: asserts that the calling address is admin. Panics otherwise.
    /// Internal use by other contracts that import this module.
    ///
    /// TODO: Replace panic with proper error codes once error type system is standardized
    pub fn assert_admin(env: Env, caller: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized");

        if admin != caller {
            panic!("caller is not admin");
        }
    }

    /// Helper: asserts that the calling address is strategist. Panics otherwise.
    pub fn assert_strategist(env: Env, caller: Address) {
        let strategist: Address = env
            .storage()
            .instance()
            .get(&DataKey::Strategist)
            .expect("strategist not set");

        if strategist != caller {
            panic!("caller is not strategist");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AccessControlContract);
        let client = AccessControlContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        assert_eq!(client.admin(), admin);
    }

    #[test]
    fn test_transfer_admin() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, AccessControlContract);
        let client = AccessControlContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);

        client.initialize(&admin);
        client.transfer_admin(&new_admin);

        assert_eq!(client.admin(), new_admin);
    }

    #[test]
    fn test_set_strategist() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, AccessControlContract);
        let client = AccessControlContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let strategist = Address::generate(&env);

        client.initialize(&admin);
        client.set_strategist(&strategist);

        assert_eq!(client.strategist(), Some(strategist.clone()));
        assert!(client.is_strategist(&strategist));
    }

    // TODO: Add test for unauthorized transfer_admin (expect panic)
    // TODO: Add test for revoking roles once that feature is implemented
}
