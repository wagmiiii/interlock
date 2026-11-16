# Interlock

A non-custodial Rust toolkit for Solana perpetuals:
- **Cost Engine (`interlock cost`):** itemises true holding costs (open, close, borrow fees over time, price impact, break-even move, liquidation distance).
- **Snapshot Logger and Backtester (`interlock snapshot`, `interlock backtest`):** records market state to SQLite and tests realized carry variance against entry rates.
- **Oracle Guard Program (`interlock-ix`):** stateless on-chain instruction (`assert_oracle_sane`) that atomically reverts transactions if oracle prices are stale, confidence is wide, or deviation from a reference exceeds limits.
- **Surfpool Demo:** mainnet-fork test demonstrating atomic reversion during synthetic oracle spikes.
- **Web Interface:** read-only viewer for position costs and demo outcomes.

An interlock is a mechanical switch that prevents a system from firing if operating parameters are out of bounds. The name conveys atomic transaction reversion (`assert_oracle_sane`) without sounding like a custodial vault.

## Critical Boundary

Interlock protects transactions signed by the user or their trading bot (e.g., open, close, size change, swap).

**It does not protect against third-party keeper liquidations, venue insolvency, or transactions executed without the guard instruction.** Pre-trade checks inside user transactions cannot prevent a keeper from liquidating an existing position.

## Project Structure

```
crates/
  interlock-core/   Pure types, cost math, liquidation models, backtest logic
  interlock-chain/  Solana RPC client wrapper, slot-consistent reads, account decoders
  interlock-store/  SQLite snapshot schema, unattended logger, coverage queries
  interlock-cli/    CLI binary (cost, snapshot run, snapshot report, backtest, guard simulate)
  interlock-api/    Axum HTTP API service
programs/
  interlock-ix/     Anchor program with stateless assert_oracle_sane instruction
scenarios/          Surfpool scenario A (normal pass) and scenario B (synthetic spike revert)
web/                Read-only React + TypeScript interface
scripts/            Toolchain installation and deployment scripts
```

## Quickstart

```bash
# Run all format checks, clippy, and workspace unit tests
just check

# Run workspace tests only
just test

# Calculate position cost
cargo run -p interlock-cli -- cost --market SOL --side long --collateral 100 --leverage 5 --hours 72
```

## Security & Invariants

1. **Non-custodial:** No private keys stored, no custody of user funds, no trade execution authority.
2. **Deterministic Math:** Integer and fixed-point `rust_decimal::Decimal` arithmetic off-chain; checked integer math (`u128`) on-chain. No floating point operations for monetary calculations.
3. **Transparent Assumptions:** All estimates, assumptions, and unverified parameters are declared in output.
4. **Synthetic Data Labeling:** All synthetic simulation test data is explicitly marked as synthetic.

## License

Licensed under the Apache License, Version 2.0 (see [LICENSE](file:///home/dell/interlock/LICENSE)).
