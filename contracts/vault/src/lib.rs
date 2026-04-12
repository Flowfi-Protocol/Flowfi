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
    /// TODO: Apply a minimum deposit threshold to prevent dust attacks
    /// TODO: Emit richer events with share price snapshot
    pub fn deposit(env: Env, from: Address, amount: i128) -> i128 {
        from.require_auth();

        if amount <= 0 {
            panic!("deposit amount must be positive");
        }

        let token_id: Address = env
            .storage()
            .instance()
            .get(&DataKey::UnderlyingToken)
            .expect("not initialized");

        let total_assets: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalAssets)
            .unwrap_or(0);

        let total_shares: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalShares)
            .unwrap_or(0);

        // Transfer tokens from user to vault
        let token_client = token::Client::new(&env, &token_id);
        token_client.transfer(&from, &env.current_contract_address(), &amount);

        // Calculate shares to mint
        // TODO: This naive formula is vulnerable to inflation attacks when total_assets is 0
        //       but total_shares > 0 (shouldn't happen, but add invariant check)
        let shares_to_mint: i128 = if total_assets == 0 || total_shares == 0 {
            amount // Bootstrap: 1 token = 1 share
        } else {
            // shares = amount * total_shares / total_assets
            amount
                .checked_mul(total_shares)
                .expect("overflow in share calculation")
                / total_assets
        };

        if shares_to_mint <= 0 {
            panic!("zero shares minted — deposit too small");
        }

        // Update storage
        let user_shares: i128 = Self::get_shares(&env, &from);
        env.storage()
            .instance()
            .set(&DataKey::Shares(from.clone()), &(user_shares + shares_to_mint));

        env.storage()
            .instance()
            .set(&DataKey::TotalShares, &(total_shares + shares_to_mint));

        env.storage()
            .instance()
            .set(&DataKey::TotalAssets, &(total_assets + amount));

        env.events().publish(
            (symbol_short!("deposit"), from.clone()),
            (amount, shares_to_mint),
        );

        shares_to_mint
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
    /// TODO: Add a withdrawal fee mechanism
    /// TODO: Consider a withdrawal queue if strategy is illiquid
    pub fn withdraw(env: Env, from: Address, shares: i128) -> i128 {
        from.require_auth();

        if shares <= 0 {
            panic!("shares must be positive");
        }

        let user_shares = Self::get_shares(&env, &from);
        if user_shares < shares {
            panic!("insufficient shares");
        }

        let total_assets: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalAssets)
            .expect("vault not initialized");

        let total_shares: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalShares)
            .expect("vault not initialized");

        if total_shares == 0 {
            panic!("no shares outstanding");
        }

        // Calculate assets to return
        let assets_out: i128 = shares
            .checked_mul(total_assets)
            .expect("overflow")
            / total_shares;

        if assets_out <= 0 {
            panic!("zero assets out — shares too small");
        }

        let token_id: Address = env
            .storage()
            .instance()
            .get(&DataKey::UnderlyingToken)
            .expect("not initialized");

        // Burn shares and update state BEFORE transfer (checks-effects-interactions)
        env.storage()
            .instance()
            .set(&DataKey::Shares(from.clone()), &(user_shares - shares));

        env.storage()
            .instance()
            .set(&DataKey::TotalShares, &(total_shares - shares));

        env.storage()
            .instance()
            .set(&DataKey::TotalAssets, &(total_assets - assets_out));

        // Transfer tokens back to user
        let token_client = token::Client::new(&env, &token_id);
        token_client.transfer(&env.current_contract_address(), &from, &assets_out);

        env.events().publish(
            (symbol_short!("withdraw"), from.clone()),
            (shares, assets_out),
        );

        assets_out
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
