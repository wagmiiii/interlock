#!/usr/bin/env bash
set -euo pipefail

# HARD RULE: Never send mainnet transaction or deploy without explicit human approval in current session
echo "================================================================"
echo "                   MAINNET DEPLOYMENT SAFETY CHECK              "
echo "================================================================"
echo "This script deploys the Interlock program to Solana Mainnet-Beta."
echo "Hard rule from AGENTS.md: requires explicit human approval."
read -p "Type 'APPROVE_MAINNET_DEPLOY' to continue: " CONFIRMATION

if [ "$CONFIRMATION" != "APPROVE_MAINNET_DEPLOY" ]; then
    echo "Deployment aborted."
    exit 1
fi

KEYPAIR_PATH="${1:-}"
if [ -z "$KEYPAIR_PATH" ]; then
    echo "Usage: ./scripts/deploy_mainnet.sh <path-to-mainnet-deploy-keypair.json>"
    exit 1
fi

echo "Deploying to Solana Mainnet-Beta..."
# anchor deploy --provider.cluster mainnet --provider.wallet "$KEYPAIR_PATH"
