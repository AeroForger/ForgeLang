#!/usr/bin/env bash
# tests/run.sh — ForgeLang test runner (ported from Python reference)
set -uo pipefail

FEATURES_DIR="$(cd "$(dirname "$0")" && pwd)/features"
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
    echo "ERROR: zero tests found in $FEATURES_DIR" >&2
    exit 1
fi

PASS=0
FAIL=0

for anvil in "${TESTS[@]}"; do
    name="$(basename "$anvil" .anvil)"
    expected="$FEATURES_DIR/$name.expected"
    stdin_file="$FEATURES_DIR/$name.stdin"

    if [ ! -f "$expected" ]; then
        echo "MISS $name (no .expected)"
        FAIL=$((FAIL+1))
        continue
    fi

    tmpdir="$(mktemp -d)"
    exe="$tmpdir/$name"

    if ! "$FURNACE" "$anvil" -o "$exe" -lm >"$tmpdir/compile.log" 2>&1; then
        echo "FAIL $name (compile)"
        sed 's/^/    /' "$tmpdir/compile.log" >&2
        FAIL=$((FAIL+1))
        rm -rf "$tmpdir"
        continue
    fi

    if [ -f "$stdin_file" ]; then
        actual="$("$exe" < "$stdin_file" 2>&1)"
    else
        actual="$("$exe" < /dev/null 2>&1)"
    fi
    expected_content="$(cat "$expected")"

    # bash $() strips trailing newlines on both sides → trailing-newline-insensitive
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
    fi
    rm -rf "$tmpdir"
done

echo ""
echo "Results: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]