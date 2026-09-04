#!/usr/bin/env bash
# tests/run_all.sh — Unified test runner for ForgeLang
# Runs feature tests, error tests, and CLI tests in sequence.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FURNACE="${FURNACE:-$ROOT/target/release/furnace}"
PASS=0
FAIL=0
FAILED_TESTS=""

if [ ! -x "$FURNACE" ]; then
    echo "furnace binary not found at $FURNACE" >&2
    echo "build with: cargo build --release" >&2
    exit 2
fi

echo "=== Feature Tests ==="
shopt -s nullglob
FEATURES_DIR="$ROOT/tests/features"
TESTS=( "$FEATURES_DIR"/*.anvil )
shopt -u nullglob

if [ ${#TESTS[@]} -eq 0 ]; then
    echo "ERROR: zero tests found in $FEATURES_DIR" >&2
    exit 1
fi

for anvil in "${TESTS[@]}"; do
    name="$(basename "$anvil" .anvil)"
    expected="$FEATURES_DIR/$name.expected"
    stdin_file="$FEATURES_DIR/$name.stdin"

    if [ ! -f "$expected" ]; then
        echo "MISS $name (no .expected)"
        FAIL=$((FAIL+1))
        FAILED_TESTS="$FAILED_TESTS $name"
        continue
    fi

    tmpdir="$(mktemp -d)"
    exe="$tmpdir/$name"

    if ! (cd "$tmpdir" && "$FURNACE" compile "$anvil" linux >"$tmpdir/compile.log" 2>&1); then
        echo "FAIL $name (compile)"
        sed 's/^/    /' "$tmpdir/compile.log" >&2
        FAIL=$((FAIL+1))
        FAILED_TESTS="$FAILED_TESTS $name"
        rm -rf "$tmpdir"
        continue
    fi

    if [ -f "$stdin_file" ]; then
        actual="$("$exe" < "$stdin_file" 2>&1)"
    else
        actual="$("$exe" < /dev/null 2>&1)"
    fi
    expected_content="$(cat "$expected")"

    if [ "$actual" = "$expected_content" ]; then
        echo "PASS $name"
        PASS=$((PASS+1))
    else
        echo "FAIL $name (output)"
        echo "--- expected ---" >&2
        echo "$expected_content" >&2
        echo "--- actual ---" >&2
        echo "$actual" >&2
        FAIL=$((FAIL+1))
        FAILED_TESTS="$FAILED_TESTS $name"
    fi
    rm -rf "$tmpdir"
done

echo ""
echo "=== Error Tests ==="
ERRORS_DIR="$ROOT/tests/errors"
shopt -s nullglob
ERR_TESTS=( "$ERRORS_DIR"/*.anvil )
shopt -u nullglob

if [ ${#ERR_TESTS[@]} -eq 0 ]; then
    echo "ERROR: zero error tests found in $ERRORS_DIR" >&2
    exit 1
fi

for anvil in "${ERR_TESTS[@]}"; do
    name="$(basename "$anvil" .anvil)"

    tmpdir="$(mktemp -d)"
    if (cd "$tmpdir" && "$FURNACE" compile "$anvil" linux >"$tmpdir/compile.log" 2>&1); then
        echo "FAIL $name (should have been rejected but compiled)"
        FAIL=$((FAIL+1))
        FAILED_TESTS="$FAILED_TESTS $name"
    else
        expected_file="$ERRORS_DIR/$name.error"
        if [ -f "$expected_file" ]; then
            expected_pattern="$(cat "$expected_file")"
            if grep -qF "$expected_pattern" "$tmpdir/compile.log"; then
                echo "PASS $name"
                PASS=$((PASS+1))
            else
                echo "FAIL $name (error message mismatch)"
                echo "  expected to contain: $expected_pattern" >&2
                sed 's/^/    /' "$tmpdir/compile.log" >&2
                FAIL=$((FAIL+1))
                FAILED_TESTS="$FAILED_TESTS $name"
            fi
        else
            echo "PASS $name (rejected)"
            PASS=$((PASS+1))
        fi
    fi
    rm -rf "$tmpdir"
done

echo ""
echo "=== CLI Tests ==="
echo "(running cli.sh)"
if bash "$ROOT/tests/cli.sh" 2>&1; then
    echo "CLI tests passed"
    PASS=$((PASS+1))
else
    echo "FAIL: CLI tests"
    FAIL=$((FAIL+1))
    FAILED_TESTS="$FAILED_TESTS cli"
fi

echo ""
echo "=========================================="
echo "Total: $PASS passed, $FAIL failed"
if [ -n "$FAILED_TESTS" ]; then
    echo "Failed:$FAILED_TESTS"
fi
echo "=========================================="
[ "$FAIL" -eq 0 ]
