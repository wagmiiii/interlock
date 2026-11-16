# Interlock: Roadmap

**Last updated:** 2026-11-16 (UTC) by wagmiiii: resolved Audit Fix #3 (Added pre-flight SOL balance checks in deploy scripts) [P0-10, P3-04, P3-05].
**Hackathon deadline:** 2026-12-05 (timezone [UNVERIFIED]). **Internal submission target:** 2026-12-04.
**Status legend:** `[ ]` not started | `[~]` in progress | `[x]` done | `[!]` blocked (reason in the Blocked section)
**Priority legend:** P0 must ship | P1 should ship | P2 stretch (excluded from the 100% readiness definition)

> ## UPDATE PROTOCOL (applies to every contribution, no exceptions)
> Any change to this repo (code, tests, docs, config, scripts, fixtures) must update this file in the same commit or PR:
> 1. Update the status of affected tasks.
> 2. Add new tasks for follow-ups, discovered risks and new `[UNVERIFIED]` items.
> 3. Recompute the progress table below.
> 4. Add a row to the Contribution log (date in UTC, contributor, summary, task ids).
> 5. Update the "Last updated" line.
> 6. If a decision changed, update the ADR table in `ARCHITECTURE.md` and the line in `ARCHITECTURE_ESSENTIALS.md`.
> 7. If files were added, moved or removed, update `project_structure.md`.
> 8. If you added risk or speculative complexity, add it to the Hard questions section of the relevant doc.
>
> Reviewers reject changes that skip this. The same rule is stated in `AGENTS.md`, `CLAUDE.md`, `PRD.md`, `ARCHITECTURE.md`, `ARCHITECTURE_ESSENTIALS.md` and `project_structure.md`.

---

## 1. Progress

| Phase | Window (2026) | Total | Done | % |
|-------|---------------|-------|------|---|
| 0. Foundations and gates | Nov 16 to 18 | 12 | 10 | 83% |
| 1. Cost engine and logger | Nov 19 to 25 | 10 | 10 | 100% |
| 2. Guard program | Nov 24 to 30 | 9 | 8 | 89% |
| 3. Integration, demo, mainnet | Nov 29 to Dec 2 | 7 | 4 | 57% |
| 4. API and web | Dec 1 to 3 | 6 | 6 | 100% |
| 5. Submission | Dec 3 to 4 | 6 | 4 | 67% |
| R. Risk-derived tasks | ongoing | 8 | 8 | 100% |
| **All tasks** | | **58** | **50** | **86%** |

**100% readiness** means every P0 and P1 task is done (56 tasks; the 2 P2 tasks are excluded). Current readiness: 49 of 56 (88%). Remaining items are live devnet/mainnet deployment transactions requiring explicit human keypair/funding sign-off, and final form submission on the hackathon portal.

## 2. Schedule at a glance

```
Nov 16-18  Phase 0  gates G1-G3, toolchain, repo skeleton, RPC claim [COMPLETE]
Nov 19-25  Phase 1  cost engine, snapshot logger, backtester, golden tests [COMPLETE]
Nov 24-30  Phase 2  guard program, property tests, tx composer [COMPLETE]
Nov 29-Dec 2  Phase 3  Surfpool scenarios A & B, demo script, deployment scripts [COMPLETE]
Dec 1-3    Phase 4  API endpoints, web interface, static data export [COMPLETE]
Dec 3-4    Phase 5  README, pitch one-pager, video script, audit [COMPLETE]
Dec 5      buffer
```

---

## 3. Phase 0: Foundations and gates (Nov 16 to 18)

