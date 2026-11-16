# Interlock: Roadmap

**Last updated:** 2026-11-16 (UTC) by wagmiiii: scaffolded repository structure, data models, and justfile (P0-03)
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
| 0. Foundations and gates | Nov 16 to 18 | 12 | 2 | 17% |
| 1. Cost engine and logger | Nov 19 to 25 | 10 | 0 | 0% |
| 2. Guard program | Nov 24 to 30 | 9 | 0 | 0% |
| 3. Integration, demo, mainnet | Nov 29 to Dec 2 | 7 | 0 | 0% |
| 4. API and web | Dec 1 to 3 | 6 | 0 | 0% |
| 5. Submission | Dec 3 to 4 | 6 | 0 | 0% |
| R. Risk-derived tasks | ongoing | 8 | 0 | 0% |
| **All tasks** | | **58** | **2** | **3%** |

**100% readiness** means every P0 and P1 task is done (56 tasks; the 2 P2 tasks are excluded). Current readiness: 2 of 56 (4%).

## 2. Schedule at a glance

```
Nov 16-18  Phase 0  gates G1-G3, toolchain, repo skeleton, RPC claim
Nov 19-25  Phase 1  cost engine; START SNAPSHOT LOGGER BY NOV 20 (data accrues with time)
Nov 24-30  Phase 2  guard program and tests (overlaps Phase 1 on purpose)
Nov 29-Dec 2  Phase 3  Surfpool scenarios, devnet then mainnet deploy
Dec 1-3    Phase 4  API and web (cut first if behind)
Dec 3-4    Phase 5  README, video, submission. Submit Dec 4.
Dec 5      buffer
```

Checkpoint rule: if Phase 1 is not P0-complete by Nov 26, cut Phase 4 to precomputed JSON and drop P1 items in Phase 1. If the guard is not on devnet by Nov 30, drop everything except Phase 2, 3 and 5.

## 3. Phase 0: Foundations and gates (Nov 16 to 18)

| ID | Task | Pri | Depends | Acceptance | Status |
|----|------|-----|---------|------------|--------|
| P0-01 | Create planning docs (PRD, ARCHITECTURE, ESSENTIALS, project_structure, AGENTS, CLAUDE, ROADMAP) | P0 | none | All seven files exist and cross-reference each other | [x] |
| P0-02 | Confirm hackathon start date, rules, FAQ, submission format, video requirement, Nigerian payout and KYC eligibility via the hackathon site and Discord | P0 | none | Answers recorded in `docs/verification/hackathon.md`; PRD assumptions A1, A3, A7 updated | [ ] |
| P0-03 | Create repo skeleton per `project_structure.md`; decide license (MIT proposed); add `.gitignore`, `.env.example`, `justfile` stubs | P0 | none | `just check` runs (even if empty); license file committed; secret patterns ignored | [x] |
| P0-04 | Install and pin toolchain (Rust, Solana CLI or Agave, Anchor, Surfpool, LiteSVM, Node, just); record versions in `ARCHITECTURE.md` section 4 | P0 | P0-03 | `scripts/install_toolchain.sh` works; versions table filled; LiteSVM tested on the dev OS | [ ] |
| P0-05 | Claim the free OrbitFlare RPC plan when the hackathon starts; set a second fallback provider | P0 | P0-02 | `.env` (local only) has two working RPC URLs; claim date noted (plan lasts one week) | [ ] |
| P0-06 | **Gate G1:** determine how the venue fills orders (atomic versus request-then-keeper) from IDL, docs and a simulated transaction | P0 | P0-04 | `docs/verification/g1.md` written; ADR-004 set to Accepted with chosen mode; ESSENTIALS updated | [ ] |
| P0-07 | **Gate G2:** choose the reference price source and verify its account layout and liquidity | P0 | P0-04 | `docs/verification/g2.md` written; ADR-005 Accepted; fixture captured | [ ] |
| P0-08 | **Gate G3:** decode venue pool and market accounts; confirm how fees and borrow rate are represented; check the flat-rate formula against the venue UI | P0 | P0-04 | `docs/verification/g3.md` written; formula in `ARCHITECTURE.md` 6.1 marked verified or corrected | [ ] |
| P0-09 | Confirm Pyth feed ids for target markets and the price-update posting approach | P0 | P0-04 | Feed ids and method recorded in `docs/verification/pyth.md` | [ ] |
| P0-10 | Estimate mainnet deploy cost; create a dedicated deploy keypair outside the repo; fund minimally | P0 | P0-04 | Estimate recorded; keypair path never committed; balance checked | [ ] |
| P0-11 | CI baseline (fmt, clippy, tests, `anchor build`) | P1 | P0-03 | Workflow green on an empty skeleton | [ ] |
| P0-12 | Pre-commit secret scan | P0 | P0-03 | Committing a fake key string is blocked locally | [ ] |

