//! FlowFi Vault Contract
//!
//! The Vault is the central entry point for user funds. Users deposit assets
//! and receive "shares" representing their proportional ownership of the pool.
//! Shares allow proportional withdrawals and are used by the Rewards Engine
//! to compute reward entitlements.
//!
//! Architecture:
//!   User --> deposit(amount) --> Vault --> issues shares
//!   User --> withdraw(shares) --> Vault --> burns shares, returns assets
//!   Vault --> Strategy Router (for yield allocation, future)
//!
//! TODO: Add share price manipulation protection (ERC-4626-style virtual offset)
//! TODO: Integrate with Strategy Router for actual yield deployment
//! TODO: Add deposit/withdrawal caps (circuit breakers)
//! TODO: Support multiple underlying assets (multi-asset vault)
//! TODO: Add fee-on-deposit / fee-on-withdrawal for protocol revenue

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, Map,
};

/// Persistent storage keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Address of the underlying token (e.g. XLM or a Stellar asset)
    UnderlyingToken,
    /// Admin address (set at initialization)
    Admin,
    /// Total shares outstanding
    TotalShares,
    /// Per-user share balance mapping
    Shares(Address),
    /// Total assets currently held in the vault
    /// NOTE: This is naive — it doesn't account for assets deployed to strategies.
    /// TODO: Replace with cross-contract query to Strategy Router
    TotalAssets,
}

