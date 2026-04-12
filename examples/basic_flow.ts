/**
 * FlowFi Protocol — Basic Flow Example
 *
 * Demonstrates the complete user lifecycle:
 *   1. Connect to testnet
 *   2. Check vault state
 *   3. Deposit tokens
 *   4. Wait (simulate time passing)
 *   5. Check pending rewards
 *   6. Claim rewards
 *   7. Withdraw
 *
 * Prerequisites:
 *   - Deploy contracts and update CONTRACT_ADDRESSES below
 *   - Set SECRET_KEY environment variable to a funded testnet keypair
 *
 * Run:
 *   cd examples
 *   npm install
 *   SECRET_KEY=S... npx ts-node basic_flow.ts
 */

import { FlowFiClient, Networks } from '../sdk/src';
import { Keypair } from '@stellar/stellar-sdk';

// ─── Configuration ────────────────────────────────────────────────────────────

const RPC_URL = 'https://soroban-testnet.stellar.org';
const NETWORK_PASSPHRASE = Networks.TESTNET;

// Update these with your deployed contract addresses (from .deployed.json)
const CONTRACT_ADDRESSES = {
  vault: process.env.VAULT_CONTRACT_ID ?? '',
  rewards: process.env.REWARDS_CONTRACT_ID ?? '',
  strategyRouter: process.env.STRATEGY_ROUTER_CONTRACT_ID ?? '',
  accessControl: process.env.ACCESS_CONTROL_CONTRACT_ID ?? '',
};

// ─── Main ─────────────────────────────────────────────────────────────────────

async function main() {
  const secretKey = process.env.SECRET_KEY;
  if (!secretKey) {
    console.error('❌ Set SECRET_KEY environment variable to a testnet secret key');
    process.exit(1);
  }

  if (!CONTRACT_ADDRESSES.vault) {
    console.error('❌ Set VAULT_CONTRACT_ID (and other contract IDs) as env vars');
    process.exit(1);
  }

  const signer = Keypair.fromSecret(secretKey);
  console.log(`🔑 Using account: ${signer.publicKey()}\n`);

  const client = new FlowFiClient({
    rpcUrl: RPC_URL,
    networkPassphrase: NETWORK_PASSPHRASE,
    contracts: CONTRACT_ADDRESSES,
  });

  // ─── 1. Check vault state ──────────────────────────────────────────────────
  console.log('📊 Fetching vault state...');
  const vaultState = await client.getVaultState();
  console.log(`   Total Assets: ${vaultState.totalAssets.toLocaleString()} stroops`);
  console.log(`   Total Shares: ${vaultState.totalShares.toLocaleString()}`);
  console.log(`   Share Price:  ${(Number(vaultState.sharePrice) / 1_000_000).toFixed(6)}`);
  console.log('');

  // ─── 2. Check existing position ───────────────────────────────────────────
  console.log('👤 Checking your position...');
  const before = await client.getUserPosition(signer.publicKey());
  console.log(`   Shares:           ${before.shares.toLocaleString()}`);
  console.log(`   Pending Rewards:  ${before.pendingRewards.toLocaleString()}`);
  console.log(`   Estimated Value:  ${before.estimatedValue.toLocaleString()} stroops`);
  console.log('');

  // ─── 3. Deposit ───────────────────────────────────────────────────────────
  const depositAmount = 1_000_000n; // 0.1 XLM in stroops
  console.log(`💰 Depositing ${depositAmount.toLocaleString()} stroops...`);

  const depositResult = await client.deposit(signer, depositAmount);
  console.log(`   ✅ Deposit confirmed: ${depositResult.txHash}`);
  console.log(`   Shares minted: ${depositResult.sharesMinted.toLocaleString()}`);
  console.log('');

  // ─── 4. Position after deposit ────────────────────────────────────────────
  const afterDeposit = await client.getUserPosition(signer.publicKey());
  console.log('📈 Position after deposit:');
  console.log(`   Shares: ${afterDeposit.shares.toLocaleString()}`);
  console.log('');

  // ─── 5. Check pending rewards ─────────────────────────────────────────────
  console.log('⏳ Checking pending rewards (rewards accrue over ledgers)...');
  const pending = await client.getPendingRewards(signer.publicKey());
  console.log(`   Pending: ${pending.toLocaleString()} reward tokens`);
  console.log('');

  // ─── 6. Claim rewards ─────────────────────────────────────────────────────
  if (pending > 0n) {
    console.log('🎁 Claiming rewards...');
    const claimResult = await client.claimRewards(signer);
    console.log(`   ✅ Claimed: ${claimResult.rewardsClaimed.toLocaleString()} tokens`);
    console.log(`   Tx: ${claimResult.txHash}`);
    console.log('');
  } else {
    console.log('ℹ️  No rewards to claim yet (need more ledgers to pass)');
    console.log('');
  }

  // ─── 7. Withdraw half ─────────────────────────────────────────────────────
  const sharesToWithdraw = afterDeposit.shares / 2n;
  if (sharesToWithdraw > 0n) {
    console.log(`🏧 Withdrawing ${sharesToWithdraw.toLocaleString()} shares...`);
    const withdrawResult = await client.withdraw(signer, sharesToWithdraw);
    console.log(`   ✅ Withdrew: ${withdrawResult.assetsReturned.toLocaleString()} stroops`);
    console.log(`   Tx: ${withdrawResult.txHash}`);
    console.log('');
  }

  // ─── 8. Final position ────────────────────────────────────────────────────
  const final = await client.getUserPosition(signer.publicKey());
  console.log('🏁 Final position:');
  console.log(`   Shares:          ${final.shares.toLocaleString()}`);
  console.log(`   Pending Rewards: ${final.pendingRewards.toLocaleString()}`);
  console.log(`   Estimated Value: ${final.estimatedValue.toLocaleString()} stroops`);
  console.log('');
  console.log('✅ Basic flow complete!');
}

main().catch((err) => {
  console.error('Fatal error:', err);
  process.exit(1);
});