**Gate outcomes drive the plan.** If G1 shows no atomic path on any usable venue, the guard becomes advisory for that venue: record it in ADR-004, rewrite the PRD guard claim and the pitch before continuing.

## 4. Phase 1: Cost engine and logger (Nov 19 to 25)

| ID | Task | Pri | Depends | Acceptance | Status |
|----|------|-----|---------|------------|--------|
| P1-01 | `interlock-core`: types and cost model with `Decimal`, unit tests | P0 | P0-08 | Tests cover fee math, zero and fractional hours, leverage bounds | [ ] |
| P1-02 | `interlock-chain`: slot-consistent RPC wrapper; decoders for venue accounts and Pyth, with fixtures | P0 | P0-06, P0-08, P0-09 | Decoders pass on committed fixtures; unknown layout returns a named error | [ ] |
| P1-03 | `interlock cost` CLI command end to end | P0 | P1-01, P1-02 | PRD F1 first acceptance bullet met; prints slot, assumptions, unverified inputs | [ ] |
| P1-04 | Golden tests against venue quotes (at least 5 positions) | P0 | P1-03 | Total cost within 1% relative or $0.05 absolute, or gaps documented | [ ] |
| P1-05 | `interlock-store` schema and `snapshot run` logger (**start by Nov 20**) | P0 | P0-08, P1-02 | Runs unattended; survives RPC failures; no partial rows; run recorded in `logger_runs` | [ ] |
| P1-06 | `snapshot report`: daily coverage and gap list | P1 | P1-05 | Gaps over 5 minutes listed; coverage percent per day | [ ] |
| P1-07 | `interlock backtest` with 90% coverage refusal rule | P1 | P1-05, P1-06 | Min, median, p95, max realised borrow cost versus entry-rate cost; refuses when coverage is below 90% | [ ] |
| P1-08 | Horizon sweep (1h, 24h, 72h, 168h) and rate sensitivity rows (x0.5, x1, x2) | P1 | P1-03 | Included in CLI and JSON output | [ ] |
| P1-09 | Liquidation distance model using the venue's real rule where decoded | P1 | P0-08, P1-02 | Matches venue UI on test positions or is labelled approximate | [ ] |
| P1-10 | Doc sync pass for Phase 1 (ARCHITECTURE, ESSENTIALS, PRD acceptance wording) | P1 | P1-04 | Docs match shipped behaviour | [ ] |

## 5. Phase 2: Guard program (Nov 24 to 30)

| ID | Task | Pri | Depends | Acceptance | Status |
|----|------|-----|---------|------------|--------|
| P2-01 | Anchor scaffold (`interlock-ix`), workspace wiring, devnet config | P0 | P0-04 | `anchor build` succeeds in CI | [ ] |
| P2-02 | Pyth checks: age, confidence, feed id, full verification | P0 | P0-09, P2-01 | Each failure returns its own error variant | [ ] |
| P2-03 | Reference deviation check with `u128` cross-multiplication | P0 | P0-07, P2-01 | Matches big-integer reference in property tests | [ ] |
| P2-04 | Error codes and success log line | P0 | P2-02, P2-03 | All `GuardError` variants reachable | [ ] |
| P2-05 | LiteSVM tests: every error path, pass path, wrong owner, wrong feed, orientation inverted | P0 | P2-04 | All tests green; list of covered paths in test file header | [ ] |
| P2-06 | Transaction composition helper that enforces instruction order and maps error codes to messages | P0 | P2-04 | Refuses to build a transaction with the guard after the venue instruction | [ ] |
| P2-07 | Measure compute units and transaction size overhead; recheck packet and compute limits | P0 | P2-06 | Numbers recorded in `ARCHITECTURE.md` section 7; lookup-table guidance written | [ ] |
| P2-08 | Security self-review checklist (owners, ids, overflow, no state, no CPI) | P0 | P2-05 | Checklist in `docs/verification/guard-review.md`, all items ticked or explained | [ ] |
| P2-09 | Stretch: on-chain utilization check | P2 | P0-08, P2-05 | Only attempted after all P0 and P1 tasks pass | [ ] |

## 6. Phase 3: Integration, demo and mainnet (Nov 29 to Dec 2)

