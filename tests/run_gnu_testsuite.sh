#!/bin/sh
# Run GNU ed 1.22.4's official testsuite against a target binary.
#
# Usage:
#   ED_BIN=/path/to/ed tests/run_gnu_testsuite.sh
#   tests/run_gnu_testsuite.sh /path/to/ed
#   tests/run_gnu_testsuite.sh --self-check     # prove the suite vs /usr/bin/ed
#
# Prints a line of the form:
#   GNU_ED_TESTSUITE: X / Y passing
# and exits 0 only if every official check passed.
#
# The official testsuite/check.sh is the authority; this wrapper only
# sets up the binary, reports counts, and forwards the official result.

set -eu

# Prefer bash for the official script. dash leaves $LINENO blank, so
# start-up failures print no line numbers and cannot be counted.
official_interpreter() {
	if command -v bash >/dev/null 2>&1; then
		printf '%s\n' bash
	else
		printf '%s\n' ""
	fi
}

# Start-up failures are the numbers on the line right after
# "testing ed-...". Each number may have a reason in brackets after it.
# A following "*** ..." line is a script failure, not a start-up failure.
count_startup_failures() {
	printf '%s\n' "$1" | awk '
		BEGIN { c = 0 }
		/testing ed-/ {
			if ((getline line) > 0 && line !~ /\*\*\*/) {
				gsub(/\([^)]*\)/, "", line)
				n = split(line, a, /[^0-9]+/)
				for (i = 1; i <= n; i++) if (a[i] != "") c++
			}
			exit
		}
		END { print c + 0 }
	'
}

# Built-in check so a miscount cannot go silent. Fixtures cover mixed
# start-up and script failures, bracketed reasons, and script-only output.
verify_failure_counter() {
	_assert_startup_count() {
		_got=$(count_startup_failures "$2" | tr -d " \n")
		[ -n "$_got" ] || _got=0
		if [ "$_got" -ne "$3" ]; then
			echo "run_gnu_testsuite.sh: failure counter $1 counted $_got start-up failures, expected $3" >&2
			exit 2
		fi
	}
	_assert_startup_count mixed "$(printf '%s\n' "testing ed-1.22.4..." " 44 52 54" "*** a ***" "*** b ***")" 3
	_assert_startup_count reasons "$(printf '%s\n' "testing ed-1.22.4..." " 44 (no match) 52 54")" 3
	_assert_startup_count scripts_only "$(printf '%s\n' "testing ed-1.22.4..." "*** a ***" "*** b ***")" 0
	_assert_startup_count none "$(printf '%s\n' "testing ed-1.22.4..." "tests completed successfully.")" 0
	_assert_startup_count blank_lineno "$(printf '%s\n' "testing ed-1.22.4..." "   " "*** a ***")" 0
}

verify_failure_counter

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
TESTDIR="${SCRIPT_DIR}/gnu_ed_testsuite"
VERSION="1.22.4"
OFFICIAL_CHECK="${TESTDIR}/check.sh"

ED_BIN="${ED_BIN:-}"
SELF_CHECK=0

usage() {
	cat <<'USAGE'
Usage: tests/run_gnu_testsuite.sh [--self-check] [ED_BIN]

Run GNU ed 1.22.4's official testsuite against ED_BIN (or $ED_BIN).
--self-check  uses /usr/bin/ed and expects a 100% pass.
USAGE
}

for arg in "$@"; do
	case "$arg" in
		--self-check) SELF_CHECK=1 ;;
		-h|--help) usage; exit 0 ;;
		--) shift; break ;;
		-*)
			echo "unknown option: $arg" >&2
			usage >&2
			exit 2
			;;
		*)
			if [ -n "$ED_BIN" ]; then
				echo "unexpected argument: $arg" >&2
				usage >&2
				exit 2
			fi
			ED_BIN=$arg
			;;
	esac
done

if [ "$SELF_CHECK" -eq 1 ] && [ -z "$ED_BIN" ]; then
	ED_BIN=/usr/bin/ed
fi

if [ -z "$ED_BIN" ]; then
	echo "ED_BIN is required (or pass --self-check)" >&2
	usage >&2
	exit 2
fi

if [ ! -e "$ED_BIN" ]; then
	echo "ED_BIN not found: $ED_BIN" >&2
	exit 2
fi
if [ ! -x "$ED_BIN" ]; then
	echo "ED_BIN is not executable: $ED_BIN" >&2
	exit 2
fi

