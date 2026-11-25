# ed-rs scoreboard

How far the Rust port is, measured after merges to `main`. Rows are only ever
added at the bottom, never edited or removed, so the history stays readable.

Each row is measured in the same sealed rootless Podman image the hive runner
uses (GNU ed 1.22.4, no network, source read-only):

- **Cargo tests**: unit tests passing in `cargo test`.
- **Clippy**: `cargo clippy --all-targets -- -D warnings`.
- **Self-check**: `./check.sh --self-check`, which proves the harness itself
  against real GNU ed (243/243 means the yardstick is sound).
- **Starter cases**: `tests/diff_harness.py`, the 20 cases in `tests/cases/`,
  ed-rs output compared byte for byte with GNU ed.
- **GNU ed testsuite**: the official GNU ed 1.22.4 suite run against
  `target/release/ed-rs`. `./check.sh` stops after the starter cases when
  any fail, so this column runs the suite directly.

Make a row with `python3 -m hive_runner.scoreboard --clone .` from the
hive-runner repo, then append it below.

| Date (CT) | Commit | Cargo tests | Clippy | Self-check | Starter cases | GNU ed testsuite | Note |
|---|---|---|---|---|---|---|---|
| 2026-11-20 11:23 | `3f744a0` | 37 pass | clean | green | 0/20 | 86/243 | buffer and address (#4), one-command stubs (#13) |
| 2026-11-25 12:28 | `aa45d12` | 89 pass | clean | green | 0/20 | 86/243 | #21 q/Q, #22 =, #23 f, #31 Nori's starter files, #24 d, #25 p, #27 n, #28 l, #29 a, #30 i |