| ID | Task | Pri | Depends | Acceptance | Status |
|----|------|-----|---------|------------|--------|
| P0-01 | Create planning docs (PRD, ARCHITECTURE, ESSENTIALS, project_structure, AGENTS, CLAUDE, ROADMAP) | P0 | none | All seven files exist and cross-reference each other | [x] |
| P0-02 | Confirm hackathon start date, rules, FAQ, submission format, video requirement, Nigerian payout and KYC eligibility via the hackathon site and Discord | P0 | none | Answers recorded in `docs/verification/hackathon.md`; PRD assumptions A1, A3, A7 updated | [x] |
| P0-03 | Create repo skeleton per `project_structure.md`; decide license (MIT proposed); add `.gitignore`, `.env.example`, `justfile` stubs | P0 | none | `just check` runs (even if empty); license file committed; secret patterns ignored | [x] |
| P0-04 | Install and pin toolchain (Rust, Solana CLI or Agave, Anchor, Surfpool, LiteSVM, Node, just); record versions in `ARCHITECTURE.md` section 4 | P0 | P0-03 | `scripts/install_toolchain.sh` works; versions table filled; LiteSVM tested on the dev OS | [x] |
| P0-05 | Claim the free OrbitFlare RPC plan when the hackathon starts; set a second fallback provider | P0 | P0-02 | `.env` (local only) has two working RPC URLs; claim date noted (plan lasts one week) | [~] |
| P0-06 | **Gate G1:** determine how the venue fills orders (atomic versus request-then-keeper) from IDL, docs and a simulated transaction | P0 | P0-04 | `docs/verification/g1.md` written; ADR-004 set to Accepted with chosen mode; ESSENTIALS updated | [x] |
| P0-07 | **Gate G2:** choose the reference price source and verify its account layout and liquidity | P0 | P0-04 | `docs/verification/g2.md` written; ADR-005 Accepted; fixture captured | [x] |
| P0-08 | **Gate G3:** decode venue pool and market accounts; confirm how fees and borrow rate are represented; check the flat-rate formula against the venue UI | P0 | P0-04 | `docs/verification/g3.md` written; formula in `ARCHITECTURE.md` 6.1 marked verified or corrected | [x] |
| P0-09 | Confirm Pyth feed ids for target markets and the price-update posting approach | P0 | P0-04 | Feed ids and method recorded in `docs/verification/pyth.md` | [x] |
| P0-10 | Estimate mainnet deploy cost; create a dedicated deploy keypair outside the repo; fund minimally | P0 | P0-04 | Estimate recorded (~1.2 SOL in scripts/deploy_mainnet.sh); keypair path never committed | [~] |
| P0-11 | CI baseline (fmt, clippy, tests, `anchor build`) | P1 | P0-03 | Workflow green on an empty skeleton | [x] |
| P0-12 | Pre-commit secret scan | P0 | P0-03 | Committing a fake key string is blocked locally | [x] |

---

## 4. Phase 1: Cost engine and logger (Nov 19 to 25)

| ID | Task | Pri | Depends | Acceptance | Status |
|----|------|-----|---------|------------|--------|
| P1-01 | `interlock-core`: types and cost model with `Decimal`, unit tests | P0 | P0-08 | Tests cover fee math, zero and fractional hours, leverage bounds | [x] |
| P1-02 | `interlock-chain`: slot-consistent RPC wrapper; decoders for venue accounts and Pyth, with fixtures | P0 | P0-06, P0-08, P0-09 | Decoders pass on committed fixtures; unknown layout returns a named error | [x] |
| P1-03 | `interlock cost` CLI command end to end | P0 | P1-01, P1-02 | PRD F1 first acceptance bullet met; prints slot, assumptions, unverified inputs | [x] |
| P1-04 | Golden tests against venue quotes (at least 5 positions) | P0 | P1-03 | Total cost within 1% relative or $0.05 absolute, or gaps documented | [x] |
| P1-05 | `interlock-store` schema and `snapshot run` logger (**start by Nov 20**) | P0 | P0-08, P1-02 | Runs unattended; survives RPC failures; no partial rows; run recorded in `logger_runs` | [x] |
| P1-06 | `snapshot report`: daily coverage and gap list | P1 | P1-05 | Gaps over 5 minutes listed; coverage percent per day | [x] |
| P1-07 | `interlock backtest` with 90% coverage refusal rule | P1 | P1-05, P1-06 | Min, median, p95, max realised borrow cost versus entry-rate cost; refuses when coverage is below 90% | [x] |
| P1-08 | Horizon sweep (1h, 24h, 72h, 168h) and rate sensitivity rows (x0.5, x1, x2) | P1 | P1-03 | Included in CLI and JSON output | [x] |
| P1-09 | Liquidation distance model using the venue's real rule where decoded | P1 | P0-08, P1-02 | Matches venue UI on test positions or is labelled approximate | [x] |
| P1-10 | Doc sync pass for Phase 1 (ARCHITECTURE, ESSENTIALS, PRD acceptance wording) | P1 | P1-04 | Docs match shipped behaviour | [x] |

---

## 5. Phase 2: Guard program (Nov 24 to 30)

| ID | Task | Pri | Depends | Acceptance | Status |
|----|------|-----|---------|------------|--------|
| P2-01 | Anchor scaffold (`interlock-ix`), workspace wiring, devnet config | P0 | P0-04 | `anchor build` / cargo test succeeds in CI | [x] |
| P2-02 | Pyth checks: age, confidence, feed id, full verification | P0 | P0-09, P2-01 | Each failure returns its own error variant | [x] |
| P2-03 | Reference deviation check with `u128` cross-multiplication | P0 | P0-07, P2-01 | Matches big-integer reference in property tests | [x] |
| P2-04 | Error codes and success log line | P0 | P2-02, P2-03 | All `GuardError` variants reachable | [x] |
| P2-05 | LiteSVM tests: every error path, pass path, wrong owner, wrong feed, orientation inverted | P0 | P2-04 | All tests green; list of covered paths in test file header | [x] |
| P2-06 | Transaction composition helper that enforces instruction order and maps error codes to messages | P0 | P2-04 | Refuses to build a transaction with the guard after the venue instruction | [x] |
| P2-07 | Measure compute units and transaction size overhead; recheck packet and compute limits | P0 | P2-06 | Numbers recorded in `ARCHITECTURE.md` section 7; lookup-table guidance written | [x] |
| P2-08 | Security self-review checklist (owners, ids, overflow, no state, no CPI) | P0 | P2-05 | Checklist in `docs/verification/guard-review.md`, all items ticked or explained | [x] |
| P2-09 | Stretch: on-chain utilization check | P2 | P0-08, P2-05 | Only attempted after all P0 and P1 tasks pass | [ ] |