# Resolve to an absolute path so the official check.sh can exec it
# after it changes into a temporary objdir.
case "$ED_BIN" in
	/*) ;;
	*) ED_BIN=$(CDPATH= cd -- "$(dirname "$ED_BIN")" && pwd)/$(basename "$ED_BIN") ;;
esac

if [ ! -x "$OFFICIAL_CHECK" ]; then
	echo "official check.sh missing: $OFFICIAL_CHECK" >&2
	exit 2
fi

n_err=$(ls -1 "${TESTDIR}"/*.err | wc -l)
n_ed=$(ls -1 "${TESTDIR}"/*.ed | wc -l)
n_err=$(echo "$n_err" | tr -d ' ')
n_ed=$(echo "$n_ed" | tr -d ' ')

# Official check.sh has 16 always-on startup/CLI assertions, plus 3
# filename-encoding checks that run only if a latin-1 name can be created.
n_startup=16
n_encoding=0
enc_dir=$(mktemp -d "${TMPDIR:-/tmp}/ed-enc.XXXXXX")
# The official script uses the latin-1 byte 0xE9 in "café".
if python3 -c '
import os, sys
p = os.path.join(sys.argv[1], b"caf\xe9_iso.txt".decode("latin-1"))
try:
    open(p, "wb").close()
except OSError:
    sys.exit(1)
' "$enc_dir" 2>/dev/null; then
	n_encoding=3
fi
rm -rf "$enc_dir"

n_total=$((n_startup + n_encoding + 3 * n_err + n_ed))

echo "testing GNU ed ${VERSION} official testsuite against ${ED_BIN}"
echo "  startup/CLI checks:                 ${n_startup}"
if [ "$n_encoding" -gt 0 ]; then
	echo "  filename-encoding checks:           ${n_encoding}"
else
	echo "  filename-encoding checks:           skipped (latin-1 names not usable)"
fi
echo "  error scripts, file stdin:          ${n_err}"
echo "  error scripts, --loose-exit-status: ${n_err}"
echo "  error scripts, piped:               ${n_err}"
echo "  script tests (.ed vs .r):           ${n_ed}"
echo "  total official checks:              ${n_total}"
echo

objdir=$(mktemp -d "${TMPDIR:-/tmp}/ed-gnu-suite.XXXXXX")
cleanup() {
	rm -rf "$objdir"
}
trap cleanup EXIT INT HUP TERM

# Official check.sh expects a binary named "ed" in the objdir (cwd).
ln -s "$ED_BIN" "${objdir}/ed"

set +e
# Run from objdir, as Makefile.in does: check.sh testdir pkgversion
official_interp=$(official_interpreter)
if [ -n "$official_interp" ]; then
	# bash fills in $LINENO; a blank probe means start-up
	# failures would be silently dropped from the count.
	lineno_probe=$(bash -c 'printf %s "$LINENO"')
	if [ -z "$lineno_probe" ]; then
		echo "bash is on PATH but \$LINENO is empty; start-up failures would not be counted" >&2
		exit 2
	fi
fi
official_out=$(
	cd "$objdir" || exit 1
	if [ -n "$official_interp" ]; then
		bash "$OFFICIAL_CHECK" "$TESTDIR" "$VERSION"
	else
		"$OFFICIAL_CHECK" "$TESTDIR" "$VERSION"
	fi
)
official_rc=$?
set -e

printf '%s\n' "$official_out"

if [ "$official_rc" -eq 0 ]; then
	n_pass=$n_total
	echo
	echo "GNU_ED_TESTSUITE: ${n_pass} / ${n_total} passing"
	exit 0
fi

# Best-effort failed-check count from official messages.
# Start-up failures are the numbers on the line right after
# "testing ed-..."; script failures print a "*** ... ***" line.
n_fail_startup=$(count_startup_failures "$official_out" | tr -d " \n")
[ -n "$n_fail_startup" ] || n_fail_startup=0
n_fail_scripts=$(printf '%s\n' "$official_out" | grep -c '\*\*\*' || true)
n_fail=$((n_fail_startup + n_fail_scripts))
if [ "$n_fail" -eq 0 ]; then
	# Official failed, but we couldn't name individual cases.
	n_fail=1
fi
# If bash ran the official script and the start-up line came out
# as blank spaces, line numbers were lost and the count is wrong.
if [ -n "$official_interp" ] && [ "$n_fail_startup" -eq 0 ]; then
	startup_line=$(printf '%s\n' "$official_out" | awk '/testing ed-/{getline; print; exit}')
	case "$startup_line" in
		*[![:space:]]*) ;;
		*)
			if [ -n "$startup_line" ]; then
				echo "run_gnu_testsuite.sh: start-up failure line numbers were blank; cannot count them" >&2
				exit 2
			fi
			;;
	esac
fi
n_pass=$((n_total - n_fail))
if [ "$n_pass" -lt 0 ]; then
	n_pass=0
fi

echo
echo "GNU_ED_TESTSUITE: ${n_pass} / ${n_total} passing"
exit 1