/// Events emitted by the vault
const EVT_DEPOSIT: &str = "deposit";
const EVT_WITHDRAW: &str = "withdraw";

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    /// Initialize the vault.
    ///
    /// # Arguments
    /// - `admin`: Protocol admin with elevated permissions
    /// - `token`: The underlying Stellar token this vault accepts
    pub fn initialize(env: Env, admin: Address, token: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::UnderlyingToken, &token);
        env.storage().instance().set(&DataKey::TotalShares, &0_i128);
        env.storage().instance().set(&DataKey::TotalAssets, &0_i128);
    }

    /// Deposit `amount` of the underlying token into the vault.
    /// Mints shares to `from` proportional to their deposit.
    ///
    /// Share calculation:
    ///   shares_out = (amount * total_shares) / total_assets
    ///   If vault is empty: shares_out = amount (1:1 bootstrap)
    ///
    /// # Arguments
    /// - `from`: The depositing user (must have authorized this call)
    /// - `amount`: Amount of underlying tokens to deposit
    ///
    /// TODO: Implement token transfer from user to vault
    /// TODO: Calculate shares to mint proportional to deposit
    /// TODO: Update user share balance and total shares
    /// TODO: Update total assets in vault
    /// TODO: Emit deposit event
    /// TODO: Add protection against share price manipulation (ERC-4626-style virtual offset)
    /// TODO: Apply a minimum deposit threshold to prevent dust attacks
    /// TODO: Emit richer events with share price snapshot
    pub fn deposit(env: Env, from: Address, amount: i128) -> i128 {
        from.require_auth();

        if amount <= 0 {
            panic!("deposit amount must be positive");
        }

        // TODO: Transfer tokens from user to vault
        // TODO: Calculate shares_to_mint based on total_assets and total_shares
        // TODO: Handle bootstrap case where vault is empty (1:1 ratio)
        // TODO: Mint shares to user (update storage)
        // TODO: Update total_assets
        // TODO: Emit deposit event

        panic!("TODO: deposit() implementation needed");
    }

    /// Withdraw assets by burning `shares`.
    /// Returns the underlying token amount sent to `to`.
    ///
    /// Amount calculation:
    ///   assets_out = (shares * total_assets) / total_shares
    ///
    /// # Arguments
    /// - `from`: The withdrawing user
    /// - `shares`: Number of shares to burn
    ///
    /// TODO: Verify user has sufficient shares to withdraw
    /// TODO: Calculate assets_out proportional to shares and total vault balance
    /// TODO: Burn shares (update storage)
    /// TODO: Transfer underlying tokens back to user
    /// TODO: Emit withdraw event
    /// TODO: Add a withdrawal fee mechanism
    /// TODO: Consider a withdrawal queue if strategy is illiquid
    pub fn withdraw(env: Env, from: Address, shares: i128) -> i128 {
        from.require_auth();

        if shares <= 0 {
            panic!("shares must be positive");
        }

        // TODO: Get user's share balance
        // TODO: Check user has sufficient shares (user_shares >= shares)
        // TODO: Get total_shares and total_assets
        // TODO: Calculate assets_out = shares * total_assets / total_shares
        // TODO: Burn shares from user and update total_shares
        // TODO: Decrement total_assets
        // TODO: Transfer tokens back to user
        // TODO: Emit withdraw event

        panic!("TODO: withdraw() implementation needed");
    }

    /// Returns the share balance of a given user.
    pub fn balance_of(env: Env, user: Address) -> i128 {
        Self::get_shares(&env, &user)
    }

    /// Returns the total assets held in the vault.
    ///
    /// TODO: Include assets deployed in strategies (requires cross-contract call)
    pub fn total_assets(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalAssets)
            .unwrap_or(0)
    }

    /// Returns the total shares outstanding.
    pub fn total_shares(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalShares)
            .unwrap_or(0)
    }

    /// Returns the current share price in terms of underlying assets.
    /// share_price = total_assets / total_shares
    ///
    /// Returns 1_000_000 (representing 1.0 with 6 decimal precision) if vault is empty.
    ///
    /// TODO: Use higher precision arithmetic to avoid rounding errors
    pub fn share_price(env: Env) -> i128 {
        let total_assets = Self::total_assets(env.clone());
        let total_shares = Self::total_shares(env);

        if total_shares == 0 {
            return 1_000_000; // 1.0 in 6-decimal fixed point
        }

        // Scale by 1e6 for fixed-point representation
        total_assets
            .checked_mul(1_000_000)
            .expect("overflow in share price")
            / total_shares
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn get_shares(env: &Env, user: &Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Shares(user.clone()))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, MockAuth, MockAuthInvoke},
        token::{Client as TokenClient, StellarAssetClient},
        Env, IntoVal,
    };

    fn setup_env() -> (Env, Address, Address, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        // Deploy a test token
        let token_id = env.register_stellar_asset_contract(admin.clone());

        (env, admin, user, token_id)
    }

    #[test]
    fn test_deposit_and_withdraw() {
        let (env, admin, user, token_id) = setup_env();
        env.mock_all_auths();

        // Mint tokens to user
        let token_admin = StellarAssetClient::new(&env, &token_id);
        token_admin.mint(&user, &1_000_000);

        // Deploy vault
        let vault_id = env.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&env, &vault_id);
        vault.initialize(&admin, &token_id);

        // First deposit
        let shares = vault.deposit(&user, &500_000);
        assert_eq!(shares, 500_000); // Bootstrap: 1:1
        assert_eq!(vault.balance_of(&user), 500_000);
        assert_eq!(vault.total_assets(), 500_000);

        // Withdraw half
        let assets_out = vault.withdraw(&user, &250_000);
        assert_eq!(assets_out, 250_000);
        assert_eq!(vault.balance_of(&user), 250_000);
    }

    #[test]
    fn test_share_price_initially_one() {
        let (env, admin, _, token_id) = setup_env();
        env.mock_all_auths();

        let vault_id = env.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&env, &vault_id);
        vault.initialize(&admin, &token_id);

        assert_eq!(vault.share_price(), 1_000_000);
    }

    // TODO: Test that share price increases after yield is added
    // TODO: Test deposit with non-zero existing shares (proportional minting)
    // TODO: Test withdraw with insufficient shares (expect panic)
    // TODO: Test zero-amount deposit (expect panic)
}
