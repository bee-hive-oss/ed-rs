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
| 2026-11-25 16:54 | `a9574a1` | 102 pass | clean | green | 0/20 | 86/243 | through ticket #69 (PRs #92 to #119 merged Nov 25); binary still stops at stdin read (ticket #33 open) |
| 2026-11-25 17:23 | `1a1289b` | 117 pass | clean | green | 10/20 | 86/243 | through ticket #41 (PRs #120 to #132 merged Nov 25, input wiring); binary now reads and runs commands, GNU suite waits on r/w/s/g/u (tickets #70 to #91) |
| 2026-11-25 17:31 | `8fc9eb2` | 123 pass | clean | green | 15/20 | 111/243 | through ticket #80 (PRs #133 to #144 merged Nov 25: E, e, g, v, G, V, h, H, j, k, m) |
| 2026-11-25 17:44 | `339eecd` | 124 pass | clean | green | 20/20 | 170/243 | through ticket #88 (PRs #145 to #153 merged Nov 25: P, r, s, t, u, w, W, x) |
| 2026-11-25 18:11 | `f624c00` | 125 pass | clean | green | 20/20 | 182/243 | through ticket #91 (PRs #155, #157, #159 merged Nov 25: z, !, null command) |
| 2026-11-25 20:51 | `37b9b6e` | 125 pass | clean | green | 20/20 | 242/243 | through ticket #165 (PRs #168, #169, and #171 merged Nov 25: piped errors, start-up, global list; PR #170 unbuffered stdin left open, conflicts with #169 on startup.rs; wrapper counts start-up checks #167) |
| 2026-11-25 21:00 | `99f724f` | 125 pass | clean | green | 20/20 | 243/243 | through ticket #165 plus #170 unbuffered stdin (PRs #168-#171 merged Nov 25) |

Coverage columns added 2026-11-26. New rows include C lines %, C branches %, Rust lines %, and Rust branches % from the GitHub coverage run on that commit (Rust figures are the GNU test suite alone). Earlier rows keep their original columns.

| Date (CT) | Commit | Cargo tests | Clippy | Self-check | Starter cases | GNU ed testsuite | C lines % | C branches % | Rust lines % | Rust branches % | Note |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 2026-11-26 18:19 | `37f10d0` | 126 pass | clean | green | 20/20 | 243/243 | 84.2% | 67.4% | 79.7% | 66.2% | PR #185 merged Nov 26: Diagnose empty and trailing-slash names as Invalid filename |
| 2026-11-26 18:23 | `3dc7c25` | 127 pass | clean | green | 20/20 | 243/243 | 84.2% | 67.4% | 79.7% | 66.2% | PR #186 merged Nov 26: Print backslashes in f output without doubling them |
| 2026-11-26 18:49 | `992e33d` | 127 pass | clean | green | 20/20 | 243/243 | 84.2% | 67.4% | 79.7% | 66.2% | PR #187 merged Nov 26: Add the shared repo-index.toml |

## Upstream C bugs fixed

Bugs in the original C tool (the same pinned GNU version as the table
above) that this Rust port has fixed. A bug counts only when a test
fails on the real GNU tool and passes on the Rust port. Link the
upstream bug report when one exists. Rows are only ever added at the
bottom, never edited or removed.

Count so far: 0.

| Date (CT) | Commit | GNU version | Bug | Proof test | Upstream report |
|---|---|---|---|---|---|
