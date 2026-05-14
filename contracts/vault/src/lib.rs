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

/// Persistent storage keys used by the Vault contract.
/// 
/// These keys map to different types of data stored in the contract's instance storage.
/// Keys use Soroban's `Symbol`-based storage mechanism for gas efficiency.
/// 
/// # Storage Layout
/// 
/// - `UnderlyingToken`: Single value (Address)
/// - `Admin`: Single value (Address) 
/// - `TotalShares`: Single value (i128)
/// - `Shares(user)`: Per-user mapping (i128)
/// - `TotalAssets`: Single value (i128)
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Address of the underlying token (e.g. XLM or a Stellar asset).
    /// Set once at initialization and cannot be changed.
    UnderlyingToken,
    /// Admin address with elevated permissions (set at initialization).
    /// Only the admin can call governance functions.
    Admin,
    /// Total shares outstanding across all users.
    /// Increases on deposits, decreases on withdrawals.
    TotalShares,
    /// Per-user share balance mapping.
    /// Stores the number of vault shares owned by each user.
    Shares(Address),
    /// Total assets currently held in the vault.
    /// NOTE: This is a naive accounting—it does not include assets deployed to strategies.
    /// TODO: Replace with cross-contract query to Strategy Router for accurate TVL.
    TotalAssets,
}

/// Event emitted when a user deposits tokens into the vault.
/// 
/// Event data: `("deposit", user_address)` → `(amount_deposited, shares_minted)`
const EVT_DEPOSIT: &str = "deposit";

