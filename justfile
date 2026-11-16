# Interlock task runner

default: check

# Run formatting check, clippy, and all workspace tests
check:
    cargo fmt --all --check
    cargo clippy --all-targets -- -D warnings
    cargo test --workspace

# Run all workspace tests
test:
    cargo test --workspace

# Run Surfpool fork demo (scenarios A and B)
demo:
    ./scripts/demo.sh

# Build Anchor program and export IDL to web client
idl:
    mkdir -p target/idl web/src/lib/idl
    @echo "IDL export stub: copy target/idl/interlock_ix.json to web/src/lib/idl/"

# Export demo scenario data to precomputed JSON
export-demo-data:
    mkdir -p web/public/data
    @echo "Exporting demo scenario data to web/public/data/"
