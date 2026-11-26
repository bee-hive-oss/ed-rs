#!/bin/sh
# Single entry point for ed-rs proofs.
#
# Runs, in order:
#   1) cargo test
#   2) cargo clippy --all-targets -- -D warnings
#   3) tests/diff_harness.py   (20 starter cases vs GNU ed)
#   4) tests/run_gnu_testsuite.sh  (official GNU ed 1.22.4 testsuite)
#
# Usage:
#   ./check.sh              test the ed-rs binary (build + all proofs)
#   ./check.sh --self-check prove the harnesses against /usr/bin/ed
#
# If cargo is not on PATH, cargo steps run in a rootless podman container
# built from ./Containerfile. Harnesses always run on the host so they
# compare against this machine's /usr/bin/ed (GNU ed 1.22.4 on Levo).
# Runs without root privileges.

set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
cd "$ROOT"

SELF_CHECK=0
IN_CONTAINER=0

usage() {
	cat <<'USAGE'
Usage: ./check.sh [--self-check]

  ./check.sh              build ed-rs and prove it against GNU ed
  ./check.sh --self-check prove harnesses against /usr/bin/ed
                          (also compiles the crate if cargo or podman
                          is available)

Cargo steps run locally when cargo is on PATH, otherwise via rootless
podman from ./Containerfile. No root privileges needed.
USAGE
}

for arg in "$@"; do
	case "$arg" in
		--self-check) SELF_CHECK=1 ;;
		--in-container) IN_CONTAINER=1 ;;
		-h|--help) usage; exit 0 ;;
		*)
			echo "unknown option: $arg" >&2
			usage >&2
			exit 2
			;;
	esac
done

log() {
	printf '==> %s\n' "$*"
}

have_cmd() {
	command -v "$1" >/dev/null 2>&1
}

IMAGE_NAME="localhost/ed-rs-check:latest"

run_in_podman() {
	# $1 = shell command to run at /work inside the image
	if ! have_cmd podman; then
		echo "cargo is not on PATH, and podman is not available to run ./Containerfile" >&2
		exit 2
	fi
	log "building rootless podman image ${IMAGE_NAME} from Containerfile"
	podman build -t "$IMAGE_NAME" -f "$ROOT/Containerfile" "$ROOT"
	log "running cargo steps in podman"
	# keep-id so target/ (if written into the bind mount) stays host-owned.
	# CARGO_TARGET_DIR lives inside the repo so the release binary can be
	# used by host-side harnesses in normal mode.
	podman run --rm \
		--userns=keep-id \
		-e CARGO_HOME=/tmp/cargo-home \
		-v "$ROOT:/work:rw" \
		-w /work \
		"$IMAGE_NAME" \
		sh -c "$1"
}

cargo_script() {
	# Compile proofs. In normal mode also produce target/release/ed-rs.
	if [ "$SELF_CHECK" -eq 1 ]; then
		cat <<'SCRIPT'
set -eu
cargo test
cargo clippy --all-targets -- -D warnings
SCRIPT
	else
		cat <<'SCRIPT'
set -eu
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
SCRIPT
	fi
}

run_cargo_steps() {
	if have_cmd cargo && [ "$IN_CONTAINER" -eq 0 ]; then
		log "cargo test"
		cargo test
		log "cargo clippy --all-targets -- -D warnings"
		cargo clippy --all-targets -- -D warnings
		if [ "$SELF_CHECK" -eq 0 ]; then
			log "cargo build --release"
			cargo build --release
		fi
		return 0
	fi

	if [ "$IN_CONTAINER" -eq 1 ]; then
		echo "cargo is not on PATH inside the container" >&2
		exit 2
	fi

	run_in_podman "$(cargo_script)"
}

find_ed_rs() {
	if [ -x "$ROOT/target/release/ed-rs" ]; then
		echo "$ROOT/target/release/ed-rs"
		return 0
	fi
	if [ -x "$ROOT/target/debug/ed-rs" ]; then
		echo "$ROOT/target/debug/ed-rs"
		return 0
	fi
	return 1
}

run_harnesses() {
	if ! have_cmd python3; then
		echo "python3 is required to run tests/diff_harness.py" >&2
		exit 2
	fi
	if [ ! -x /usr/bin/ed ]; then
		echo "/usr/bin/ed is required as the GNU ed reference" >&2
		exit 2
	fi

	if [ "$SELF_CHECK" -eq 1 ]; then
		cand=/usr/bin/ed
		log "diff_harness.py --self-check (ref=cand=/usr/bin/ed)"
	else
		if ! cand=$(find_ed_rs); then
			echo "ed-rs binary not found under target/release or target/debug" >&2
			exit 1
		fi
		log "diff_harness.py (ref=/usr/bin/ed cand=${cand})"
	fi

	# Run both suites every time, even when the first fails, so both
	# scoreboard lines are always printed and a regression in either one
	# is visible. The script still fails if either suite fails.
	harness_rc=0
	python3 "$ROOT/tests/diff_harness.py" --ref /usr/bin/ed --cand "$cand" || harness_rc=$?

	gnu_rc=0
	if [ "$SELF_CHECK" -eq 1 ]; then
		log "run_gnu_testsuite.sh --self-check"
		"$ROOT/tests/run_gnu_testsuite.sh" --self-check || gnu_rc=$?
	else
		log "run_gnu_testsuite.sh ${cand}"
		"$ROOT/tests/run_gnu_testsuite.sh" "$cand" || gnu_rc=$?
	fi

	if [ "$harness_rc" -ne 0 ] || [ "$gnu_rc" -ne 0 ]; then
		echo "harness failures: diff_harness.py exit ${harness_rc}, run_gnu_testsuite.sh exit ${gnu_rc}" >&2
		exit 1
	fi
}

log "ed-rs check (self_check=${SELF_CHECK}) in $ROOT"
run_cargo_steps
run_harnesses
log "all checks passed"
