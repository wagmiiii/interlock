# Interlock: Architecture Essentials

Short reference for coding agents. Full detail lives in `ARCHITECTURE.md`. If this file and `ARCHITECTURE.md` disagree, fix both in the same change.

> **Roadmap rule:** any contribution to this repo must update `ROADMAP.md` in the same change.

## What it is
- Non-custodial Rust toolkit for Solana perps: cost engine (`interlock cost`), snapshot logger and backtester, stateless Anchor oracle interlock program (`interlock-ix`, instruction `assert_oracle_sane`), Surfpool demo, read-only web page.
- An interlock is a mechanical switch that prevents a system from firing if operating parameters are out of bounds. Conveys atomic transaction reversion (`assert_oracle_sane`) without sounding like a custodial vault.
- Guard protects the user's own signed transactions only. It cannot stop keeper liquidations. Never claim otherwise.

## Invariants (never break)
1. No custody, no key handling, no order placement.
2. Guard program: stateless, no accounts of its own, integers only, no floats.
3. Off-chain money math uses `rust_decimal`. No `f64` for money.
4. Decoders fail loudly on unknown layout. Never guess a venue parameter.
5. Synthetic data is always labelled synthetic.
6. Unverified claims carry `[UNVERIFIED]` until a task resolves them.
7. Every change updates `ROADMAP.md`.

## Components
- `interlock-core`: pure types and math (no IO).
- `interlock-chain`: RPC reads and decoders (read-only).
- `interlock-store`: SQLite snapshots.
- `interlock-cli`: `cost`, `snapshot run|report`, `backtest`, `guard simulate`.
- `interlock-api`: optional axum layer.
- `programs/interlock-ix`: Anchor program, one instruction `assert_oracle_sane`.
- `web/`: static, read-only.
- `scenarios/`: Surfpool demo scenarios A (normal) and B (synthetic spike).

Dependency direction: cli/api -> chain, store -> core. Core depends on nothing.

## Key decisions
- ADR-002: guard limits are instruction arguments, no config PDA.
- ADR-003: one venue, concrete code, no trait layer.
- ADR-004: pre-trade instruction in the same transaction. PENDING gate G1.
- ADR-005: reference price source. PENDING gate G2.
- ADR-006: Decimal off-chain, basis-point integers on-chain.
- ADR-007: SQLite for snapshots.
- ADR-008: Pyth via official receiver SDK, require full verification.
- ADR-010: on-chain utilization check deferred (P2).
- ADR-011: upgrade authority decision PENDING.

## Open gates
- G1: does the venue fill atomically or via keepers? Decides guard mode.
- G2: which reference price source, and is its layout decodable?
- G3: how are venue fees and borrow rate represented, and does the flat-rate model match?

## Core formulas
- `borrow_usd = size_usd * borrow_rate_per_hour * hours`
- `size_usd = collateral_usd * leverage`
- Deviation check: `|p_oracle - p_ref| * 10_000 > max_deviation_bps * p_ref` (u128, no pre-division)

## Transaction order
`[ComputeBudget] [pull-oracle update if needed] [assert_oracle_sane] [venue instruction]`

## Testing
- Core: unit plus proptest. Program: LiteSVM, every error path. Integration: Surfpool. Cost engine: golden tests against venue quotes.

## Do not build
Config PDA, multi-venue layer, WASM, wallet connect, alerts, governance monitor, DuckDB, extra scenarios, browser E2E tests.

## Top risks
1. Venue uses keeper fills, guard has nothing to protect (G1).
2. Venue account layout changes break decoders.
3. Cost numbers wrong, credibility lost.
4. Transaction size or compute overhead too high.
5. Time: 19 days, solo, first Anchor project.

## Where to look
- Task list and status: `ROADMAP.md`
- Product scope and acceptance criteria: `PRD.md`
- File placement: `project_structure.md`
- Working rules: `AGENTS.md`
