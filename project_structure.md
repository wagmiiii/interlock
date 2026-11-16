# Interlock: Project Structure

Where everything lives and where new files go. Planned layout, created in task P0-03. Keep this file in sync with the real tree.

> **Roadmap rule:** any contribution to this repo (including adding or moving files) must update `ROADMAP.md` in the same change.

## Tree

```
interlock/
├── AGENTS.md                      # working rules for all coding agents
├── CLAUDE.md                      # Claude Code entry point (imports AGENTS.md)
├── ROADMAP.md                     # task list and contribution log (update on every change)
├── PRD.md                         # what we build, for whom
├── ARCHITECTURE.md                # full design, data models, ADR table
├── ARCHITECTURE_ESSENTIALS.md     # short outline of critical decisions
├── project_structure.md           # this file
├── README.md                      # public-facing: what it is, quickstart, demo, disclaimer
├── LICENSE                        # decision pending (MIT proposed)
├── Cargo.toml                     # workspace root (crates + programs)
├── Anchor.toml                    # Anchor config
├── justfile                       # task runner: check, test, demo, fmt
├── rust-toolchain.toml            # pinned Rust version
├── .env.example                   # env var names only, no values
├── .gitignore                     # keys, .env, target, db files, node_modules
├── .github/
│   └── workflows/
│       └── ci.yml                 # fmt, clippy, test, anchor build (never mainnet)
├── crates/
│   ├── interlock-core/            # pure types and math, no IO
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs           # PositionInput, VenueParams, MarketSnapshot, CostBreakdown
│   │   │   ├── cost.rs            # cost model
│   │   │   ├── liquidation.rs     # liquidation distance model
│   │   │   └── backtest.rs        # window replay and percentile math
│   │   └── tests/                 # unit and proptest
│   ├── interlock-chain/           # RPC reads and decoders (read-only)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── rpc.rs             # slot-consistent getMultipleAccounts wrapper
│   │   │   ├── venue.rs           # venue account decoders
│   │   │   └── pyth.rs            # Pyth price reads
│   │   └── tests/fixtures/        # captured account bytes with capture slot in filename
│   ├── interlock-store/           # SQLite snapshots
│   │   └── src/{lib.rs,schema.rs,writer.rs,coverage.rs}
│   ├── interlock-cli/             # binary: cost, snapshot, backtest, guard
│   │   └── src/{main.rs,cmd/}
│   └── interlock-api/             # optional axum layer
│       └── src/{main.rs,routes.rs}
├── programs/
│   └── interlock-ix/              # Anchor program (single instruction)
│       ├── src/{lib.rs,errors.rs,checks.rs,reference.rs}
│       └── tests/                 # LiteSVM tests
├── scenarios/
│   ├── README.md                  # how scenarios work and how to rerun
│   ├── a_normal.*                 # Surfpool scenario: guarded tx passes
│   ├── b_spike_synthetic.*        # Surfpool scenario: guarded tx reverts (SYNTHETIC)
│   └── fixtures/
├── web/                           # Vite + React + TypeScript (strict)
│   ├── src/{main.tsx,App.tsx,components/,lib/}
│   └── public/data/               # precomputed JSON if the API is cut
├── scripts/
│   ├── install_toolchain.sh       # pinned installs
│   ├── demo.sh                    # called by `just demo`
│   ├── deploy_devnet.sh           # manual, requires human-supplied keypair path
│   └── deploy_mainnet.sh          # manual, requires explicit human approval
├── data/                          # local SQLite files (gitignored)
└── docs/
    ├── audit/                     # senior engineering & security pre-launch review
    ├── verification/              # one short note per resolved [UNVERIFIED] item
    └── demo/                      # video script, pitch one-pager, screenshots
```

## Placement rules

| You are adding | Put it in |
|----------------|-----------|
| Pure math or types | `crates/interlock-core` |
| Anything that talks to RPC | `crates/interlock-chain` |
| Anything that reads or writes SQLite | `crates/interlock-store` |
| A CLI command | `crates/interlock-cli/src/cmd/<name>.rs` |
| On-chain logic | `programs/interlock-ix/src` |
| Captured on-chain data for tests | `crates/interlock-chain/tests/fixtures/` (filename includes slot and source) |
| A demo scenario | `scenarios/` |
| Resolution of an [UNVERIFIED] item | `docs/verification/<id>.md`, then update the doc that carried the marker |
| Pitch or video material | `docs/demo/` |

## Naming and conventions

- Crates: `interlock-<role>` (and `interlock-ix` for the on-chain program). Rust modules: `snake_case`. TypeScript files: `camelCase.ts` or `PascalCase.tsx` for components.
- Fixtures: `<account-kind>_<slot>_<yyyymmdd>.bin`.
- Branches: `feat/<task-id>-short-name`, `fix/<task-id>-short-name`, `docs/<task-id>-short-name`.
- Task ids (`P1-03`, `R-02`, `G1`) from `ROADMAP.md` appear in commit messages and PR titles.

## Never commit

Private keys or keypair files, `.env`, SQLite database files, RPC URLs containing API keys, `target/`, `node_modules/`.

## Generated files

- Anchor IDL: generated into `target/idl/` and copied to `web/src/lib/idl/` by `just idl`. Do not hand-edit.
- Precomputed demo JSON in `web/public/data/` is generated by `just export-demo-data`. Do not hand-edit.

---

> **Roadmap rule:** any contribution to this repo must update `ROADMAP.md` in the same change.
