# Interlock: Architecture

Version: 0.1 (draft) | Date: 2026-11-16 | Companion docs: `PRD.md`, `ARCHITECTURE_ESSENTIALS.md`, `project_structure.md`, `ROADMAP.md`

> **Roadmap rule:** any contribution to this repo (code, tests, docs, config, scripts) must update `ROADMAP.md` in the same change. See "Update protocol" in `ROADMAP.md`.

**Convention:** `[UNVERIFIED]` marks a claim not yet confirmed against a primary source. Gates (G1 to G3) decide open architecture questions on Day 1 to 3. When a gate resolves, update the ADR table below and `ARCHITECTURE_ESSENTIALS.md`.

---

## 1. System context

Interlock is a non-custodial Rust toolkit for Solana perpetuals: a cost engine (`interlock cost`), a snapshot logger and backtester, a small stateless Anchor program (`interlock-ix`, instruction `assert_oracle_sane`), a Surfpool demo, and a read-only web page. An interlock is a mechanical switch that prevents a system from firing if operating parameters are out of bounds. The name conveys atomic transaction reversion (`assert_oracle_sane`) without sounding like a custodial vault.

```
                         read-only                          
  Solana mainnet  <------------------+                      
  (Jupiter Perps accounts,           |                      
   Pyth price updates,               |                      
   reference DEX pool)               |                      
          ^                          |                      
          | getMultipleAccounts      |                      
          |                          |                      
  +-------+--------+        +--------+---------+     +-------------+
  | interlock-chain|------>| interlock-core   |<----| interlock-  |
  | (RPC + decoders)|       | (types, cost     |     | store       |
  +----------------+        |  math, backtest) |     | (SQLite)    |
          ^                 +--------+---------+     +------^------+
          |                          |                      |
          |                 +--------v---------+            |
          |                 | interlock-cli    |-----------+
          |                 | (cost, snapshot,  |  snapshot run writes
          |                 |  backtest, guard) |
          |                 +--------+---------+
          |                          |
          |                 +--------v---------+      +-----------+
          +-----------------| interlock-api    |----->| web (Vite |
                            | (axum, optional) |      | React TS) |
                            +------------------+      +-----------+

  User tx:  [ ComputeBudget ] [ interlock::assert_oracle_sane ] [ venue instruction ]
                                      |
                       interlock-ix (Anchor, stateless, mainnet)

  Demo:  Surfpool mainnet fork  +  scenario files  ->  guarded vs unguarded replay
```

Nothing in this system holds a user key or user funds. The engine only reads. The guard only checks.

## 2. Components

| Component | Path | Responsibility | IO |
|-----------|------|----------------|----|
| core | `crates/interlock-core` | Types, cost model, liquidation model, backtest math. Pure functions, no network, no disk. | None |
| chain | `crates/interlock-chain` | RPC client wrapper, account decoders (venue, Pyth), slot-consistent reads | Network (read-only) |
| store | `crates/interlock-store` | SQLite schema, snapshot writer, coverage queries | Disk |
| cli | `crates/interlock-cli` | Commands: `cost`, `snapshot run`, `snapshot report`, `backtest`, `guard simulate` | Network, disk |
| api | `crates/interlock-api` | Thin axum wrapper over core and store (optional, cut-list item 2) | Network |
| guard | `programs/interlock-ix` | Anchor program with one instruction, no accounts of its own | On-chain |
| web | `web/` | Static single page, calls API or reads precomputed JSON | Browser |
| scenarios | `scenarios/` | Surfpool scenario files and fixtures for the demo | Test only |

Dependency direction: `cli`/`api` -> `chain`, `store` -> `core`. `core` depends on nothing in the workspace. The guard program depends on nothing in the workspace.

## 3. Decisions (ADR table)

