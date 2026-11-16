# Interlock: Product Requirements Document

Version: 0.1 (draft) | Date: 2026-11-16 | Owner: Ademola (solo developer)
Target event: Solana "Perps and Prediction Markets" hackathon. Official deadline 2026-12-05 (timezone [UNVERIFIED]), so the internal submission deadline is 2026-12-04.

> **Roadmap rule:** any contribution to this repo (code, tests, docs, config, scripts) must update `ROADMAP.md` in the same change. See "Update protocol" in `ROADMAP.md`.

**Convention:** `[UNVERIFIED]` marks a claim nobody has confirmed against a primary source. Each one has a task in `ROADMAP.md`. When resolved, replace it with `[VERIFIED: source, date]`.

---

## 1. What we are building

Interlock is an open-source, non-custodial toolkit for people who trade leveraged perpetuals on Solana. An interlock is a mechanical switch that prevents a system from firing if operating parameters are out of bounds. The name conveys atomic transaction reversion (`assert_oracle_sane` via `interlock-ix`) without sounding like a custodial vault. It does two jobs: show what a position really costs to hold (`interlock cost`), and stop a user's own transaction from executing against a broken oracle price (`interlock-ix`).

Deliverables:

1. **Cost engine** (Rust library `interlock-core` and CLI `interlock cost`). Given a market, side, collateral, leverage and holding period, it reads live on-chain state and returns an itemised cost: open and close fees, estimated price impact, borrow fees over the holding period, break-even move, and liquidation distance.
2. **Snapshot logger and backtester.** The logger (`interlock snapshot run`) records market state to a local database on a schedule. The backtester (`interlock backtest`) replays that history to show how borrow cost actually varied over past holding windows, compared with the cost implied by today's rate.
3. **Oracle guard / interlock instruction** (`interlock-ix`: small Anchor program deployed to Solana mainnet). A stateless instruction (`assert_oracle_sane`) placed before a trade instruction in the same transaction. It reverts the whole transaction if the oracle price is stale, too uncertain, or deviates from an independent reference price beyond limits the caller sets.
4. **Replayable demo.** One command forks mainnet with Surfpool, injects a synthetic oracle spike, and shows the unguarded transaction proceeding while the guarded one reverts.
5. **Read-only web page** that presents the cost breakdown and the demo result.

### What the guard protects, and what it does not

This boundary is the most important sentence in the product.

- **Protects:** transactions the user (or their bot) signs, such as open, increase, decrease, close, collateral change and swap, from executing at an anomalous oracle price.
- **Does not protect:** liquidations executed by third-party keepers, venue insolvency, admin-key compromise, or any transaction the user submits without the guard. A pre-trade check inside the user's own transaction cannot stop a keeper from liquidating them.

Any README, pitch, video or UI copy that implies Interlock prevents liquidations is wrong and must be corrected.

## 2. Problem

Evidence below comes from a Gemini Deep Research brief (Nov 2026) and has not been independently verified by us. See `ROADMAP.md` tasks tagged `VERIFY`.

- **Borrow fees are quiet and compound over time.** Perp venues advertise open and close fees but charge an hourly borrow fee on top. Arithmetic check: 0.001% per hour on a $1,000 position is $1.68 per week ($1,000 x 0.00001 x 168 hours) [VERIFIED: arithmetic, 2026-11-16]. Whether Jupiter Perps' fee mechanics match this simple model is [UNVERIFIED].
- **Oracle anomalies hurt people.** The brief cites reports of a freak price spike affecting a user on a different venue, and of single-oracle designs. The cited forum threads may predate 2026 and the incident details are [UNVERIFIED].
- **Existing tools** (portfolio trackers and venue UIs) show nominal fees and position state. Whether any shows a multi-day carry projection is [UNVERIFIED].

Honest framing: this product is a bet that systematic traders and integrators value pre-trade cost transparency and a composable price-sanity check. That demand is unproven.

## 3. Users

| ID | User | Needs | Notes |
|----|------|-------|-------|
| U1 | Systematic perp trader (runs scripts or bots on Solana) | True carry cost of a position before entering; a guard instruction their bot can prepend to its own transactions | Primary |
| U2 | Integrator (builds vaults, bots, wallets) | A small guard program with a published IDL they can compose | Secondary |
| E1 | Hackathon judge | A story they can verify in about three minutes: live program, reproducible demo, working numbers | Constraint on the demo, not a persona to design features for |

Explicitly not users: retail beginners, anyone expecting advice, anyone expecting custody.

## 4. Goals and non-goals

