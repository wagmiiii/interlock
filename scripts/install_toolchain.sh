#!/usr/bin/env bash
set -euo pipefail

echo "=== Interlock Toolchain Installation ==="
echo "Checking rustc..."
rustc --version
echo "Checking cargo..."
cargo --version

if ! command -v just &>/dev/null; then
    echo "Installing just..."
    cargo install just --locked
else
    echo "just is installed: $(just --version)"
fi

echo "Toolchain check complete."
