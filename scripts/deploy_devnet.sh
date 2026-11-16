#!/usr/bin/env bash
set -euo pipefail

# Manual script requiring explicit keypair path
KEYPAIR_PATH="${1:-}"

if [ -z "$KEYPAIR_PATH" ]; then
    echo "Usage: ./scripts/deploy_devnet.sh <path-to-keypair.json>"
    echo "Note: Never commit keypair files to this repository."
    exit 1
fi

echo "Deploying interlock-ix to Solana Devnet using keypair: $KEYPAIR_PATH"
# anchor deploy --provider.cluster devnet --provider.wallet "$KEYPAIR_PATH"