### Goals

- G1: Cost numbers that match the venue's own quote on at least five test positions (tolerance defined in F1).
- G2: A guard program live on mainnet with every error path tested.
- G3: A backtest built on at least 7 days of our own logged snapshots, with coverage reported honestly.
- G4: A demo that reproduces from a clean checkout in under 15 minutes.
- G5: A complete hackathon submission by 2026-12-04.

### Non-goals

- Custody, deposits, order matching, or holding any user key.
- Investment advice, trade signals, or automated trading.
- Multi-venue support in v1. One venue only (Jupiter Perps, pending gate G1).
- Prediction-market features. This project addresses the perps half of the hackathon only.
- User traction. We have no audience and we are not pretending otherwise.
- A token, accounts, auth, notifications, or mobile apps.

## 5. Requirements

Priority: P0 must ship, P1 should ship, P2 stretch.

### F1. Cost engine (P0)

**Input:** market, side, collateral (USD), leverage, holding horizon (hours).
**Output:** `CostBreakdown` (see `ARCHITECTURE.md` section 5): size, open fee, close fee, price impact (or "unavailable" with reason), borrow cost over the horizon, total cost in USD and as a percent of collateral, break-even price move, liquidation price and distance, and a sweep across 1h, 24h, 72h and 168h.

Acceptance criteria:
- `interlock cost --market SOL --side long --collateral 100 --leverage 5 --hours 72` prints a breakdown in under 3 seconds on a normal connection.
- Output declares the slot it was computed at and lists every assumption and every [UNVERIFIED] input used.
- Golden tests: for at least 5 positions, the engine's total cost is within 1% relative (or $0.05 absolute) of the venue's own simulated quote. If the venue offers no quote, we compare against fee schedules and document the gap.
- If any required venue parameter cannot be decoded, the command fails loudly instead of guessing.

### F2. Snapshot logger and backtester (logger P0, backtester P1)

Acceptance criteria:
- `interlock snapshot run` records market state every N seconds (default 60) to SQLite, tolerates RPC failures, and never writes partial rows.
- `interlock snapshot report` prints coverage per day and lists gaps longer than 5 minutes.
- `interlock backtest ...` reports min, median, p95 and max realised borrow cost across rolling windows, next to the cost implied by the entry-time rate.
- The backtester refuses to produce a result when coverage in the requested window is below 90%.

### F3. Oracle guard program (P0)

Instruction `assert_oracle_sane` (stateless, no signer, no custody, no stored accounts).

Acceptance criteria:
- Rejects: stale price, confidence interval too wide, wrong feed id, insufficient verification level, reference source invalid, deviation above the limit, and arithmetic overflow. Each has its own error code.
- Passes silently (one log line) when all checks hold.
- LiteSVM tests cover every error path plus a property test on the deviation math.
- Deployed to mainnet; one real mainnet transaction invoking it is linked in the README.
- Compute units and transaction size overhead are measured and recorded.

### F4. Replayable demo (P0)

Acceptance criteria:
- One command (`just demo`) forks mainnet via Surfpool, runs scenario A (normal conditions: guarded transaction passes) and scenario B (synthetic oracle spike: guarded transaction reverts, unguarded proceeds).
- Scenario B is labelled "synthetic" in output and in the video.
- Documented in README with expected output.

### F5. Web page (P1)

Acceptance criteria:
- Single page: cost form, breakdown table, horizon chart, demo result panel, disclaimer.
- Read-only. No wallet connection.
- Works from precomputed JSON if the API is cut.

### F6. On-chain utilization check (P2, stretch)

Reverts if the target venue's pool utilization exceeds a limit. Only attempted if F1 to F4 are done, because it depends on decoding venue account layouts inside the program.

## 6. User stories

- U1: "Before I open 5x SOL long for three days, I want to see what the borrow fee will cost me at today's rate and across past windows."
- U1: "My bot signs its own transactions. I want to prepend one instruction that aborts if the oracle looks wrong."
- U2: "I want an IDL and a documented account list so I can add the guard to my vault's rebalance transaction."
- E1: "I want to run one command and see the guard block a bad price."

## 7. Non-functional requirements

- **Security:** no private keys read or stored by any tool except a dedicated deploy keypair used manually; guard program holds no funds and no state.
- **Reproducibility:** pinned toolchain versions recorded in `ARCHITECTURE.md`; fixtures committed for offline test runs.
- **Honesty:** synthetic data is always labelled; unverified claims are always marked.
- **Legal:** the tool is informational software, not advice, and does not touch custody or matching. The legal position of building and distributing it from Nigeria [UNVERIFIED] must be checked by a qualified person before any public launch beyond the hackathon.
- **Licensing:** MIT proposed, decision pending (task P0-03).

