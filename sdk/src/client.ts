/**
 * FlowFi SDK — Main Client
 *
 * Provides a high-level TypeScript interface for interacting with
 * the FlowFi Protocol smart contracts on Stellar/Soroban.
 *
 * Usage:
 * ```typescript
 * import { FlowFiClient } from '@flowfi/sdk';
 *
 * const client = new FlowFiClient({
 *   rpcUrl: 'https://soroban-testnet.stellar.org',
 *   networkPassphrase: Networks.TESTNET,
 *   contracts: { vault: 'C...', rewards: 'C...', strategyRouter: 'C...', accessControl: 'C...' }
 * });
 *
 * const result = await client.deposit(keypair, 1_000_000n);
 * ```
 *
 * TODO: Add retry logic for failed transactions
 * TODO: Add fee bumping support for congested networks
 * TODO: Add simulation-before-submit for better error messages
 * TODO: Support hardware wallet signers (Ledger)
 */

import {
  Contract,
  Keypair,
  Networks,
  SorobanRpc,
  TransactionBuilder,
  nativeToScVal,
  scValToNative,
  xdr,
} from '@stellar/stellar-sdk';

import {
  FlowFiConfig,
  DepositResult,
  WithdrawResult,
  ClaimRewardsResult,
  VaultState,
  UserPosition,
  StrategyInfo,
} from './types';

/** Base fee in stroops for Soroban transactions */
const BASE_FEE = '100';

/** Default transaction timeout in seconds */
const TX_TIMEOUT_SECONDS = 30;

export class FlowFiClient {
  private readonly config: FlowFiConfig;
  private readonly server: SorobanRpc.Server;

  constructor(config: FlowFiConfig) {
    this.config = config;
    this.server = new SorobanRpc.Server(config.rpcUrl, {
      allowHttp: config.rpcUrl.startsWith('http://'),
    });
  }

  // ---------------------------------------------------------------------------
  // Vault interactions
  // ---------------------------------------------------------------------------

