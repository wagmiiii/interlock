#!/usr/bin/env bash
set -euo pipefail

echo "========================================================"
echo "          Interlock Replayable Demo (Surfpool)          "
echo "========================================================"
echo ""
echo "Running Scenario A (Normal Market Conditions)..."
cargo run -q -p interlock-cli -- guard --scenario normal
echo ""
echo "Running Scenario B (Synthetic Oracle Spike)..."
cargo run -q -p interlock-cli -- guard --scenario spike
echo ""
echo "Demo complete."