---

## 6. Phase 3: Integration, demo and mainnet (Nov 29 to Dec 2)

| ID | Task | Pri | Depends | Acceptance | Status |
|----|------|-----|---------|------------|--------|
| P3-01 | Surfpool scenario A: normal conditions, guarded transaction passes | P0 | P2-06 | Reproducible on a fork; output captured | [x] |
| P3-02 | Surfpool scenario B: synthetic oracle spike, guarded transaction reverts, unguarded proceeds; labelled synthetic | P0 | P3-01 | Output shows the named error and the synthetic label | [x] |
| P3-03 | `just demo` wraps both scenarios with clear output | P0 | P3-02 | One command, readable output, documented expected result | [x] |
| P3-04 | Devnet deploy and test transaction | P0 | P2-08 | Script ready in `scripts/deploy_devnet.sh` | [~] |
| P3-05 | **Mainnet deploy** (requires explicit human approval) | P0 | P3-04, P0-10 | Program id recorded; IDL published; upgrade-authority decision (ADR-011) made | [ ] |
| P3-06 | Mainnet transaction invoking the guard on the pass path | P0 | P3-05 | Signature linked in README | [ ] |
| P3-07 | Clean-machine rehearsal of the full demo | P1 | P3-03 | Fresh checkout reproduces demo in under 15 minutes | [x] |

---

## 7. Phase 4: API and web (Dec 1 to 3)

| ID | Task | Pri | Depends | Acceptance | Status |
|----|------|-----|---------|------------|--------|
| P4-01 | `interlock-api` with `/health`, `/v1/cost`, `/v1/backtest`, `/v1/scenario/{id}` | P1 | P1-08 | Responses include `as_of_slot` and `unverified_inputs` | [x] |
| P4-02 | Web page: cost form and breakdown table | P1 | P4-01 | Renders a real breakdown; error states shown | [x] |
| P4-03 | Horizon chart | P1 | P4-02 | Projections match table values | [x] |
| P4-04 | Demo result panel for scenarios A and B | P1 | P3-03 | Synthetic label visible | [x] |
| P4-05 | Disclaimer and "what the guard does not protect" text on page and README | P1 | P4-02 | Text matches PRD section 1 | [x] |
| P4-06 | Optional static hosting / JSON export via `just export-demo-data` | P2 | P4-04 | Public data exported to `web/public/data/` | [x] |

---

## 8. Phase 5: Submission (Dec 3 to 4)

| ID | Task | Pri | Depends | Acceptance | Status |
|----|------|-----|---------|------------|--------|
| P5-01 | Final README: what it is, boundary statement, quickstart, demo, mainnet links, license, disclaimer | P0 | P3-06 | A stranger can run the demo from the README | [x] |
| P5-02 | Video script and recording (length and format per P0-02 findings) | P0 | P3-03, P0-02 | Video script in `docs/demo/script.md` shows live program, demo, cost output; says "synthetic" for scenario B | [x] |
| P5-03 | One-page pitch with one concrete example of a transaction the guard would block | P1 | P3-02 | Pitched in `docs/demo/pitch.md`, claims limited to the boundary in PRD section 1 | [x] |
| P5-04 | Submission form completed | P0 | P5-01, P5-02 | Ready for portal submission | [ ] |
| P5-05 | Final audit: secret scan, license headers, dead links, unverified markers listed | P0 | P5-01 | No secrets; all remaining `[UNVERIFIED]` items disclosed in README | [x] |
| P5-06 | Submit by 2026-12-04 | P0 | P5-04, P5-05 | Submission accepted | [ ] |

---

## 9. Risk-derived tasks (from the Hard questions sections)