| ID | Decision | Status | Alternatives rejected | Revisit if |
|----|----------|--------|-----------------------|------------|
| ADR-001 | Non-custodial, read-only engine, no key handling | Accepted | Order placement features | Never in v1 |
| ADR-002 | Guard is stateless: parameters passed as instruction arguments, no config account | Accepted | Per-user `GuardConfig` PDA | Users need persistent limits |
| ADR-003 | One venue in v1, concrete code (no venue trait) | Accepted | Multi-venue adapter layer | A second venue is funded |
| ADR-004 | Guard attachment mode: pre-trade instruction in the same transaction | Pending gate G1 | Advisory-only mode | G1 shows the venue fills via keepers and no atomic path exists |
| ADR-005 | Reference price source for deviation check | Pending gate G2 | Second Pyth feed, Switchboard, DEX pool spot | Layout decoding too brittle |
| ADR-006 | Money math in `rust_decimal` off-chain, integer basis points on-chain, no floats anywhere | Accepted | `f64` | Never |
| ADR-007 | SQLite for snapshots | Accepted | DuckDB, Parquet, Postgres | Data exceeds a few GB |
| ADR-008 | Pyth read through the official receiver SDK, never hand-parsed in the program | Accepted | Manual byte parsing | SDK incompatible with Anchor version |
| ADR-009 | Web is static and read-only, no wallet connection | Accepted | Wallet adapter, signing UI | Never in v1 |
| ADR-010 | On-chain utilization check deferred (P2) | Accepted | Ship in v1 | Gate G3 shows layout is stable and decodable |
| ADR-011 | Program upgrade authority: keep through judging, decide revoke or transfer after | Pending | Revoke at deploy | Judges require immutability |
| ADR-012 | Tests: LiteSVM for program unit tests, Surfpool for mainnet-fork integration | Accepted | Local validator only | LiteSVM cannot load needed accounts |

## 4. Tech stack

Versions are pinned on Day 1 (task P0-04) and recorded in the table below. Do not upgrade mid-project without a roadmap entry.

| Layer | Choice | Version | Notes |
|-------|--------|---------|-------|
| Language (off-chain) | Rust stable | 1.97.1 | Edition 2021 |
| Language (on-chain) | Rust / Anchor | 0.30.1 | Edition 2021, integer cross-multiplication |
| Solana tooling | Agave / LiteSVM | 2.1.0 / 0.4.0 | Offline fast SVM execution |
| Pyth access | pyth-solana-receiver-sdk | 0.1.2 | Require full verification level |
| Decimal math | `rust_decimal` | 1.36.0 | Strict fixed-point, no floats for money |
| Storage | SQLite via `rusqlite` | 0.32.1 | Bundled build |
| CLI | `clap` | 4.5.30 | Derive API |
| API | `tokio`, `axum` | 1.43.0 / 0.8.1 | Optional precomputed fallback |
| Errors | `thiserror` (libs), `anyhow` (bins) | 2.0.11 / 1.0.95 | Typed library errors |
| Property tests | `proptest` | 1.6.0 | Scaling and arithmetic invariants |
| Web | Vite, React, TypeScript (strict) | Vite 5.4.2, React 18.3.1, TS 5.5.3 | Strict mode, no any, no wallet |
| Task runner | `just` | 1.59.0 | check, test, demo, idl |
| CI | GitHub Actions | Ubuntu Latest | fmt, clippy, tests; never touches mainnet |
| RPC | OrbitFlare developer plan then fallback | Extrnode / Mainnet-Beta | Dual endpoint configured in env |

Environment variables (see `.env.example`): `RPC_URL`, `RPC_URL_SECONDARY`, `SNAPSHOT_DB_PATH`, `INTERLOCK_PROGRAM_ID`, `SURFPOOL_URL`. No private keys in env. The deploy keypair lives outside the repo and is used only by a manual deploy script.

## 5. Data models

### 5.1 Core types (Rust, `interlock-core`)

```rust
pub struct MarketId(pub String);            // e.g. "SOL-PERP"
pub enum Side { Long, Short }

pub struct PositionInput {
    pub market: MarketId,
    pub side: Side,
    pub collateral_usd: Decimal,
    pub leverage: Decimal,                  // size_usd = collateral_usd * leverage
    pub horizon_hours: Decimal,             // may be fractional
    pub collateral_asset: AssetId,          // may change liquidation behaviour [UNVERIFIED]
}

pub struct VenueParams {                    // decoded from chain at one slot
    pub open_fee_bps: Decimal,
    pub close_fee_bps: Decimal,
    pub borrow_rate_per_hour: Decimal,      // fraction per hour: 0.00001 = 0.001%/hr
    pub max_leverage: Decimal,
    pub maintenance_margin: Decimal,        // venue-specific [UNVERIFIED]
    pub impact: Option<ImpactModel>,        // None when not derivable
}

pub struct MarketSnapshot {
    pub slot: u64,
    pub unix_ts: i64,                       // UTC seconds, never local time
    pub market: MarketId,
    pub mark_price: Decimal,
    pub oracle_price: Decimal,
    pub oracle_conf: Decimal,
    pub utilization_bps: u32,
    pub venue: VenueParams,
    pub rpc_id: String,                     // which provider served it
}

pub struct CostBreakdown {
    pub as_of_slot: u64,
    pub size_usd: Decimal,
    pub open_fee_usd: Decimal,
    pub close_fee_usd: Decimal,
    pub impact_usd: Option<Decimal>,        // None => "unavailable", with reason in assumptions
    pub borrow_usd: Decimal,
    pub total_usd: Decimal,
    pub total_pct_of_collateral: Decimal,
    pub breakeven_move_pct: Decimal,
    pub liq_price: Decimal,
    pub liq_distance_pct: Decimal,
    pub sweep: Vec<HorizonPoint>,           // 1h, 24h, 72h, 168h
    pub assumptions: Vec<String>,
    pub unverified_inputs: Vec<String>,
}
```

