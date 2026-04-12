/**
 * FlowFi SDK — Type Definitions
 *
 * TODO: Generate these types automatically from Soroban contract bindings
 *       once `stellar contract bindings typescript` stabilizes.
 */

/** Configuration for the FlowFi SDK client */
export interface FlowFiConfig {
  /** Soroban RPC endpoint URL */
  rpcUrl: string;
  /** Stellar network passphrase */
  networkPassphrase: string;
  /** Deployed contract addresses */
  contracts: ContractAddresses;
}

/** Deployed contract addresses for a given network */
export interface ContractAddresses {
  vault: string;
  rewards: string;
  strategyRouter: string;
  accessControl: string;
}

/** Result of a deposit operation */
export interface DepositResult {
  /** Transaction hash */
  txHash: string;
  /** Number of shares minted */
  sharesMinted: bigint;
  /** Amount deposited */
  amountDeposited: bigint;
}

/** Result of a withdraw operation */
export interface WithdrawResult {
  txHash: string;
  /** Shares burned */
  sharesBurned: bigint;
  /** Underlying assets returned */
  assetsReturned: bigint;
}

/** Result of a claimRewards operation */
export interface ClaimRewardsResult {
  txHash: string;
  /** Reward amount claimed */
  rewardsClaimed: bigint;
}

/** Vault state snapshot */
export interface VaultState {
  totalAssets: bigint;
  totalShares: bigint;
  /** Share price in 6-decimal fixed point */
  sharePrice: bigint;
}

/** User position in the vault */
export interface UserPosition {
  shares: bigint;
  pendingRewards: bigint;
  /** Estimated underlying asset value of shares */
  estimatedValue: bigint;
}

/** Strategy information */
export interface StrategyInfo {
  address: string;
  weightBps: number;
  active: boolean;
  allocated: bigint;
}

// TODO: Add types for governance proposals (once governance is implemented)
// TODO: Add types for multi-strategy portfolio breakdown
// TODO: Add types for historical reward snapshots