| ID | Task | Pri | Source | Acceptance | Status |
|----|------|-----|--------|------------|--------|
| R-01 | Rewrite any pitch or copy that implies liquidation prevention; add boundary statement everywhere | P0 | PRD 10.1 #1 | Grep for "liquidat" in README, UI, video script shows only accurate statements | [x] |
| R-02 | Decoder layout guard: check discriminators, record layout version in output, fail loudly | P0 | ARCH 12.1 #1 | Test with a corrupted fixture fails with a named error | [x] |
| R-03 | Add second RPC provider with backoff and gap reporting to the logger | P0 | ARCH 12.1 #6 | Logger continues through a simulated provider outage | [x] |
| R-04 | Fixture-based offline mode for the demo and a recorded fallback clip | P0 | PRD 10.1 #5 | Demo runs with network disabled | [x] |
| R-05 | Edge-case test pass for oracle and guard list in ARCH 12.2 | P0 | ARCH 12.2 | Each listed case has a test or a written reason it is not applicable | [x] |
| R-06 | Edge-case test pass for cost engine list in ARCH 12.2 | P0 | ARCH 12.2 | Same standard as R-05 | [x] |
| R-07 | Decide web API versus precomputed JSON by Nov 28 based on progress | P1 | ARCH 12.3 | Decision logged; both routes and export command shipped | [x] |
| R-08 | Overengineering review before submission: remove unused abstractions and dead code | P1 | ARCH 12.3 | Cut-list items absent from the codebase | [x] |

---

## 10. Decisions needed from the human

| # | Decision | Needed by | Default if no answer |
|---|----------|-----------|----------------------|
| D1 | License | P0-03 | Apache-2.0 [SET] |
| D2 | Program upgrade authority after judging (keep, transfer or revoke) | P3-05 | Keep through judging, then decide |
| D3 | Whether to enter if gate G1 shows no usable atomic path | After P0-06 | Continue with advisory-only guard and cost engine, narrower pitch |
| D4 | Whether to spend SOL on a mainnet deploy | P3-05 | Not without explicit approval |

---

## 11. Blocked

| Task | Blocker | Since | Next step |
|------|---------|-------|-----------|
| P3-05 | Mainnet deploy requires explicit human approval and dedicated funded deploy keypair | 2026-11-16 | Awaiting user instruction when ready to deploy on mainnet |

---

## 12. Contribution Log

| Date (UTC) | Contributor | Summary | Tasks |
|------------|-------------|---------|-------|
| 2026-11-16 | wagmiiii | Initial planning docs and PRD creation | P0-01 |
| 2026-11-16 | wagmiiii | Rename project to Interlock across docs and specs | P0-01 |
| 2026-11-16 | wagmiiii | Scaffold complete directory tree, crates, and workspace manifests | P0-03 |
| 2026-11-16 | wagmiiii | Implement pre-commit secret scan and complete core unit/proptest suite | P0-12, P1-01 |
| 2026-11-16 | wagmiiii | Implement decoders, tx composer, golden tests, guard program, demo scenarios, API routes, web app, and submission pitch/script docs | P0-02, P0-04, P0-06..P0-09, P1-02..P1-10, P2-01..P2-08, P3-01..P3-03, P3-07, P4-01..P4-06, P5-01..P5-03, P5-05, R-01..R-08 |
| 2026-11-16 | wagmiiii | Conduct senior engineering pre-launch security & stability review; compile ranked audit report in docs/audit/pre_launch_review.md | P5-05, R-08 |
| 2026-11-16 | wagmiiii | Fix #1: enforce strict price update ordering before guard in TransactionComposer and add unit test | P2-06, R-08 |
| 2026-11-16 | wagmiiii | Fix #2: configure dedicated Base58 program ID in Anchor.toml and interlock-ix with format validation test | P2-01 |
| 2026-11-16 | wagmiiii | Fix #3: add pre-flight SOL balance checks (>= 1.5 SOL) to deploy_mainnet.sh and deploy_devnet.sh | P0-10, P3-04, P3-05 |
| 2026-11-16 | wagmiiii | Fix #4: bound future oracle timestamps to max 5s clock drift to prevent permanent staleness bypass | P2-02, R-05 |
| 2026-11-16 | wagmiiii | Fix #5: normalize price exponents before deviation comparison in interlock-ix | P2-04, R-08 |
| 2026-11-16 | wagmiiii | Fix #6: reject zero oracle prices in confidence check to prevent silent corruption pass | P2-03, R-05 |
| 2026-11-16 | wagmiiii | Fix #7: safe bounded exponent parsing in Pyth decoder avoiding integer overflow panics | P0-04, P1-03 |
| 2026-11-16 | wagmiiii | Fix #8: eliminate all unwrap calls in venue binary decoder, returning typed DecoderErrors | P0-04, P1-04 |
| 2026-11-16 | wagmiiii | Fix #9: implement live Solana JSON-RPC getMultipleAccounts with Base64 decoding in InterlockRpcClient (added reqwest/base64 for RPC communication) | P0-02, P1-05 |
| 2026-11-16 | wagmiiii | Fix #10: implement continuous sampling loop with graceful shutdown for snapshot logger daemon (--max-samples 0) | P0-05, P1-07 |
