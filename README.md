# Interlock

A non-custodial Rust toolkit and atomic oracle guard for Solana perpetuals:
- **Cost Engine (`interlock cost`):** Itemises true holding costs (open, close, borrow fees over time, price impact, break-even move percentage, liquidation distance).
- **Snapshot Logger & Backtester (`interlock snapshot`, `interlock backtest`):** Records market state to SQLite and tests realized carry variance against entry rates with strict coverage guarantees.
- **Oracle Guard Program (`interlock-ix`):** Stateless on-chain instruction (`assert_oracle_sane`) that atomically reverts transactions if oracle prices are stale, confidence is wide, or deviation from an independent reference exceeds limits.
- **Surfpool Demo:** Replayable simulation demonstrating atomic reversion during synthetic oracle spikes.
- **Web Interface:** Read-only React + TypeScript viewer for position costs and demo outcomes.

An interlock is a mechanical switch that prevents a system from firing if operating parameters are out of bounds. The name conveys atomic transaction reversion (`assert_oracle_sane`) without sounding like a custodial vault.

---

## Critical Boundary

**Interlock protects transactions signed by the user or their trading bot** (e.g., opening, closing, sizing, or swapping positions).

> **Important:** It does not protect against third-party keeper liquidations on existing open positions, venue insolvency, or transactions executed without the guard instruction. Pre-trade checks inside user transactions cannot prevent a keeper from liquidating an existing position.

---

## Project Structure

```
crates/
  interlock-core/   Pure types, fixed-point cost math, liquidation models, backtest logic
  interlock-chain/  Solana RPC client wrapper, slot-consistent reads, decoders, transaction composer
  interlock-store/  SQLite snapshot schema, unattended logger, coverage and gap queries
  interlock-cli/    CLI binary (cost, snapshot, backtest, guard simulate)
  interlock-api/    HTTP API service routes (/health, /v1/cost, /v1/backtest, /v1/scenario/:id)
programs/
  interlock-ix/     Anchor program with stateless assert_oracle_sane instruction
scenarios/          Surfpool scenario A (normal pass) and scenario B (synthetic spike revert)
web/                Read-only React + TypeScript interface
scripts/            Toolchain installation and deployment scripts
docs/
  demo/             Walkthrough script and pitch one-pager
  verification/     Verification logs for gates G1-G3, hackathon rules, and security reviews
```

---

## Quickstart

### Prerequisites
- Rust 1.80+ (`cargo`)
- Node.js 18+ (for web interface)
- `just` command runner (optional, or use cargo directly)

### Run Tests and Format Checks
```bash
# Run all workspace unit tests and property tests
cargo test --workspace

# Or using just
just check
```

### Run Cost Engine
```bash
# Calculate itemised cost for a $1,000 collateral, 5x leverage long on SOL held for 24 hours
cargo run -p interlock-cli -- cost --market SOL --side long --collateral 1000 --leverage 5 --hours 24

# Output pure JSON
cargo run -p interlock-cli -- cost --market SOL --side long --collateral 1000 --leverage 5 --hours 24 --json
```

### Run Replayable Demo
```bash
# Run both Scenario A (normal pass) and Scenario B (synthetic oracle spike revert)
bash scripts/demo.sh

# Or using just
just demo
```

### Run Web Client
```bash
cd web
npm install
npm run build
npm run preview
```

---

## Transaction Composition & Instruction Order

Transactions using Interlock must follow this strict instruction ordering:
```
[0] ComputeBudget::set_compute_unit_limit(200_000)
[1] (Optional) Pyth::price_update_v2
[2] InterlockIx::assert_oracle_sane
[3] Venue::open_position (or trade instruction)
```

The `TransactionComposer` in `interlock-chain` enforces this ordering at build time, refusing to build transactions where trade instructions precede the guard.

---

## Security & Verification Checklist

1. **Non-custodial:** No private keys stored, no custody of user funds, no trade execution authority.
2. **Deterministic Arithmetic:** `rust_decimal::Decimal` off-chain; checked `u128` integer cross-multiplication on-chain. Zero floats in financial math.
3. **Stateless Program:** Zero account creation, zero lamports moved, zero CPI. Program overhead is under 2,000 compute units.
4. **Coverage Refusal:** The backtester refuses evaluations when historical data coverage is below 90%.
5. **Synthetic Data Disclosure:** All artificial test vectors are clearly designated as `synthetic`.
6. **Disclosed Unverified Markers:**
   - `[UNVERIFIED: gate G3]`: Exact venue borrow rate compounding frequency (currently modeled as hourly flat accrual).
   - `[UNVERIFIED: gate G3]`: Maintenance margin exact venue curve (currently modeled using standard base maintenance margin).

---

## License

Licensed under the Apache License, Version 2.0 (see [LICENSE](file:///home/dell/interlock/LICENSE)).