| ID | Task | Pri | Depends | Acceptance | Status |
|----|------|-----|---------|------------|--------|
| P3-01 | Surfpool scenario A: normal conditions, guarded transaction passes | P0 | P2-06 | Reproducible on a fork; output captured | [ ] |
| P3-02 | Surfpool scenario B: synthetic oracle spike, guarded transaction reverts, unguarded proceeds; labelled synthetic | P0 | P3-01 | Output shows the named error and the synthetic label | [ ] |
| P3-03 | `just demo` wraps both scenarios with clear output | P0 | P3-02 | One command, readable output, documented expected result | [ ] |
| P3-04 | Devnet deploy and test transaction | P0 | P2-08 | Program id and signature recorded | [ ] |
| P3-05 | **Mainnet deploy** (requires explicit human approval) | P0 | P3-04, P0-10 | Program id recorded; IDL published; upgrade-authority decision (ADR-011) made | [ ] |
| P3-06 | Mainnet transaction invoking the guard on the pass path | P0 | P3-05 | Signature linked in README | [ ] |
| P3-07 | Clean-machine rehearsal of the full demo | P1 | P3-03 | Fresh checkout reproduces demo in under 15 minutes | [ ] |

## 7. Phase 4: API and web (Dec 1 to 3, first to be cut)

| ID | Task | Pri | Depends | Acceptance | Status |
|----|------|-----|---------|------------|--------|
| P4-01 | `interlock-api` with `/health`, `/v1/cost`, `/v1/backtest`, `/v1/scenario/{id}` | P1 | P1-08 | Responses include `as_of_slot` and `unverified_inputs` | [ ] |
| P4-02 | Web page: cost form and breakdown table | P1 | P4-01 | Renders a real breakdown; error states shown | [ ] |
| P4-03 | Horizon chart | P1 | P4-02 | Chart matches table values | [ ] |
| P4-04 | Demo result panel for scenarios A and B | P1 | P3-03 | Synthetic label visible | [ ] |
| P4-05 | Disclaimer and "what the guard does not protect" text on page and README | P1 | P4-02 | Text matches PRD section 1 | [ ] |
| P4-06 | Optional static hosting | P2 | P4-04 | Public URL in README | [ ] |

Fallback: if cut, export JSON via `just export-demo-data` and serve from `web/public/data/`, or ship CLI plus video only.

## 8. Phase 5: Submission (Dec 3 to 4)

| ID | Task | Pri | Depends | Acceptance | Status |
|----|------|-----|---------|------------|--------|
| P5-01 | Final README: what it is, boundary statement, quickstart, demo, mainnet links, license, disclaimer | P0 | P3-06 | A stranger can run the demo from the README | [ ] |
| P5-02 | Video script and recording (length and format per P0-02 findings) | P0 | P3-03, P0-02 | Video shows live program, demo, cost output; says "synthetic" for scenario B | [ ] |
| P5-03 | One-page pitch with one concrete example of a transaction the guard would block | P1 | P3-02 | Claims limited to the boundary in PRD section 1 | [ ] |
| P5-04 | Submission form completed | P0 | P5-01, P5-02 | Confirmation saved | [ ] |
| P5-05 | Final audit: secret scan, license headers, dead links, unverified markers listed | P0 | P5-01 | No secrets; all remaining `[UNVERIFIED]` items disclosed in README | [ ] |
| P5-06 | Submit by 2026-12-04 | P0 | P5-04, P5-05 | Submission accepted | [ ] |

## 9. Risk-derived tasks (from the Hard questions sections)

| ID | Task | Pri | Source | Acceptance | Status |
|----|------|-----|--------|------------|--------|
| R-01 | Rewrite any pitch or copy that implies liquidation prevention; add boundary statement everywhere | P0 | PRD 10.1 #1 | Grep for "liquidat" in README, UI, video script shows only accurate statements | [ ] |
| R-02 | Decoder layout guard: check discriminators, record layout version in output, fail loudly | P0 | ARCH 12.1 #1 | Test with a corrupted fixture fails with a named error | [ ] |
| R-03 | Add second RPC provider with backoff and gap reporting to the logger | P0 | ARCH 12.1 #6 | Logger continues through a simulated provider outage | [ ] |
| R-04 | Fixture-based offline mode for the demo and a recorded fallback clip | P0 | PRD 10.1 #5 | Demo runs with network disabled | [ ] |
| R-05 | Edge-case test pass for oracle and guard list in ARCH 12.2 | P0 | ARCH 12.2 | Each listed case has a test or a written reason it is not applicable | [ ] |
| R-06 | Edge-case test pass for cost engine list in ARCH 12.2 | P0 | ARCH 12.2 | Same standard as R-05 | [ ] |
| R-07 | Decide web API versus precomputed JSON by Nov 28 based on progress | P1 | ARCH 12.3 | Decision logged; unused code removed | [ ] |
| R-08 | Overengineering review before submission: remove unused abstractions and dead code | P1 | ARCH 12.3 | Cut-list items absent from the codebase | [ ] |

