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

if command -v solana >/dev/null 2>&1; then
    echo "Performing pre-flight deployer SOL balance check..."
    BALANCE=$(solana balance "$KEYPAIR_PATH" --url mainnet-beta 2>/dev/null | awk '{print $1}' || echo "0")
    if [ -n "$BALANCE" ] && [ "$BALANCE" != "0" ]; then
        echo "Deployer balance: $BALANCE SOL"
        awk -v b="$BALANCE" 'BEGIN {if (b < 1.5) {print "Error: Insufficient deployer SOL (" b " SOL). Minimum 1.5 SOL required to fund program buffer."; exit 1}}'
    fi
fi

echo "Deploying to Solana Mainnet-Beta..."
# anchor deploy --provider.cluster mainnet --provider.wallet "$KEYPAIR_PATH"
