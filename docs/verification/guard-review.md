# Security Self-Review Checklist: `interlock-ix` (Task P2-08)

- **Date:** 2026-11-16
- **Program Name:** `interlock-ix`
- **Instruction:** `assert_oracle_sane`

## Checklist

- [x] **Stateless:** Program creates no accounts, stores no state, and writes to no PDAs.
- [x] **No lamport handling:** No lamports are debited, credited, or transferred.
- [x] **No CPI execution:** Instruction performs pure validation in handler.
- [x] **Checked integer arithmetic:** All calculations use checked math (`checked_sub`, `checked_mul`) with `u128` cross-multiplication to eliminate overflow and truncation issues.
- [x] **Owner & program id validation:** Ensures price feed account is owned by the designated Pyth Receiver program.
- [x] **Feed id match:** Confirms the account data feed id matches the caller's expected feed id.
- [x] **Staleness check:** Validates current slot timestamp against publish time within `max_age_secs`.
- [x] **Confidence bounds:** Validates confidence interval against `max_conf_bps`.
- [x] **Reference deviation check:** Compares primary oracle price with reference price using `max_deviation_bps`.
- [x] **Dedicated error variants:** Every failure condition returns a unique `GuardError` code.
