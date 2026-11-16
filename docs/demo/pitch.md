# Interlock: Pitch One-Pager

## The Mechanical Switch for Solana Perpetuals

**Interlock** is a non-custodial Rust toolkit and stateless Solana on-chain instruction (`assert_oracle_sane`) that prevents catastrophic trade execution by enforcing pre-trade oracle sanity checks atomically.

---

### The Concrete Problem It Solves

On November 16, 2026, an arbitrage bot or trader issues a signed transaction to open a 10x leveraged long on `SOL-PERP` with $10,000 collateral ($100,000 position size) when the true market price is $150.00.
During a microsecond oracle publisher glitch or network packet reordering, the venue's pull-oracle price updates to $172.50 (+15.0% spike).

#### Without Interlock:
1. The transaction executes at the mark price of $172.50.
2. The user receives an immediate fill at a 15% premium.
3. As the next slot's true oracle price settles back to $150.00, the position instantly suffers an unrealized loss of:
   $$\text{Loss} = \$100,000 \times 15\% = \$15,000$$
4. Because the user posted $10,000 collateral, the position is instantly wiped out and liquidated in the subsequent slot.

#### With Interlock:
1. The user's transaction includes `interlock-ix::assert_oracle_sane` prepended before the venue's `open_position` instruction.
2. The instruction checks the Pyth price ($172.50) against an independent reference source ($150.00) using integer `u128` cross-multiplication.
3. The measured deviation is 1,500 bps, exceeding the trader's 50 bps limit.
4. The instruction immediately returns `GuardError::DeviationExceeded`.
5. **Atomic Guarantee:** Solana runtime aborts the entire transaction. The venue trade instruction is **never executed**.
6. **Result:** $10,000 in collateral is preserved intact. Total cost: ~5,000 lamports ($0.0008) in base transaction fee.

---

### Key Capabilities

1. **Deterministic Cost Engine (`interlock cost`):**
   - Precise itemisation of entry/exit fees, holding carry over time, price impact, and breakeven move percentages.
   - 168-hour horizon sweeps and rate sensitivity tables (0.5x, 1x, 2x borrow rates).
   - Fixed-point `rust_decimal::Decimal` eliminates floating point rounding errors.

2. **Snapshot Logger & Backtester (`interlock snapshot`, `interlock backtest`):**
   - Unattended market state recorder logging to SQLite with coverage calculation and gap reporting.
   - Refusal rule: Refuses backtests when historical coverage is below 90%.

3. **Atomic Oracle Guard (`interlock-ix`):**
   - Stateless Anchor program with zero account allocation, zero lamport transfers, and zero CPI.
   - Enforces max age staleness, confidence interval bounds, feed id verification, and reference divergence.
   - Under 2,000 compute units overhead; negligible impact on transaction gas budgets.

---

### Critical Boundary & Non-Custodial Guarantee

> **What Interlock Protects:** Transactions signed by the user or their bot (order creation, sizing, closure).
>
> **What Interlock Cannot Protect:** Third-party keeper liquidations on existing open positions. Pre-trade instruction guards only evaluate parameters inside the user's signed transaction; they do not intervene in liquidations performed by external liquidators.
>
> **Non-Custodial:** Interlock never holds private keys, never holds collateral, and has no admin keys or upgrade backdoors.

---

### Hackathon Track Alignment: Solana Perps and Prediction Markets
- **Code:** Rust, Anchor, Solana CLI, LiteSVM, React, TypeScript.
- **License:** Apache-2.0.
- **Demo:** Reproducible offline and fork-replayable via `just demo`.