  /**
   * Deposit underlying tokens into the FlowFi vault.
   *
   * @param signer - Keypair of the depositing user
   * @param amount - Amount to deposit (in token's smallest unit)
   * @returns DepositResult with tx hash and shares minted
   *
   * TODO: Add slippage protection (min shares out)
   * TODO: Support non-Keypair signers (e.g. WalletConnect)
   */
  async deposit(signer: Keypair, amount: bigint): Promise<DepositResult> {
    const account = await this.server.getAccount(signer.publicKey());

    const contract = new Contract(this.config.contracts.vault);

    const operation = contract.call(
      'deposit',
      nativeToScVal(signer.publicKey(), { type: 'address' }),
      nativeToScVal(amount, { type: 'i128' })
    );

    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: this.config.networkPassphrase,
    })
      .addOperation(operation)
      .setTimeout(TX_TIMEOUT_SECONDS)
      .build();

    const preparedTx = await this.server.prepareTransaction(tx);
    preparedTx.sign(signer);

    const response = await this.server.sendTransaction(preparedTx);

    if (response.status === 'ERROR') {
      throw new Error(`Transaction failed: ${JSON.stringify(response)}`);
    }

    // Poll for confirmation
    const confirmed = await this.pollForConfirmation(response.hash);

    // Extract shares minted from return value
    // TODO: Parse the actual return value from the confirmed tx result
    const sharesMinted = this.extractReturnValue(confirmed);

    return {
      txHash: response.hash,
      sharesMinted,
      amountDeposited: amount,
    };
  }

  /**
   * Withdraw assets from the vault by burning shares.
   *
   * @param signer - Keypair of the withdrawing user
   * @param shares - Number of shares to burn
   * @returns WithdrawResult with tx hash and assets returned
   *
   * TODO: Add minimum assets out parameter for slippage protection
   */
  async withdraw(signer: Keypair, shares: bigint): Promise<WithdrawResult> {
    const account = await this.server.getAccount(signer.publicKey());

    const contract = new Contract(this.config.contracts.vault);

    const operation = contract.call(
      'withdraw',
      nativeToScVal(signer.publicKey(), { type: 'address' }),
      nativeToScVal(shares, { type: 'i128' })
    );

    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: this.config.networkPassphrase,
    })
      .addOperation(operation)
      .setTimeout(TX_TIMEOUT_SECONDS)
      .build();

    const preparedTx = await this.server.prepareTransaction(tx);
    preparedTx.sign(signer);

    const response = await this.server.sendTransaction(preparedTx);

    if (response.status === 'ERROR') {
      throw new Error(`Transaction failed: ${JSON.stringify(response)}`);
    }

    const confirmed = await this.pollForConfirmation(response.hash);
    const assetsReturned = this.extractReturnValue(confirmed);

    return {
      txHash: response.hash,
      sharesBurned: shares,
      assetsReturned,
    };
  }

  /**
   * Claim accumulated reward tokens.
   *
   * @param signer - Keypair of the claiming user
   * @returns ClaimRewardsResult with tx hash and amount claimed
   */
  async claimRewards(signer: Keypair): Promise<ClaimRewardsResult> {
    const account = await this.server.getAccount(signer.publicKey());

    const contract = new Contract(this.config.contracts.rewards);

    const operation = contract.call(
      'claim_rewards',
      nativeToScVal(signer.publicKey(), { type: 'address' })
    );

    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: this.config.networkPassphrase,
    })
      .addOperation(operation)
      .setTimeout(TX_TIMEOUT_SECONDS)
      .build();

    const preparedTx = await this.server.prepareTransaction(tx);
    preparedTx.sign(signer);

    const response = await this.server.sendTransaction(preparedTx);

    if (response.status === 'ERROR') {
      throw new Error(`Transaction failed: ${JSON.stringify(response)}`);
    }

    const confirmed = await this.pollForConfirmation(response.hash);
    const rewardsClaimed = this.extractReturnValue(confirmed);

    return {
      txHash: response.hash,
      rewardsClaimed,
    };
  }

  // ---------------------------------------------------------------------------
  // Read-only queries
  // ---------------------------------------------------------------------------

  /**
   * Fetch the current vault state (total assets, shares, share price).
   *
   * Uses simulation (no transaction) for read-only calls.
   *
   * TODO: Batch these three calls into a single simulation
   */
  async getVaultState(): Promise<VaultState> {
    const contract = new Contract(this.config.contracts.vault);

    const [totalAssets, totalShares, sharePrice] = await Promise.all([
      this.simulateRead(contract, 'total_assets', []),
      this.simulateRead(contract, 'total_shares', []),
      this.simulateRead(contract, 'share_price', []),
    ]);

    return {
      totalAssets: BigInt(totalAssets),
      totalShares: BigInt(totalShares),
      sharePrice: BigInt(sharePrice),
    };
  }

  /**
   * Fetch a user's current position (shares and pending rewards).
   *
   * @param userPublicKey - Stellar public key of the user
   *
   * TODO: Also return historical deposit amount for PnL tracking
   */
  async getUserPosition(userPublicKey: string): Promise<UserPosition> {
    const vaultContract = new Contract(this.config.contracts.vault);
    const rewardsContract = new Contract(this.config.contracts.rewards);

    const userAddress = nativeToScVal(userPublicKey, { type: 'address' });

    const [shares, pendingRewards, sharePrice] = await Promise.all([
      this.simulateRead(vaultContract, 'balance_of', [userAddress]),
      this.simulateRead(rewardsContract, 'pending_rewards', [userAddress]),
      this.simulateRead(vaultContract, 'share_price', []),
    ]);

    const sharesBigInt = BigInt(shares);
    const sharePriceBigInt = BigInt(sharePrice);

    // estimated value = shares * sharePrice / 1_000_000 (6-decimal precision)
    const estimatedValue = (sharesBigInt * sharePriceBigInt) / 1_000_000n;

    return {
      shares: sharesBigInt,
      pendingRewards: BigInt(pendingRewards),
      estimatedValue,
    };
  }

  /**
   * Fetch pending rewards for a user without claiming.
   *
   * @param userPublicKey - Stellar public key of the user
   */
  async getPendingRewards(userPublicKey: string): Promise<bigint> {
    const contract = new Contract(this.config.contracts.rewards);
    const userAddress = nativeToScVal(userPublicKey, { type: 'address' });

    const result = await this.simulateRead(contract, 'pending_rewards', [userAddress]);
    return BigInt(result);
  }

  /**
   * Fetch info about all registered strategies.
   *
   * TODO: This is currently incomplete — needs parsing of Vec<Address> from Soroban
   */
  async getStrategies(): Promise<StrategyInfo[]> {
    // TODO: Implement full strategy list fetching
    // Steps:
    // 1. Call strategy_router.strategy_list() to get Vec<Address>
    // 2. For each address, call strategy_router.strategy_info(address)
    // 3. Parse and return all StrategyInfo objects
    throw new Error('getStrategies() not yet fully implemented — see TODO');
  }

  // ---------------------------------------------------------------------------
  // Internal helpers
  // ---------------------------------------------------------------------------

  /**
   * Submit a read-only call via transaction simulation.
   * No fee is charged; just reads on-chain state.
   */
  private async simulateRead(
    contract: Contract,
    method: string,
    args: xdr.ScVal[]
  ): Promise<string> {
    const account = await this.server.getAccount(
      // Use a dummy account for simulation reads
      // TODO: Allow passing a real account for authenticated reads
      'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN'
    );

    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: this.config.networkPassphrase,
    })
      .addOperation(contract.call(method, ...args))
      .setTimeout(TX_TIMEOUT_SECONDS)
      .build();

    const simResult = await this.server.simulateTransaction(tx);

    if (SorobanRpc.Api.isSimulationError(simResult)) {
      throw new Error(`Simulation failed: ${simResult.error}`);
    }

    if (!simResult.result) {
      throw new Error('Simulation returned no result');
    }

    return scValToNative(simResult.result.retval).toString();
  }

  /**
   * Poll the RPC for a transaction to be confirmed (SUCCESS or FAILED).
   *
   * TODO: Add configurable timeout and retry limits
   * TODO: Use exponential backoff instead of fixed interval
   */
  private async pollForConfirmation(
    txHash: string
  ): Promise<SorobanRpc.Api.GetTransactionResponse> {
    const maxAttempts = 30;
    const pollIntervalMs = 1000;

    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      await new Promise((resolve) => setTimeout(resolve, pollIntervalMs));

      const response = await this.server.getTransaction(txHash);

      if (response.status === SorobanRpc.Api.GetTransactionStatus.SUCCESS) {
        return response;
      }

      if (response.status === SorobanRpc.Api.GetTransactionStatus.FAILED) {
        throw new Error(`Transaction failed: ${txHash}`);
      }

      // NOT_FOUND means still pending — keep polling
    }

    throw new Error(`Transaction not confirmed after ${maxAttempts} attempts: ${txHash}`);
  }

  /**
   * Extract the i128 return value from a confirmed Soroban transaction.
   *
   * TODO: This is brittle — improve once Stellar SDK provides better helpers
   */
  private extractReturnValue(response: SorobanRpc.Api.GetTransactionResponse): bigint {
    if (response.status !== SorobanRpc.Api.GetTransactionStatus.SUCCESS) {
      throw new Error('Cannot extract value from non-success transaction');
    }

    // TODO: Properly parse returnValue XDR from response
    // This is a placeholder — needs proper implementation
    const returnValue = (response as any).returnValue;
    if (!returnValue) {
      return 0n;
    }

    return BigInt(scValToNative(returnValue));
  }
}
