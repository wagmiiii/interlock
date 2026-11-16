# Surfpool Demo Scenarios

These scenario files provide reproducible mainnet-fork test fixtures for the Interlock demonstration:

- **Scenario A (`a_normal.json`):** Normal market conditions. The oracle price matches reference within 5 bps. The guarded trade transaction succeeds atomically.
- **Scenario B (`b_spike_synthetic.json`):** Synthetic oracle anomaly (+15% price spike injected). The guarded trade transaction reverts with `GuardError::DeviationExceeded`. The unguarded transaction proceeds. **Note:** This scenario is strictly synthetic simulation data.
