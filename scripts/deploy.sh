#!/usr/bin/env bash
# FlowFi Protocol — Deployment Script
#
# Deploys all FlowFi contracts to the configured Soroban network.
# Contracts are deployed in dependency order and cross-initialized.
#
# Usage:
#   cp .env.example .env   # Fill in your values
#   source .env
#   chmod +x scripts/deploy.sh
#   ./scripts/deploy.sh
#
# TODO: Add --dry-run flag that simulates deployment without committing
# TODO: Add --upgrade flag for re-deploying existing contracts
# TODO: Add health checks after each deployment

set -euo pipefail

# -----------------------------------------------------------------------
# Validate environment
# -----------------------------------------------------------------------

REQUIRED_VARS=(
  "SOROBAN_NETWORK"
  "SOROBAN_RPC_URL"
  "DEPLOYER_SECRET_KEY"
  "REWARD_TOKEN_ADDRESS"
)

for var in "${REQUIRED_VARS[@]}"; do
  if [[ -z "${!var:-}" ]]; then
    echo "❌ Missing required environment variable: $var"
    echo "   Copy .env.example to .env and fill in your values."
    exit 1
  fi
done

echo "🚀 FlowFi Protocol — Deployment"
echo "   Network:  $SOROBAN_NETWORK"
echo "   RPC URL:  $SOROBAN_RPC_URL"
echo ""

DEPLOYER_PUBLIC=$(stellar keys public-key deployer 2>/dev/null || \
  stellar keys generate deployer --secret-key "$DEPLOYER_SECRET_KEY" --network "$SOROBAN_NETWORK" > /dev/null && \
  stellar keys public-key deployer)

echo "   Deployer: $DEPLOYER_PUBLIC"
echo ""

# -----------------------------------------------------------------------
# Build all contracts
# -----------------------------------------------------------------------

echo "🔨 Building contracts..."

cargo build \
  --target wasm32-unknown-unknown \
  --release \
  --workspace \
  2>&1 | tail -5

echo "✅ Build complete"
echo ""

# -----------------------------------------------------------------------
# Optimize WASM binaries
# -----------------------------------------------------------------------

echo "⚙️  Optimizing WASM..."

for contract in access_control vault rewards strategy_router; do
  stellar contract optimize \
    --wasm "target/wasm32-unknown-unknown/release/flowfi_${contract}.wasm" \
    --wasm-out "target/wasm32-unknown-unknown/release/flowfi_${contract}.optimized.wasm"
  echo "   ✓ flowfi_${contract}.wasm optimized"
done

echo ""

# -----------------------------------------------------------------------
# Deploy contracts in dependency order
# -----------------------------------------------------------------------

echo "📦 Deploying Access Control..."
ACCESS_CONTROL_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/flowfi_access_control.optimized.wasm \
  --source deployer \
  --network "$SOROBAN_NETWORK")
echo "   ✓ Access Control: $ACCESS_CONTROL_ID"

echo "📦 Deploying Vault..."
VAULT_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/flowfi_vault.optimized.wasm \
  --source deployer \
  --network "$SOROBAN_NETWORK")
echo "   ✓ Vault: $VAULT_ID"

echo "📦 Deploying Rewards Engine..."
REWARDS_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/flowfi_rewards.optimized.wasm \
  --source deployer \
  --network "$SOROBAN_NETWORK")
echo "   ✓ Rewards: $REWARDS_ID"

echo "📦 Deploying Strategy Router..."
STRATEGY_ROUTER_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/flowfi_strategy_router.optimized.wasm \
  --source deployer \
  --network "$SOROBAN_NETWORK")
echo "   ✓ Strategy Router: $STRATEGY_ROUTER_ID"

echo ""

# -----------------------------------------------------------------------
# Initialize contracts
# -----------------------------------------------------------------------

echo "🔧 Initializing contracts..."

# Initialize Access Control
stellar contract invoke \
  --id "$ACCESS_CONTROL_ID" \
  --source deployer \
  --network "$SOROBAN_NETWORK" \
  -- initialize \
  --admin "$DEPLOYER_PUBLIC"
echo "   ✓ Access Control initialized"

# Initialize Vault
stellar contract invoke \
  --id "$VAULT_ID" \
  --source deployer \
  --network "$SOROBAN_NETWORK" \
  -- initialize \
  --admin "$DEPLOYER_PUBLIC" \
  --token "$REWARD_TOKEN_ADDRESS"
echo "   ✓ Vault initialized"

# Initialize Rewards Engine (reward rate: 1000 tokens per ledger, scaled by 1e9)
stellar contract invoke \
  --id "$REWARDS_ID" \
  --source deployer \
  --network "$SOROBAN_NETWORK" \
  -- initialize \
  --admin "$DEPLOYER_PUBLIC" \
  --reward_token "$REWARD_TOKEN_ADDRESS" \
  --vault "$VAULT_ID" \
  --reward_rate 1000000000000
echo "   ✓ Rewards Engine initialized"

# Initialize Strategy Router
stellar contract invoke \
  --id "$STRATEGY_ROUTER_ID" \
  --source deployer \
  --network "$SOROBAN_NETWORK" \
  -- initialize \
  --admin "$DEPLOYER_PUBLIC" \
  --strategist "$DEPLOYER_PUBLIC" \
  --vault "$VAULT_ID"
echo "   ✓ Strategy Router initialized"

echo ""

# -----------------------------------------------------------------------
# Save deployed addresses
# -----------------------------------------------------------------------

cat > .deployed.json << EOF
{
  "network": "$SOROBAN_NETWORK",
  "deployedAt": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "deployer": "$DEPLOYER_PUBLIC",
  "contracts": {
    "accessControl": "$ACCESS_CONTROL_ID",
    "vault": "$VAULT_ID",
    "rewards": "$REWARDS_ID",
    "strategyRouter": "$STRATEGY_ROUTER_ID"
  }
}
EOF

echo "📄 Addresses saved to .deployed.json"
echo ""
echo "✅ FlowFi Protocol deployed successfully!"
echo ""
echo "Contract Addresses:"
echo "  Access Control:  $ACCESS_CONTROL_ID"
echo "  Vault:           $VAULT_ID"
echo "  Rewards Engine:  $REWARDS_ID"
echo "  Strategy Router: $STRATEGY_ROUTER_ID"
echo ""
echo "Next steps:"
echo "  1. Fund the Rewards contract with reward tokens"
echo "  2. Set a strategist: stellar contract invoke --id $ACCESS_CONTROL_ID -- set_strategist --strategist <ADDRESS>"
echo "  3. Try a deposit via the SDK or CLI"
