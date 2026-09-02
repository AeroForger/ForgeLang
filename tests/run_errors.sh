#!/usr/bin/env bash
# tests/run_errors.sh — Error tests for ForgeLang parser/compiler
set -uo pipefail

FEATURES_DIR="$(cd "$(dirname "$0")" && pwd)/errors"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FURNACE="${FURNACE:-$ROOT/target/release/furnace}"

if [ ! -x "$FURNACE" ]; then
    echo "furnace binary not found at $FURNACE" >&2
    echo "build with: cargo build --release" >&2
    exit 2
fi

shopt -s nullglob
TESTS=( "$FEATURES_DIR"/*.anvil )
shopt -u nullglob

if [ ${#TESTS[@]} -eq 0 ]; then
    echo "ERROR: zero error tests found in $FEATURES_DIR" >&2
    exit 1
fi

PASS=0
FAIL=0

for anvil in "${TESTS[@]}"; do
    name="$(basename "$anvil" .anvil)"

    tmpdir="$(mktemp -d)"
    if (cd "$tmpdir" && "$FURNACE" compile "$anvil" linux >"$tmpdir/compile.log" 2>&1); then
        echo "FAIL $name (should have been rejected but compiled)"
        FAIL=$((FAIL+1))
    else
        expected_file="$FEATURES_DIR/$name.error"
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
            fi
        else
            echo "PASS $name (rejected)"
            PASS=$((PASS+1))
        fi
    fi
    rm -rf "$tmpdir"
done

echo ""
echo "Error tests: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
