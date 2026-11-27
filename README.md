# ed-rs

Clean-room Rust port of GNU `ed`, verified byte-for-byte against GNU `ed` using Honeypot CI.

Credited to **Bee, Grok Bot, and Nori** under **GPL-3.0-or-later**.

## Status: 243/243

ed-rs passes **all 243 checks of the official GNU `ed` 1.22.6 test suite**, plus the 20 starter cases byte for byte against GNU `ed`, with `cargo test` and `cargo clippy -D warnings` clean.

Progress is tracked in [SCOREBOARD.md](SCOREBOARD.md), one row per measurement after merges to `main`, rows only ever appended. It went from the first stubs to 182/243 to 242/243 and then 243/243 on Nov 25, 2026.

The public GitHub history is an email-only mailmap rewrite of the private Forgejo history, so commit IDs differ from those in SCOREBOARD.md; code and messages are unchanged.

## Support

- Buy Me a Coffee: https://buymeacoffee.com/bee805
- X: https://x.com/bee__805

## Structure (small-file seats)
- `src/main.rs` — entry point
- `src/cli.rs` — CLI flags and startup options
- `src/address.rs` — address and range parser
- `src/buffer.rs` — line buffer, marks, modified state, undo
- `src/commands.rs` — command parser and dispatcher
- `src/regex.rs` — basic regular expression matching and substitution
- `src/io.rs` — file I/O, shell escapes, prompts, and errors
- `tests/diff_harness.py` — side-by-side byte-for-byte scoreboard against GNU `ed`
- `tests/gnu_ed_testsuite/` — official GNU `ed` 1.22.6 testsuite (GPL-2.0-or-later; unmodified)
- `AGENTS.md` — seat map and contribution rules

## Proofs

The single entry point is `./check.sh`. It runs `cargo test`, `cargo clippy --all-targets -- -D warnings`, the 20 starter cases in `tests/diff_harness.py`, and the official GNU `ed` 1.22.6 testsuite.

```sh
./check.sh                 # build ed-rs and prove it against GNU ed
./check.sh --self-check    # prove the harnesses against /usr/bin/ed
```

If `cargo` is not on `PATH`, `./check.sh` runs the cargo steps in a rootless `podman` container built from `Containerfile`. No sudo.

You can also run the harnesses directly:

```sh
python3 tests/diff_harness.py --ref /usr/bin/ed --cand ./target/release/ed-rs
ED_BIN=./target/release/ed-rs tests/run_gnu_testsuite.sh
tests/run_gnu_testsuite.sh --self-check
```
