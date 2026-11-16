#!/usr/bin/env bash
set -euo pipefail

# Interlock Pre-Commit Secret Scanner (Task P0-12)
# Ensures no private keys, secrets, or API keys are committed to repository.

echo "Running pre-commit secret scan..."

# Check staged file names
FORBIDDEN_FILES=$(git diff --cached --name-only | grep -E '(\.keypair$|id\.json$|deploy-keypair.*\.json$|\.env$)' || true)
if [ -n "$FORBIDDEN_FILES" ]; then
    echo "ERROR: Attempted to commit secret or private key file(s):"
    echo "$FORBIDDEN_FILES"
    echo "Aborting commit per AGENTS.md security invariants."
    exit 1
fi

# Patterns that indicate secret keys or private key content
TEST_PATTERN="FAKE_PRIVATE_KEY""_TEST_STRING"
SUSPICIOUS_PATTERNS=(
    "BEGIN (RSA |EC |OPENSSH |DSA )?PRIVATE KEY"
    "\[[0-9]{1,3},[0-9]{1,3},[0-9]{1,3},[0-9]{1,3},[0-9]{1,3},"
    "PRIVATE_KEY\s*=\s*['\"][a-zA-Z0-9_\-]{20,}['\"]"
    "SECRET_KEY\s*=\s*['\"][a-zA-Z0-9_\-]{20,}['\"]"
    "$TEST_PATTERN"
    "ghp_[a-zA-Z0-9]{30,}"
)

# Search staged diffs for forbidden patterns, excluding this scan script itself
DIFF_CONTENT=$(git diff --cached -U0 -- ':(exclude)scripts/pre_commit_secret_scan.sh')

for PATTERN in "${SUSPICIOUS_PATTERNS[@]}"; do
    MATCHES=$(echo "$DIFF_CONTENT" | grep -E "^\+" | grep -E "$PATTERN" || true)
    if [ -n "$MATCHES" ]; then
        echo "ERROR: Suspicious secret pattern detected in staged diff:"
        echo "Pattern: $PATTERN"
        echo "$MATCHES"
        echo "Aborting commit per AGENTS.md security invariants."
        exit 1
    fi
done

echo "Secret scan passed cleanly."
exit 0