/// Event emitted when a user withdraws tokens from the vault.
/// 
/// Event data: `("withdraw", user_address)` → `(shares_burned, amount_returned)`
const EVT_WITHDRAW: &str = "withdraw";

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    /// Initialize the vault with an admin and underlying token.
    ///
    /// This function must be called exactly once after the vault contract is deployed.
    /// It sets up the initial state, including storage for shares, assets, and permissions.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `admin` - Protocol admin address with elevated permissions. This address can perform
    ///   governance functions (e.g., updating vault parameters in future versions).
    /// * `token` - The underlying Stellar token contract address. All deposits/withdrawals use this token.
    ///
    /// # Panics
    /// 
    /// * If the vault is already initialized (has admin set)
    ///
    /// # Example
    /// 
    /// ```ignore
    /// let admin = Address::generate(&env);
    /// let token = env.register_stellar_asset_contract(admin.clone());
    /// vault.initialize(&admin, &token);
    /// ```
    pub fn initialize(env: Env, admin: Address, token: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::UnderlyingToken, &token);
        env.storage().instance().set(&DataKey::TotalShares, &0_i128);
        env.storage().instance().set(&DataKey::TotalAssets, &0_i128);
    }

    /// Deposit tokens into the vault and receive shares in return.
    ///
    /// Users deposit their underlying tokens and receive vault shares proportional to their
    /// contribution. Shares represent ownership of the pool and are used to calculate
    /// withdrawal amounts and reward entitlements.
    ///
    /// # Share Calculation
    /// 
    /// ```text
    /// If vault is empty (total_assets == 0):
    ///   shares_out = amount (1:1 bootstrap)
    /// 
    /// Otherwise:
    ///   shares_out = (amount * total_shares) / total_assets
    /// ```
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `from` - The depositing user address. Must have authorized this call via Soroban's auth framework.
    /// * `amount` - Amount of underlying tokens to deposit (in the token's smallest unit).
    ///   Must be positive.
    ///
    /// # Returns
    /// 
    /// The number of vault shares minted to the user.
    ///
    /// # Panics
    /// 
    /// * If `amount <= 0` (deposit must be positive)
    /// * If implementation is incomplete (TODO)
    ///
    /// # Events
    /// 
    /// Emits `EVT_DEPOSIT` event with user address and minted shares.
    ///
    /// # Implementation Notes
    /// 
    /// This is a stub that panics with a TODO message. Full implementation requires:
    /// - Transfer tokens from user to vault using Soroban's token interface
    /// - Calculate proportional shares based on current vault state
    /// - Update user's share balance and total shares in storage
    /// - Update total assets tracked by vault
    ///
    /// TODO: Implement token transfer and share minting
    /// TODO: Add ERC-4626-style virtual offset to prevent share price manipulation
    /// TODO: Add minimum deposit threshold to prevent dust attacks
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

    /// Withdraw underlying tokens by burning vault shares.
    ///
    /// Users burn shares to receive their proportional share of the vault's underlying assets.
    /// The amount received is calculated based on the current share price.
    ///
    /// # Amount Calculation
    /// 
    /// ```text
    /// assets_out = (shares * total_assets) / total_shares
    /// ```
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `from` - The withdrawing user address. Must have authorized this call.
    /// * `shares` - Number of vault shares to burn. Must be positive and within user's balance.
    ///
    /// # Returns
    /// 
    /// The number of underlying tokens returned to the user.
    ///
    /// # Panics
    /// 
    /// * If `shares <= 0` (withdrawal must be positive)
    /// * If user has insufficient shares (insufficient balance)
    /// * If implementation is incomplete (TODO)
    ///
    /// # Events
    /// 
    /// Emits `EVT_WITHDRAW` event with user address and returned assets.
    ///
    /// # Implementation Notes
    /// 
    /// This is a stub that panics with a TODO message. Full implementation requires:
    /// - Verify user has sufficient shares
    /// - Calculate assets to return based on current share price
    /// - Burn shares from user's balance
    /// - Update total shares and total assets
    /// - Transfer underlying tokens back to user
    ///
    /// TODO: Implement share burning and token transfers
    /// TODO: Add withdrawal fee mechanism for protocol revenue
    /// TODO: Consider withdrawal queue for illiquid strategies
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

    /// Returns the vault share balance of a given user.
    ///
    /// This is a read-only function that returns the number of vault shares owned by the user.
    /// Shares represent the user's proportional ownership of the vault's assets.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    /// * `user` - The user address to query
    ///
    /// # Returns
    /// 
    /// The number of vault shares owned by the user. Returns `0` if the user has never deposited.
    pub fn balance_of(env: Env, user: Address) -> i128 {
        Self::get_shares(&env, &user)
    }

    /// Returns the total underlying assets currently held in the vault.
    ///
    /// This is a read-only function that returns the total amount of underlying tokens
    /// held in the vault. It does not include assets that have been allocated to strategies.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    ///
    /// # Returns
    /// 
    /// The total underlying tokens in the vault (in the token's smallest unit).
    ///
    /// # Notes
    /// 
    /// The vault's total assets are tracked naively. Once the Strategy Router is integrated,
    /// this should include a cross-contract query to sum assets deployed in strategies.
    /// For now, it only reflects tokens directly held in the vault.
    ///
    /// TODO: Include assets deployed in strategies via cross-contract queries
    pub fn total_assets(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalAssets)
            .unwrap_or(0)
    }

    /// Returns the total number of vault shares outstanding.
    ///
    /// This is a read-only function that returns the total supply of vault shares
    /// across all users. This value increases with deposits and decreases with withdrawals.
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    ///
    /// # Returns
    /// 
    /// The total shares in circulation. Returns `0` if the vault is empty.
    pub fn total_shares(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalShares)
            .unwrap_or(0)
    }

    /// Returns the current share price in terms of underlying assets.
    ///
    /// The share price represents how many underlying tokens each vault share is worth.
    /// It is calculated as the ratio of total assets to total shares, expressed in
    /// 6-decimal fixed-point notation.
    ///
    /// # Calculation
    /// 
    /// ```text
    /// share_price = (total_assets * 1_000_000) / total_shares
    /// 
    /// If total_shares == 0:
    ///   share_price = 1_000_000 (representing 1.0)
    /// ```
    ///
    /// # Arguments
    /// 
    /// * `env` - The Soroban contract environment
    ///
    /// # Returns
    /// 
    /// The current share price in 6-decimal fixed-point notation.
    /// For example:
    /// - `1_000_000` = 1.0 tokens per share
    /// - `1_500_000` = 1.5 tokens per share
    /// - `500_000` = 0.5 tokens per share
    ///
    /// # Panics
    /// 
    /// * If multiplication overflows (theoretical, should not occur in practice)
    ///
    /// # Notes
    /// 
    /// When the vault is empty (total_shares == 0), returns 1.0 as a bootstrap value.
    /// This ensures the first deposit receives 1:1 shares for their tokens.
    ///
    /// TODO: Use higher precision arithmetic (e.g., i256) to minimize rounding errors
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

    /// Internal helper: Retrieves the share balance for a user from storage.
    ///
    /// # Arguments
    /// 
    /// * `env` - The contract environment (borrowed)
    /// * `user` - The user address to query
    ///
    /// # Returns
    /// 
    /// The user's share balance. Returns `0` if the user has no entry in storage.
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
