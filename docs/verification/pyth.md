# Verification: Pyth Feed Identifiers (Task P0-09)

- **Date Verified:** 2026-11-16
- **Network:** Solana Mainnet-Beta / Pyth Receiver Program
- **Primary Feeds:**
  - **SOL/USD:** `0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d`
  - **BTC/USD:** `0xe62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43`
  - **ETH/USD:** `0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace`
- **Validation Rules:**
  - `verification_level`: Full verification required (`VerificationLevel::Full`).
  - `max_age_secs`: Maximum allowed price staleness in seconds.
  - `max_conf_bps`: Confidence interval width relative to price in basis points.
