# Interlock: Demo Video Script (3-Minute Walkthrough)

**Target Duration:** 2 minutes 45 seconds to 3 minutes.  
**Audience:** Solana Perps and Prediction Markets Hackathon Judges, traders, and protocol developers.

---

## [00:00 - 00:30] The Problem: Blind Execution on Solana Perps

**Visual:** Terminal showing an attempted order during an oracle anomaly, followed by an unexpectedly large liquidation or instantaneous loss.

**Speaker:**
"On Solana perpetuals, opening a leveraged position often feels like signing a blank check. 
You sign a transaction with your desired collateral and leverage, but between your wallet and execution, two invisible hazards eat your capital:
First, undisclosed holding costs: borrow fees fluctuate with pool utilization, and holding fees can eat double-digit percentages of your collateral before you break even.
Second, oracle latency and spikes: if Pyth has a momentary divergence, a wide confidence interval, or a delayed publisher slot, your transaction executes at an unfair mark price, putting you underwater in slot zero.
That's why we built **Interlock**."

---

## [00:30 - 01:15] Component 1: Non-Custodial Cost Engine (`interlock cost`)

**Visual:** Terminal executing `cargo run -p interlock-cli -- cost --market SOL --side long --collateral 1000 --leverage 5 --hours 24`.

**Speaker:**
"Interlock starts with a non-custodial cost engine. 
Using slot-consistent on-chain reads, Interlock decodes venue parameters and computes the full itemised cost of your trade: open and close fees, price impact, and projected borrow fees across time.
Notice our output:
It explicitly shows the `as_of_slot`, the required breakeven price move—here 0.14%—and a 168-hour horizon sweep alongside sensitivity scenarios for 0.5x, 1x, and 2x borrow rate shifts.
It also provides estimated liquidation distance, warning you when your collateral buffer is too thin.
Every assumption and unverified parameter is stated directly in the terminal."

---

## [01:15 - 02:15] Component 2: The Atomic Oracle Guard (`interlock-ix`)

**Visual:** Split screen showing the transaction structure: `[ComputeBudget] [assert_oracle_sane] [Venue::open_position]`.

**Speaker:**
"Now for execution protection. An 'interlock' is a mechanical switch that prevents a machine from firing if operating parameters are unsafe.
Our stateless Anchor program, `interlock-ix`, provides an instruction called `assert_oracle_sane`.
You prepend this instruction directly into your trade transaction before the venue's order instruction.
Inside the Solana runtime, it performs pure `u128` integer cross-multiplication:
1. It validates the Pyth publisher feed id and demands full verification level.
2. It rejects stale prices older than your maximum age parameter.
3. It rejects wide confidence intervals.
4. And critically, it checks price deviation against an independent reference source.

If the oracle is sane, the guard completes in under 2,000 compute units and your order fills seamlessly.
If the oracle is corrupted, the guard aborts the entire transaction atomically. Your trade is NEVER submitted to the venue engine."

---

## [02:15 - 02:40] Component 3: Live Replay Demo (Scenarios A and B)

**Visual:** Terminal executing `bash scripts/demo.sh` or `just demo`.

**Speaker:**
"Let's run `just demo`.
In **Scenario A**, under normal market conditions with a 3 bps deviation between Pyth and the reference price, the guard passes and the trade executes normally.
In **Scenario B**, we simulate a synthetic 15% oracle price spike—explicitly labelled synthetic.
Watch the result: instruction 1 fails with `GuardError::DeviationExceeded`.
Because Solana transactions are strictly atomic, instruction 2—the venue order—is never executed. 
Without Interlock, the user would have bought at $172.50 into an immediate loss.
With Interlock, 100% of user collateral is preserved, costing only ~5,000 lamports for the failed transaction fee."

---

## [02:40 - 03:00] Boundary Statement & Conclusion

**Visual:** Web interface at `web/` showing the cost calculator and boundary notice.

**Speaker:**
"A clear word on our boundary: Interlock protects user-signed transactions only. It is non-custodial and read-only. It cannot and does not stop third-party keeper liquidations on existing open positions.
Interlock is fully open source under the Apache-2.0 license, completely tested with proptest and LiteSVM, and ready for integration into Solana trading bots, frontends, and venue SDKs.
Thank you."
