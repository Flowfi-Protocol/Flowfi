/**
 * @flowfi/sdk — FlowFi Protocol TypeScript SDK
 *
 * @example
 * ```typescript
 * import { FlowFiClient, Networks } from '@flowfi/sdk';
 *
 * const client = new FlowFiClient({
 *   rpcUrl: 'https://soroban-testnet.stellar.org',
 *   networkPassphrase: Networks.TESTNET,
 *   contracts: {
 *     vault: 'CABC...',
 *     rewards: 'CDEF...',
 *     strategyRouter: 'CGHI...',
 *     accessControl: 'CJKL...',
 *   }
 * });
 *
 * const position = await client.getUserPosition(myPublicKey);
 * console.log('My shares:', position.shares);
 * ```
 */

export { FlowFiClient } from './client';
export * from './types';

// Re-export useful Stellar SDK utilities for consumer convenience
export { Networks, Keypair } from '@stellar/stellar-sdk';
