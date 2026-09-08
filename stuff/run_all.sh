#!/usr/bin/env bash
# Unified ForgeLang test runner.
#
# This command runs every repository test layer and keeps going after a
# failure so that the output contains the result of every test.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="${FURNACE_TEST_TARGET_DIR:-$ROOT/target/unified-tests}"
BINARY="${FURNACE:-$TARGET_DIR/debug/furnace}"
REPORT_DIR="$(mktemp -d)"

trap 'rm -rf "$REPORT_DIR"' EXIT

STEPS=0
FAILED_STEPS=0

run_step() {
    local name="$1"
    shift
    STEPS=$((STEPS + 1))

    echo
    echo "=== $name ==="
    echo "+ $*"
    if "$@"; then
        echo "=== $name: PASS ==="
    else
        local status=$?
        FAILED_STEPS=$((FAILED_STEPS + 1))
        echo "=== $name: FAIL (exit $status) ==="
    fi
}

run_step "Rust unit and integration tests" \
    cargo test --offline --locked --target-dir "$TARGET_DIR" -- --test-threads=1

run_step "Compiler binary build" \
    cargo build --offline --locked --bin furnace --target-dir "$TARGET_DIR"

run_step "Python backend harness tests" \
    python3 -B -m unittest discover -s "$ROOT/tests" -p test_backend_harness.py

if [ -x "$BINARY" ]; then
    run_step "Legacy feature tests" \
        env FURNACE="$BINARY" bash "$ROOT/stuff/run.sh"

    run_step "Legacy error tests" \
        env FURNACE="$BINARY" bash "$ROOT/stuff/run_errors.sh"

    run_step "CLI tests" \
        env FURNACE="$BINARY" bash "$ROOT/stuff/cli.sh"

    run_step "Complete native and Cranelift regression matrix" \
        python3 -B "$ROOT/tests/backend_regressions.py" \
            --binary "$BINARY" \
            --backend both \
            --suite all \
            --report "$REPORT_DIR/all.json"
else
    echo
    echo "=== Backend regression matrix: SKIP (compiler binary was not built) ==="
    FAILED_STEPS=$((FAILED_STEPS + 1))
fi

echo
echo "========================================"
echo "Unified test result: $((STEPS - FAILED_STEPS))/$STEPS stages passed"
if [ "$FAILED_STEPS" -eq 0 ]; then
    echo "All tests passed."
else
    echo "$FAILED_STEPS stage(s) failed."
fi
echo "========================================"

if [ "$FAILED_STEPS" -eq 0 ]; then
    exit 0
fi
exit 1
