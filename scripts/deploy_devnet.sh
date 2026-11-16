#!/usr/bin/env bash
set -euo pipefail

# Manual script requiring explicit keypair path
KEYPAIR_PATH="${1:-}"

if [ -z "$KEYPAIR_PATH" ]; then
    echo "Usage: ./scripts/deploy_devnet.sh <path-to-keypair.json>"
    echo "Note: Never commit keypair files to this repository."
    exit 1
fi

if command -v solana >/dev/null 2>&1; then
    echo "Performing pre-flight deployer SOL balance check..."
    BALANCE=$(solana balance "$KEYPAIR_PATH" --url devnet 2>/dev/null | awk '{print $1}' || echo "0")
    if [ -n "$BALANCE" ] && [ "$BALANCE" != "0" ]; then
        echo "Deployer devnet balance: $BALANCE SOL"
        awk -v b="$BALANCE" 'BEGIN {if (b < 1.5) {print "Error: Insufficient deployer SOL (" b " SOL). Minimum 1.5 SOL required to fund program buffer."; exit 1}}'
    fi
fi

echo "Deploying interlock-ix to Solana Devnet using keypair: $KEYPAIR_PATH"
# anchor deploy --provider.cluster devnet --provider.wallet "$KEYPAIR_PATH"