## 8. Success criteria

| ID | Criterion | Measure |
|----|-----------|---------|
| S1 | Guard live on mainnet and invoked | Program id and tx signature in README |
| S2 | Cost engine validated | Golden test report, at least 5 positions |
| S3 | Backtest on own data | At least 7 days of snapshots, coverage report published |
| S4 | Demo reproducible | Clean-machine run under 15 minutes |
| S5 | Submission complete | Form submitted by 2026-12-04 |
| S6 | No custody | Code review checklist passes |

Traction (users, volume) is explicitly not a success criterion.

## 9. Assumptions and open questions

| # | Item | Status |
|---|------|--------|
| A1 | Official start date, rules, FAQ, submission format | [UNVERIFIED], task P0-02 |
| A2 | Main track judging criteria | Unpublished at time of writing |
| A3 | Nigerian residents can receive prizes | [UNVERIFIED], task P0-02 |
| A4 | Jupiter Perps fill model (atomic versus request-then-keeper) | [UNVERIFIED], gate G1 |
| A5 | Venue fee and borrow-rate mechanics match the simple model | [UNVERIFIED], gate G3 |
| A6 | Free RPC plan lasts one week from hackathon start | Per hackathon page; plan a fallback |
| A7 | Submission requires public repo and demo video | Assumed from the sponsor bounty text; [UNVERIFIED] for main track |

## 10. Hard questions

### 10.1 What would break?

1. **The core value claim may not hold.** A guard inside the user's own transaction cannot stop keeper liquidations, which is the pain the research brief emphasises. What remains is protection for user-initiated transactions at bad oracle prices, a narrower claim. Mitigation: state the boundary everywhere (section 1) and lead the pitch with cost transparency, not liquidation prevention.
2. **Venue fill model (gate G1).** If the venue fills orders later through keepers, the guard sees a price the fill never uses, and the guard has nothing to protect for that venue. Mitigation: Day-1 check; fall back to a venue or transaction type that settles atomically, or ship the cost engine with an honestly scoped guard.
3. **Wrong numbers.** If the cost model is off, credibility is gone. Mitigation: golden tests against venue quotes; fail loudly on missing parameters.
4. **Thin data.** A backtest on a few days of one market proves little. Mitigation: start the logger by Nov 20, report coverage, state sample size on every result.
5. **Demo flakiness.** Live RPC can fail while recording. Mitigation: committed fixtures and a recorded fallback.
6. **Mainnet deployment problems.** Cost, buffer handling and first-time Anchor mistakes. Mitigation: devnet rehearsal first; tiny program; deploy approval step.
7. **Time.** 19 days, solo, first Anchor project. Mitigation: cut list in section 11; P2 only after P0 and P1.
8. **Prize eligibility** may be restricted for Nigerian residents [UNVERIFIED]. Mitigation: ask on Day 1 in the hackathon Discord.

### 10.2 What edge cases are we missing?

- A user with collateral in a different asset than the position, or cross-collateral setups.
- Promotional zero-fee periods or fee tiers that change quotes.
- Positions below the venue's minimum size, or leverage above its maximum.
- Markets that are closed or paused, including non-crypto markets outside trading hours.
- A user who runs the guard on a plain swap (works; should not be blocked or advertised beyond scope).
- A user who reads cost output as a prediction or as advice. Requires a visible disclaimer.
- A venue upgrade that changes account layouts between our test and the judging window.
- A stablecoin depeg that changes effective collateral value.

### 10.3 What is overengineered?

- Three personas. Only U1 drives requirements.
- Wallet connection. A read-only page is enough for the demo.
- Alerts and notifications.
- Multi-venue abstraction. One venue, concrete code.
- Monitoring of admin or governance changes. Cut.
- This documentation set is large for a three-week solo project. `ARCHITECTURE_ESSENTIALS.md` exists so agents do not need to read everything. Keep the docs lean after the first pass.

## 11. Cut list (in order of cutting)

1. F6 on-chain utilization check
2. Web API (use precomputed JSON)
3. Chart polish
4. Backtester percentiles beyond min, median and max
5. Web page entirely (CLI plus video)

Never cut: F1, F3, F4, the mainnet deployment, and the honesty rules.

---

> **Roadmap rule:** any contribution to this repo must update `ROADMAP.md` in the same change.