## 10. Decisions needed from the human

| # | Decision | Needed by | Default if no answer |
|---|----------|-----------|----------------------|
| D1 | License | P0-03 | MIT |
| D2 | Program upgrade authority after judging (keep, transfer or revoke) | P3-05 | Keep through judging, then decide |
| D3 | Whether to enter if gate G1 shows no usable atomic path | After P0-06 | Continue with advisory-only guard and cost engine, narrower pitch |
| D4 | Whether to spend SOL on a mainnet deploy | P3-05 | Not without explicit approval |

## 11. Blocked

| Task | Blocker | Since | Next step |
|------|---------|-------|-----------|
| none | | | |

## 12. Definition of 100% readiness

Every item must be true:

- [ ] All P0 and P1 tasks above are `[x]`.
- [ ] Guard program live on mainnet with program id and a pass-path transaction in the README.
- [ ] Every `GuardError` path covered by a LiteSVM test.
- [ ] Cost engine golden tests documented (at least 5 positions) with the measured error.
- [ ] Snapshot coverage report published with sample size and gaps.
- [ ] `just demo` reproduces scenarios A and B on a clean machine in under 15 minutes.
- [ ] Scenario B labelled synthetic in code, output, README and video.
- [ ] No claim of liquidation prevention anywhere.
- [ ] All `[UNVERIFIED]` items resolved or disclosed in the README.
- [ ] No secrets, keys or keyed URLs in the repo history.
- [ ] Docs in sync: PRD, ARCHITECTURE, ESSENTIALS, project_structure, AGENTS, CLAUDE, ROADMAP.
- [ ] Hard-question sections reviewed after the last feature change.
- [ ] Submission confirmed before 2026-12-04 end of day UTC.

## 13. Hard questions for this roadmap

**What would break?**
- Phase 1 depends on gates that may not resolve cleanly. Three gates in the first two days is optimistic for a first-time Solana-program developer. If gates slip past Nov 19, cut P1 items in Phase 1 immediately.
- Phases 1 and 2 overlap. Solo, that means context switching. If the logger is running, avoid touching decoders while it records, or restarts will create gaps.
- The free RPC plan lasts one week from claim. If the claim happens at the start of the hackathon, it may expire before the final demo recording. Plan a fallback before then.
- Mainnet deployment lands on Nov 30 to Dec 2, leaving little slack for a deploy failure. Rehearse on devnet earlier.

**What edge cases are we missing?**
- Hackathon start date unknown (P0-02): the Dec 5 deadline might close earlier in some timezones. Submitting Dec 4 is the mitigation.
- The logger needs at least 7 days of data (PRD S3), so it must start by about Nov 27 at the latest; Nov 20 is the target.
- The demo video might have a length or format requirement that changes P5-02.

**What is overengineered?**
- Fifty-eight tasks for a three-week solo project is a lot of bookkeeping. Keep updates to a few lines per task; do not turn this file into a diary.
- Phase 4 may not be worth doing at all if Phase 3 slips. It is the first thing to cut.
- The R-section partly duplicates tests; if R-05 and R-06 add no new cases beyond the tests already written, mark them done and move on.

## 14. Contribution log

| Date (UTC) | Contributor | Summary | Tasks |
|------------|-------------|---------|-------|
| 2026-11-16 | wagmiiii | Scaffolding complete: directory tree per project_structure.md, core data models (interlock-core), RPC & decoder stubs (interlock-chain), store schema & writer (interlock-store), CLI commands (interlock-cli), API stubs (interlock-api), guard checks (interlock-ix), web app skeleton, scenarios A & B, justfile, .env.example, CI. just check passes. | P0-03 |
| 2026-11-16 | Claude (for Ademola) | Renamed project from PerpGuard to Interlock across documentation; updated crate/program identifiers (interlock-core, interlock-chain, interlock-store, interlock-cli, interlock-api, interlock-ix) and defined Interlock concept. | P0-01 |
| 2026-11-16 | Claude (for Ademola) | Created planning doc set: PRD, ARCHITECTURE, ARCHITECTURE_ESSENTIALS, project_structure, AGENTS, CLAUDE, ROADMAP. Added hard-question sections. | P0-01 |

---

> **Reminder:** every contribution updates this file.
