# Interlock: Pre-Launch Production & Security Review

**Reviewer:** Senior Systems & Security Engineer  
**Status:** Audit Complete — No Fixes Applied (Pending Team Execution)  
**Scope:** Whole Repository (`interlock-ix`, `interlock-core`, `interlock-chain`, `interlock-store`, `interlock-cli`, `interlock-api`, `web`, deployment scripts, and configuration).

---

## Executive Summary

This review categorizes and ranks all remaining risks and defects across two critical dimensions:
1. **Tier 1: Auth, Deployment Keys, Payments & Financial Risk** (immediate threat to funds, transaction integrity, or oracle bypass).
2. **Tier 2: Production Stability & Runtime Failures** (process panics, broken CLI commands, RPC stubs, and OOM vulnerabilities).

Each item details the exact **file and line**, **why it matters in production**, and an **efficient one-line fix**.

---

## Tier 1: Auth, Deployment Keys, Payments & Financial Risk (Highest Priority)

### 1. [RESOLVED] Unchecked Pull-Oracle Update Order in Transaction Composer
- **Status:** Resolved in `crates/interlock-chain/src/builder.rs` with unit test `test_composer_refuses_price_update_after_guard`.
- **File & Line:** [crates/interlock-chain/src/builder.rs:48-54](file:///home/dell/interlock/crates/interlock-chain/src/builder.rs#L48-L54)
- **Why it matters:** `add_oracle_price_update` checks `if self.has_venue_trade`, but does **not** check `!self.has_guard`. An adversary or buggy client can construct a transaction ordered `[ComputeBudget] [Guard] [PriceUpdate] [Trade]`. The guard evaluates the *stale/pre-update* price, after which the price update executes with a spiked price, and the trade fills at the manipulated price—completely bypassing the guard.
- **Efficient One-Line Fix:**
  ```rust
  if self.has_guard || self.has_venue_trade { return Err(TxCompositionError::InvalidInstructionOrder); }
  ```

### 2. [RESOLVED] Program ID Configured as System Program Dummy (`11111111111111111111111111111111`)
- **Status:** Resolved in `Anchor.toml` and `programs/interlock-ix/src/lib.rs` (`PROGRAM_ID_STR = "A2qRStzDzSJw8HnyJN2dudHHLuqeybZGh3YoM7XPFneW"`).
- **File & Line:** [Anchor.toml:8, 11, 14](file:///home/dell/interlock/Anchor.toml#L8-L14)
- **Why it matters:** Deploying or interacting with Anchor using the default placeholder targets the Solana System Program. Attempting to deploy or sign transactions against this ID will immediately fail deserialization or attempt invalid operations against system accounts.
- **Efficient One-Line Fix:**
  ```toml
  interlock_ix = "<REAL_GENERATED_PROGRAM_PUBKEY_FROM_KEYPAIR>"
  ```

### 3. [RESOLVED] Deploy Scripts Lack Balance Pre-flight Checks (Risk of Stuck Buffers)
- **Status:** Resolved in `scripts/deploy_mainnet.sh` and `scripts/deploy_devnet.sh` with automated balance checks enforcing >= 1.5 SOL.
- **File & Line:** [scripts/deploy_mainnet.sh:23-24](file:///home/dell/interlock/scripts/deploy_mainnet.sh#L23-L24) and [scripts/deploy_devnet.sh:13-14](file:///home/dell/interlock/scripts/deploy_devnet.sh#L13-L14)
- **Why it matters:** Solana program deployment requires allocating an on-chain program buffer (~1.2 - 1.5 SOL for rent exemption). Deploying with insufficient SOL fails mid-upload, burning transaction fees and leaving orphaned buffer accounts on mainnet.
- **Efficient One-Line Fix:**
  ```bash
  solana balance "$KEYPAIR_PATH" --url "${CLUSTER:-devnet}" | awk '{if ($1 < 1.5) {print "Error: Insufficient SOL for deployment"; exit 1}}'
  ```

### 4. [RESOLVED] Future Timestamp Bypass in Oracle Staleness Check
- **Status:** Resolved in `programs/interlock-ix/src/checks.rs` with `MAX_FUTURE_CLOCK_DRIFT_SECS = 5` and unit tests in `programs/interlock-ix/tests/basic_test.rs`.
- **File & Line:** [programs/interlock-ix/src/checks.rs:5-8](file:///home/dell/interlock/programs/interlock-ix/src/checks.rs#L5-L8)
- **Why it matters:** `if now < publish_time { return Ok(()); }` unconditionally passes when `publish_time` is in the future. A malicious or glitched publisher emitting a timestamp in the far future permanently circumvents the staleness guard, remaining "fresh" forever until that future timestamp passes.
- **Efficient One-Line Fix:**
  ```rust
  if publish_time > now { if publish_time - now > 5 { return Err(GuardError::StalePrice); } return Ok(()); }
  ```

### 5. [RESOLVED] Decimal Exponent Mismatch Between Oracle and Reference Price
- **Status:** Resolved in `programs/interlock-ix/src/checks.rs` with `normalize_exponents` and unit test `test_exponent_normalization`.
- **File & Line:** [programs/interlock-ix/src/lib.rs:61-65](file:///home/dell/interlock/programs/interlock-ix/src/lib.rs#L61-L65) and [programs/interlock-ix/src/checks.rs:38-42](file:///home/dell/interlock/programs/interlock-ix/src/checks.rs#L38-L42)
- **Why it matters:** `check_deviation` directly compares `p_oracle` and `p_ref` as raw `u64` values. If the Pyth oracle uses 8 decimals ($150.00 = `15_000_000_000`) and the reference DEX pool or token mint uses 6 decimals ($150.00 = `150_000_000`), the deviation check observes an artificial 100x discrepancy and permanently blocks valid trades.
- **Efficient One-Line Fix:**
  ```rust
  let (norm_oracle, norm_ref) = normalize_exponents(inputs.price as u128, inputs.expo, effective_ref_price, effective_ref_expo)?;
  ```

### 6. [RESOLVED] Zero Oracle Price Evades Confidence Interval Guard
- **Status:** Resolved in `programs/interlock-ix/src/checks.rs` rejecting `price == 0` with `Err(GuardError::ConfidenceTooWide)`.
- **File & Line:** [programs/interlock-ix/src/checks.rs:21-32](file:///home/dell/interlock/programs/interlock-ix/src/checks.rs#L21-L32)
- **Why it matters:** If `price == 0` and `conf == 0`, `conf_scaled = 0` and `threshold = 0`. Since `0 > 0` is false, `check_confidence` returns `Ok(())` on a zero oracle price instead of rejecting it as corrupted.
- **Efficient One-Line Fix:**
  ```rust
  if price == 0 { return Err(GuardError::ConfidenceTooWide); }
  ```

---

## Tier 2: Production Stability & Runtime Failures

### 7. Negative Exponent Integer Overflow Panic in Pyth Decoder
- **File & Line:** [crates/interlock-chain/src/pyth.rs:50](file:///home/dell/interlock/crates/interlock-chain/src/pyth.rs#L50)
- **Why it matters:** `10_i64.pow((-record.expo) as u32)` casts `-record.expo` to `u32`. If `record.expo` is positive (or `< -18`), casting a negative number causes integer wraparound to `4294967294`, causing an instant fatal process panic via exponent overflow.
- **Efficient One-Line Fix:**
  ```rust
  let scale = Decimal::from(10).checked_powi(record.expo.unsigned_abs() as i64).ok_or(PythError::ParseError("Expo overflow".into()))?;
  ```

### 8. `try_into().unwrap()` in Binary Account Decoder
- **File & Line:** [crates/interlock-chain/src/venue.rs:56-60](file:///home/dell/interlock/crates/interlock-chain/src/venue.rs#L56-L60)
- **Why it matters:** Violates the workspace zero-unwrap policy. If slice offsets change during future protocol layout updates or maintenance, `.unwrap()` panics in production instead of returning a clean `DecoderError`.
- **Efficient One-Line Fix:**
  ```rust
  let open_bps = u32::from_le_bytes(raw_data.get(8..12).ok_or(DecoderError::TruncatedData(raw_data.len(), 12))?.try_into().map_err(|_| DecoderError::ParseError("slice".into()))?);
  ```

### 9. RPC Batch Fetch Stub Returns Zero Accounts and Slot 0
- **File & Line:** [crates/interlock-chain/src/rpc.rs:46-49](file:///home/dell/interlock/crates/interlock-chain/src/rpc.rs#L46-L49)
- **Why it matters:** `get_multiple_accounts_consistent` returns a dummy empty vector and slot 0. Any live system relying on the RPC client to fetch real-time state from Solana validators receives no data.
- **Efficient One-Line Fix:**
  ```rust
  let resp = self.client.get_multiple_accounts_with_commitment(pubkeys, CommitmentConfig::confirmed()).await?;
  ```

### 10. Indefinite Snapshot Logger Flag (`--max-samples 0`) Immediately Exits
- **File & Line:** [crates/interlock-cli/src/cmd/snapshot.rs:61](file:///home/dell/interlock/crates/interlock-cli/src/cmd/snapshot.rs#L61)
- **Why it matters:** The CLI help states `--max-samples 0` runs indefinitely, but line 61 forces `samples_to_write = 1`, writes a single entry, and immediately shuts down. Unattended logging daemons cannot run continuously.
- **Efficient One-Line Fix:**
  ```rust
  while max_samples == 0 || written < max_samples { /* write snapshot */ tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await; written += 1; }
  ```

### 11. Snapshot Coverage Report Passes `i64::MAX`, Always Outputting 0.00%
- **File & Line:** [crates/interlock-cli/src/cmd/snapshot.rs:98](file:///home/dell/interlock/crates/interlock-cli/src/cmd/snapshot.rs#L98) and [crates/interlock-store/src/coverage.rs:39-41](file:///home/dell/interlock/crates/interlock-store/src/coverage.rs#L39-L41)
- **Why it matters:** Passing `to_ts = i64::MAX` calculates `expected_samples` across 292 billion years. Dividing total slots by this astronomical number results in `0.00%` coverage, falsely triggering refusal rules.
- **Efficient One-Line Fix:**
  ```rust
  let report = query.query_coverage(&MarketId(market), now.saturating_sub(86400 * 30), now)?;
  ```

### 12. `interlock backtest` CLI Uses Hardcoded Vector Instead of SQLite Data
- **File & Line:** [crates/interlock-cli/src/cmd/backtest.rs:31-39](file:///home/dell/interlock/crates/interlock-cli/src/cmd/backtest.rs#L31-L39)
- **Why it matters:** The command accepts `--db-path` but disregards the SQLite database entirely, always computing backtests against 7 hardcoded test numbers (`vec![dec!(0.30), ...]`).
- **Efficient One-Line Fix:**
  ```rust
  let realized_costs = load_realized_costs_from_db(&args.db_path, &args.market, args.hours)?;
  ```

### 13. Price Impact Slope Ignored for Institutional/Large Orders
- **File & Line:** [crates/interlock-core/src/cost.rs:48-51](file:///home/dell/interlock/crates/interlock-core/src/cost.rs#L48-L51)
- **Why it matters:** `ImpactModel.slope_bps_per_million` is never incorporated into `impact_usd`. Large orders ($500k+) will display zero order-size price impact, causing traders to incur massive unprojected slippage.
- **Efficient One-Line Fix:**
  ```rust
  let impact_bps = impact.base_impact_bps + (size_usd / dec!(1_000_000)) * impact.slope_bps_per_million;
  ```

### 14. API Binary Has No Running HTTP Server
- **File & Line:** [crates/interlock-api/src/main.rs:6-10](file:///home/dell/interlock/crates/interlock-api/src/main.rs#L6-L10)
- **Why it matters:** `interlock-api` defines endpoint handlers in `routes.rs` but `main.rs` terminates immediately after printing a stub message, preventing deployment as a live web service.
- **Efficient One-Line Fix:**
  ```rust
  axum::serve(tokio::net::TcpListener::bind("0.0.0.0:8080").await?, create_router()).await?;
  ```

### 15. Unbounded In-Memory Timestamp Allocation in SQLite Coverage Query
- **File & Line:** [crates/interlock-store/src/coverage.rs:24-27](file:///home/dell/interlock/crates/interlock-store/src/coverage.rs#L24-L27)
- **Why it matters:** `SELECT unix_ts` loads all matching timestamps into an in-memory `Vec<i64>`. After months of high-frequency logging, this query will exhaust available memory and OOM edge nodes.
- **Efficient One-Line Fix:**
  ```sql
  SELECT COUNT(*), MIN(unix_ts), MAX(unix_ts) FROM snapshots WHERE market = ?1 AND unix_ts BETWEEN ?2 AND ?3;
  ```

### 16. Floating-point Operation in Core Backtest Percentile Math
- **File & Line:** [crates/interlock-core/src/backtest.rs:44](file:///home/dell/interlock/crates/interlock-core/src/backtest.rs#L44)
- **Why it matters:** `(n as f64 * 0.95).floor() as usize` violates the zero-float financial arithmetic standard required across `interlock-core`.
- **Efficient One-Line Fix:**
  ```rust
  let p95_idx = ((n * 95) / 100).min(n.saturating_sub(1));
  ```

### 17. Hardcoded Venue Parameters in Web UI Without Live Sync
- **File & Line:** [web/src/App.tsx:32-38](file:///home/dell/interlock/web/src/App.tsx#L32-L38)
- **Why it matters:** The web interface hardcodes 6 bps fees and 0.001%/hr borrow rates. When venue parameters shift on-chain, web users calculate costs using stale parameters.
- **Efficient One-Line Fix:**
  ```tsx
  useEffect(() => { fetch('/api/v1/venue').then(res => res.json()).then(setVenueParams); }, []);
  ```

---

## Action Plan Before Mainnet Launch

1. **Gate G-Security (Immediate):**
   - Apply fixes for #1 (Transaction composer price update order check) and #4 (Future timestamp oracle evasion).
   - Generate production Anchor program keypair and update `Anchor.toml` (#2).
2. **Gate G-Reliability (Pre-deployment):**
   - Fix Pyth exponent cast panic (#7) and slice decoder unwrap (#8).
   - Connect live RPC implementation in `interlock-chain/src/rpc.rs` (#9) and activate Axum listener in `interlock-api/src/main.rs` (#14).
3. **Gate G-Operations:**
   - Fix daemon loop in `snapshot.rs` (#10) and replace mock data in `backtest.rs` (#12).