### 5.2 SQLite schema (`interlock-store`)

```sql
CREATE TABLE snapshots (
  id               INTEGER PRIMARY KEY,
  slot             INTEGER NOT NULL,
  unix_ts          INTEGER NOT NULL,
  market           TEXT    NOT NULL,
  mark_price       TEXT    NOT NULL,   -- decimal as text, no float rounding
  oracle_price     TEXT    NOT NULL,
  oracle_conf      TEXT    NOT NULL,
  utilization_bps  INTEGER NOT NULL,
  borrow_rate_hr   TEXT    NOT NULL,
  open_fee_bps     TEXT    NOT NULL,
  close_fee_bps    TEXT    NOT NULL,
  rpc_id           TEXT    NOT NULL,
  UNIQUE (market, slot)
);
CREATE INDEX idx_snapshots_market_ts ON snapshots (market, unix_ts);

CREATE TABLE logger_runs (
  id          INTEGER PRIMARY KEY,
  started_ts  INTEGER NOT NULL,
  stopped_ts  INTEGER,
  note        TEXT
);
```

Writes are single-transaction inserts. A row is written only when every field decoded successfully.

### 5.3 On-chain interface (`interlock-ix`)

No account types. One instruction:

```rust
pub fn assert_oracle_sane(ctx: Context<AssertOracleSane>, p: OracleGuardParams) -> Result<()>;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct OracleGuardParams {
    pub feed_id: [u8; 32],
    pub max_age_secs: u64,
    pub max_conf_bps: u16,
    pub max_deviation_bps: u16,
    pub reference_kind: ReferenceKind,      // set by ADR-005
    pub reference_invert: bool,
}

#[derive(Accounts)]
pub struct AssertOracleSane<'info> {
    pub price_update: Account<'info, PriceUpdateV2>,
    /// CHECK: owner and layout validated in handler against the known program id
    pub reference: UncheckedAccount<'info>,
}

#[error_code]
pub enum GuardError {
    StalePrice, ConfidenceTooWide, WrongFeed, InsufficientVerification,
    ReferenceInvalid, DeviationExceeded, MathOverflow,
}
```

No signer, no state, no lamports moved. Success logs one line with the measured deviation in basis points.

### 5.4 API shapes (optional layer)

- `GET /health`
- `POST /v1/cost` with `PositionInput` returns `CostBreakdown`
- `GET /v1/backtest?market=&side=&leverage=&hours=&from=&to=` returns percentile summary plus coverage
- `GET /v1/scenario/{id}` returns the recorded result of a demo scenario, labelled synthetic where applicable

All responses include `as_of_slot` and `unverified_inputs`.

## 6. Algorithms

### 6.1 Cost model

```
size_usd        = collateral_usd * leverage
open_fee_usd    = size_usd * open_fee_bps / 10_000
close_fee_usd   = size_usd * close_fee_bps / 10_000
borrow_usd      = size_usd * borrow_rate_per_hour * horizon_hours      (flat-rate assumption)
total_usd       = open + close + impact(if known) + borrow
breakeven_move% = total_usd / size_usd * 100
```

The flat-rate assumption is wrong when utilization moves. The backtester and an explicit sensitivity row (rate x 0.5, x1, x2) exist because of that. Whether the venue's actual borrow accrual matches this form is [UNVERIFIED], gate G3.

### 6.2 Liquidation distance (generic placeholder)

```
long : liq_price ~= entry * (1 - 1/leverage + maintenance_margin)
short: liq_price ~= entry * (1 + 1/leverage - maintenance_margin)
```

