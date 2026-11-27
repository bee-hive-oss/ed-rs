# Agent guide for ed-rs

Clean-room Rust port of GNU `ed`, verified byte-for-byte against GNU `ed` 1.22.4.
License: **GPL-3.0-or-later**. Credited to **Bee, Grok Bot, and Nori**.

The imported official GNU `ed` 1.22.4 testsuite under `tests/gnu_ed_testsuite/`
remains **GPL-2.0-or-later** (see `tests/gnu_ed_testsuite/COPYING`).

## Seat files

One seat file owns one concern. Edit only the file(s) your ticket names.

| File | Owns |
|------|------|
| `src/main.rs` | Entry point; wires CLI options into the command loop and process exit |
| `src/cli.rs` | CLI flags and startup options (`-s`, `-p`, `-G`, `-l`, `-v`, `-r`, `-E`, `-h`, `-V`, optional `[FILE]`, and GNU long options) |
| `src/buffer.rs` | In-memory line buffer (1-indexed lines, current line `.`, last line `$`, marks `a`–`z`, dirty flag, single-level undo snapshot) |
| `src/address.rs` | POSIX/GNU `ed` address and range parser (`.`, `$`, numbers, `+`/`-`, `,`, `;`, `'x`, relative offsets, ranges) |
| `src/commands.rs` | Command parser and dispatcher |
| `src/regex.rs` | Basic regular expression matching and substitution |
| `src/io.rs` | File I/O, shell escapes, prompts, and errors |

## Rules

- **One change per file per PR.** Do not edit another seat's file unless the ticket explicitly spans those files.
- Before pushing, run `./check.sh` locally and only push a passing result. On a machine without Rust, `./check.sh` runs the cargo steps in rootless podman via `Containerfile`.
- Internal CI tooling names are left out of public docs on purpose, for anonymization. `./check.sh` is the only entry point you need.
- Use `./check.sh --self-check` to prove the harnesses against `/usr/bin/ed` without expecting the still-unfinished `ed-rs` binary to pass the GNU suite.
- **NEVER** weaken, delete, skip, or comment out tests in `tests/`.
- **NEVER** alter `tests/diff_harness.py` or anything under `tests/gnu_ed_testsuite/` (except that you may add new starter JSON cases under `tests/cases/` when a ticket says to).
- No `sudo`, no root, no force-push, no `--no-verify`, no history rewrites.
- **NEVER** use `git worktree`. For each ticket, use a plain `git clone` (not a worktree of another checkout). Worktrees record container paths that host-side tools cannot read.
- Keep this crate GPL-3.0-or-later, credited to Bee, Grok Bot, and Nori.
