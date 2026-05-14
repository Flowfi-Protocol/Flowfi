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

/// Storage keys used by the Access Control contract.
/// 
/// These keys store the addresses of accounts holding each role.
/// Currently supports Admin and Strategist roles.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Admin address with the highest privileges.
    /// Can transfer admin role and assign strategists.
    Admin,
    /// Strategist address with operational permissions.
    /// Can adjust strategy weights and trigger rebalances.
    Strategist,
    // TODO: Add Guardian role key for emergency controls
    // TODO: Add FeeCollector role key for protocol revenue management
}

/// Role identifier for the Admin role used in events.
const ROLE_ADMIN: Symbol = symbol_short!("ADMIN");

/// Role identifier for the Strategist role used in events.
const ROLE_STRATEGIST: Symbol = symbol_short!("STRAT");

#[contract]
pub struct AccessControlContract;

#[contractimpl]the access control system with an admin address.
    ///
    /// This function must be called exactly once immediately after the contract is deployed.
    /// It sets up the initial admin who can manage other roles.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `admin` - The initial admin address. This address will have permission to
    ///   transfer the admin role to another address and assign strategists.
    ///
    /// # Panics
    /// 
    /// * If the contract is already initialized (has admin set)
    ///
    /// # Events
    /// 
    /// Emits an `(\"INIT\", \"ADMIN\")` event with the admin address.
    ///
    /// # Example
    /// rieve the current admin address.
    ///
    /// This is a read-only query function that returns the address currently
    /// holding the admin role.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    ///
    /// # Returns
    /// 
    /// The admin address.
    ///
    /// # Panics
    /// 
    /// * If the contract is not initialized
    /// ```ignore
    /// let admin = Address::generate(&env);
    /// access_control.initialize(&admin);
    /// ```
    ///
    /// # Notes
    /// 
    /// The initial strategist is not set at initialization. Use [`set_strategist`](AccessControlContract::set_strategist)
    /// after initialization to assign a strategis
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
rieve the current strategist address, if one is set.
    ///
    /// This is a read-only query function that returns the address currently
    /// holding the strategist role, if any.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    ///
    /// # Returns
    /// 
    /// The strategist address wrapped in `Some(...)`, or `None` if no strategist
    /// has been assigned y
        env.events().publish(
            (symbol_short!("INIT"), ROLE_ADMIN),
            admin,
        );
    }

    /// Returns the current admin address.
    pub fn admin the admin role to a new address.
    ///
    /// Only the current admin can call this function. After the transfer, the new admin
    /// gains all admin permissions and the previous admin loses them.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `new_admin` - The address to transfer admin permissions to
    ///
    /// # Panics
    /// 
    /// * If the current admin does not authorize this call (checked via `require_auth()`)
    /// * If the contract is not initialized
    ///
    /// # Events
    /// 
    /// Emits a `(\"XFER\", \"ADMIN\")` event with the old and new admin addresses.
    ///
    /// # Notes
    /// 
    /// Assign a strategist to the Strategist role.
    ///
    /// Only the admin can call this function. If a strategist is already set,
    /// this replaces them with the new address.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `strategist` - The address to assign to the Strategist role
    ///
    /// # Panics
    /// 
    /// * If the admin does not authorize this call (checked via `require_auth()`)
    /// * If the contract is not initialized
    ///
    /// # Events
    /// 
    /// Emits a `(\"SET\", \"STRAT\")` event with the new strategist address.
    ///
    /// # Notes
    /// 
    /// To revoke the strategist role (unset it), you must implement that feature first.
    /// Currently, there's no way to set the strategist to `None` once assignedions, consider a two-step process
    /// (propose + accept) in future version
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized")
    }

    /// Returns the current strategist address, if set.
    pub fn strategist(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Strategist)
    }

    /// Transfers the admin role to a new address.
    /// Only  whether an address holds the admin role.
    ///
    /// This is a read-only query function that returns whether the given address
    /// is the current admin.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `address` - The address to check
    ///
    /// # Returns
    /// 
    /// `true` if the address is the current admin, `false` otherwis
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
            ( whether an address holds the strategist role.
    ///
    /// This is a read-only query function that returns whether the given address
    /// is the current strategist.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `address` - The address to check
    ///
    /// # Returns
    /// 
    /// `true` if the address is the current strategist, `false` otherwise.
    /// Returns `false` if no strategist has been set yet
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
 function: Assert that the calling address is the admin.
    ///
    /// This is an internal helper function intended to be called by other contracts
    /// that import this module for authorization checks. It verifies the caller is
    /// the admin and panics otherwise.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `caller` - The address to check for admin permissions
    ///
    /// # Panics
    /// 
    /// * If the caller is not the admin
    /// * If the contract is not initialized
    ///
    /// # Notes
    /// 
    /// The caller is expected to have already called `require_auth()` to verify
    /// cryptographic authorization. This function only checks the ro

        env.storage()
            .instance()
            .set(&DataKey::Strategist, &strategist);

        env.events().publish(
            (symbol_short!("SET"), ROLE_STRATEGIST),
            strategist,
        ); function: Assert that the calling address is the strategist.
    ///
    /// This is an internal helper function intended to be called by other contracts
    /// that import this module for authorization checks. It verifies the caller is
    /// the strategist and panics otherwise.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `caller` - The address to check for strategist permissions
    ///
    /// # Panics
    /// 
    /// * If the caller is not the strategist
    /// * If no strategist has been set yet
    /// * If the contract is not initialized
    ///
    /// # Notes
    /// 
    /// The caller is expected to have already called `require_auth()` to verify
    /// cryptographic authorization. This function only checks the rol
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