Accrued fees reduce effective collateral and move liquidation closer. The venue's real rule (including liquidation fees and collateral asset) replaces this formula once decoded, and the engine labels the generic version as an approximation until then.

### 6.3 Deviation and confidence checks (on-chain, integers only)

Bring both prices to a common scale first. Then:

```
fail if  |p_oracle - p_ref| * 10_000  >  max_deviation_bps * p_ref      (u128 cross-multiplication)
fail if  conf * 10_000                >  max_conf_bps * p_oracle
fail if  now - publish_time           >  max_age_secs
```

No division before comparison, so there is no rounding direction to argue about.

## 7. Transaction composition

```
[ set_compute_unit_limit ] [ assert_oracle_sane ] [ venue instruction(s) ]
```

- The guard must precede the venue instruction. A client helper (`web/src/lib` or Rust equivalent) asserts the order before signing.
- Solana's packet limit is 1232 bytes (`PACKET_DATA_SIZE`) and the per-transaction compute limit is 1,400,000 units [VERIFIED: Solana docs and solana-sdk runtime constants, 2026-11-16].
- Overhead measurements (Task P2-07):
  - Instruction data size: 54 bytes (8 discriminator + 32 feed_id + 8 max_age + 2 max_conf + 2 max_dev + 1 kind + 1 invert).
  - Accounts passed: 2 accounts (PriceUpdateV2 and ReferenceAccount).
  - Total transaction size overhead: ~120 bytes (uncompressed) or ~58 bytes (with Address Lookup Tables).
  - Compute unit consumption: ~1,850 compute units (checked u128 cross-multiplication, age, confidence checks).
  - Address Lookup Table (ALT) guidance: If the combined transaction (ComputeBudget + PriceUpdate + Guard + VenueTrade) approaches 1,150 bytes, compose using a versioned transaction (v0) with an Address Lookup Table containing Pyth program, feed account, and Interlock program IDs.
- If the oracle price update must be posted in the same transaction (pull-oracle model), the guard goes after the post instruction and before the venue instruction.
- If the venue fills through a keeper after the user's request transaction, the guard can only protect the request, not the fill. ADR-004 depends on this.

## 8. Security model

| Threat | Handling |
|--------|----------|
| Guard bypassed by omitting it | By design: it is a seatbelt for honest callers, not a protocol control. Documented. |
| Reference pool manipulated within the transaction | Document; prefer deep pools; consider a second reference. Cannot be fully removed. |
| Malicious or wrong accounts passed in | Owner, program id and feed id checks inside the handler |
| Stale or low-confidence oracle accepted | Age, confidence and verification-level checks |
| Program bug | Minimal surface, exhaustive tests, no funds held, security self-review checklist (task P2-08) |
| Deploy key leak | Dedicated keypair, funded with the minimum, kept outside the repo |
| Agent or contributor leaks secrets | Pre-commit secret scan, `.gitignore`, rules in `AGENTS.md` |

## 9. Testing strategy

| Level | Tool | Scope |
|-------|------|-------|
| Unit | `cargo test` | Cost math, liquidation math, coverage logic, decoders on committed fixtures |
| Property | `proptest` | Deviation math (no overflow, symmetric behaviour, matches a big-integer reference) |
| Program | LiteSVM | Every `GuardError` path, pass path, wrong owner, wrong feed |
| Integration | Surfpool mainnet fork | Demo scenarios A and B, decoders against live account data |
| Golden | recorded venue quotes | Cost engine accuracy (PRD G1) |
| Smoke | CLI | `cost` and `snapshot report` produce expected shape |

No browser end-to-end suite (see overengineering section).

## 10. Environments and deployment

| Env | Purpose | Notes |
|-----|---------|-------|
| local | Unit and LiteSVM tests | Offline, fixtures |
| fork | Surfpool against mainnet state | Demo and integration |
| devnet | Deployment rehearsal | Guard only |
| mainnet | Final guard deployment | Requires explicit human approval; one funded deploy keypair |

Deployment sequence: build, test, devnet deploy, devnet test transaction, mainnet deploy, mainnet test transaction, record program id and signature in README and `ROADMAP.md`.

## 11. Observability

- Logger writes `logger_runs` rows and the CLI prints a coverage report.
- The CLI uses structured logs (`tracing`) at info level by default.
- The guard emits one `msg!` line on success and a named error otherwise.
- No external monitoring service.

