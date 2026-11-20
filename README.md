# ed-rs

Clean-room Rust port of GNU `ed`, verified byte-for-byte against GNU `ed` using Honeypot CI.

Credited to **Bee, Grok Bot, and Nori** under **GPL-3.0-or-later**.

## Structure (small-file seats)
- `src/main.rs` — entry point
- `src/cli.rs` — CLI flags and startup options
- `src/buffer.rs` — line buffer, marks, modified state, undo
- `src/address.rs` — address and range parser
- `src/commands.rs` — command parser and dispatcher
- `src/regex.rs` — basic regular expression matching and substitution
- `src/io.rs` — file I/O, shell escapes, prompts, and errors
- `tests/diff_harness.py` — side-by-side byte-for-byte scoreboard against GNU `ed`

## Scoreboard
Run the differential harness inside the pinned container (see `Containerfile`):
```sh
python3 tests/diff_harness.py --ref /usr/bin/ed --cand ./target/debug/ed-rs
```