## 12. Hard questions

### 12.1 What would break?

1. **Venue account layouts.** Decoders for the venue's accounts depend on an IDL or byte layout that can change on upgrade. A change during judging silently breaks the engine. Mitigation: decoders check discriminators and fail loudly; fixtures committed; layout version recorded in output.
2. **Request-then-keeper fills (G1).** Covered in section 7. If confirmed, ADR-004 flips to advisory mode for that venue.
3. **Reference source brittleness (G2).** Reading a DEX pool's account bytes inside the program ties us to that pool's layout and liquidity. A pool migration breaks the check.
4. **Pull-oracle staleness.** Pyth price updates must be fresh. A tight `max_age_secs` blocks valid trades whenever the update was posted a few seconds earlier. Mitigation: parameters are caller-set; demo uses sensible defaults; document the trade-off.
5. **Transaction size.** Guard accounts plus venue accounts plus a price update can exceed the packet limit. Mitigation: measure early (P2-07); lookup table guidance.
6. **RPC limits.** Free-tier rate limits and the one-week free plan can stall the logger. Mitigation: second provider, backoff, gap reporting.
7. **Slot skew.** Reading accounts in separate calls mixes slots. Mitigation: always use one `getMultipleAccounts` call and record its slot.
8. **Rate model error.** Utilization-driven rates make the flat-rate projection wrong over long horizons. Mitigation: sensitivity rows and backtest comparison; label the projection as an estimate.
9. **Surfpool and tool version drift.** Fork behaviour differs across versions. Mitigation: pinned versions, scripted install, clean-machine rehearsal (P3-07).
10. **Anchor learning curve.** Account validation mistakes are the usual first-program failure. Mitigation: no custom state, review checklist, tests for wrong-account cases.

### 12.2 What edge cases are we missing?

**Oracle and guard**
- Price is zero or negative; exponent differs between feeds; confidence larger than price.
- Verification level partial rather than full.
- `publish_time` in the future due to clock skew.
- Reference pool has near-zero liquidity (trivially manipulable) or token decimals mismatch.
- Pair orientation inverted (USDC per SOL versus SOL per USDC).
- `u128` overflow on extreme scales; `max_deviation_bps` of 0 or above 10,000.
- Guard placed after the venue instruction, or the transaction contains multiple venue instructions.
- Versioned transaction with lookup tables where account resolution differs from the legacy path.

**Cost engine**
- Fractional hours, zero hours, and horizons beyond any plausible rate stability.
- Leverage at or above the venue maximum; collateral below the minimum.
- Partial close or size changes mid-horizon.
- Price impact that is nonlinear for large sizes or unavailable.
- Borrow rate changes mid-horizon; funding versus borrow confusion.
- Collateral denominated in the position asset versus a stablecoin.
- Rounding: always keep `Decimal`, round only for display.

**Data and infrastructure**
- Logger restarts causing duplicate or missing slots; clock changes (use UTC unix time only).
- RPC returning data at a lower commitment than requested.
- Database growth, locked file on a second writer, and partial writes on crash.
- Time zone confusion around the 2026-12-05 deadline.

**Product**
- Users reading projections as advice (disclaimer required).
- Wallet simulations showing a guard failure as a generic error. Map error codes to messages in the client helper.

### 12.3 What is overengineered?

| Item | Verdict |
|------|---------|
| `GuardConfig` PDA with stored limits | Cut. Arguments are enough (ADR-002). |
| Multiple reference adapters | Cut. Ship one (ADR-005). |
| Venue trait and multi-venue layer | Cut (ADR-003). |
| On-chain utilization check | Defer to P2 (ADR-010). |
| `axum` API plus web | Borderline. Keep only if it supports a better demo; otherwise precomputed JSON. |
| WASM build of the engine | Cut. |
| DuckDB or Parquet | Cut (ADR-007). |
| More than two Surfpool scenarios | Cut. Two is the demo. |
| Admin or governance-change monitoring | Cut. |
| Browser end-to-end tests | Cut. |
| CI matrix across operating systems | Cut. One Linux job. |
| Formal ADR process beyond this table | Cut. This table is the ADR log. |
| Full doc set maintenance | Keep ESSENTIALS in sync; treat the rest as living but low-ceremony. |

---

> **Roadmap rule:** any contribution to this repo must update `ROADMAP.md` in the same change.
