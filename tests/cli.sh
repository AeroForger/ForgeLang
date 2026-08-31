#!/usr/bin/env bash
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${FURNACE:-$ROOT/target/debug/furnace}"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

fail() {
    echo "FAIL: $1" >&2
    exit 1
}

if [ ! -x "$BIN" ]; then
    echo "furnace binary not found at $BIN" >&2
    echo "build with: cargo build --bin furnace" >&2
    exit 2
fi

# version output
actual="$($BIN -version 2>&1)"
[ "$actual" = "Furnace Alpha 3" ] || fail "-version output mismatch: '$actual'"

# help output
help_actual="$($BIN -help 2>&1)"
printf '%s\n' "$help_actual" | grep -q "Furnace Alpha 3" || fail "-help is missing banner"
printf '%s\n' "$help_actual" | grep -q "Furnace compile <file>.anvil <platform>" || fail "-help is missing compile usage"
printf '%s\n' "$help_actual" | grep -q "Furnace run <file>.anvil" || fail "-help is missing run usage"

# compile success
SRC="$TMPDIR/ok.anvil"
cat > "$SRC" <<'EOF'
Open Nunction Main()
{
    Print("ok-from-cli");
}
EOF

OUT="$TMPDIR/ok"
compile_out="$($BIN compile "$SRC" linux 2>&1)"
printf '%s\n' "$compile_out" | grep -q "Compiling" || fail "compile output missing compile notice"
printf '%s\n' "$compile_out" | grep -q "Linking" || fail "compile output missing linking notice"
printf '%s\n' "$compile_out" | grep -q "Build successful!" || fail "compile output missing success"
[ -x "$OUT" ] || fail "expected compiled executable at $OUT"

# run success
run_out="$($BIN run "$SRC" 2>&1)"
printf '%s\n' "$run_out" | grep -q "ok-from-cli" || fail "run output missing program output"

# invalid extension
if $BIN compile "$TMPDIR/invalid.txt" linux >/dev/null 2>&1; then
    fail "compile with non-anvil file should fail"
fi

# bad platform
if $BIN compile "$SRC" windows >/dev/null 2>&1; then
    fail "compile with unsupported platform should fail"
fi

# fake file path
if $BIN run "$TMPDIR/missing.anvil" >/dev/null 2>&1; then
    fail "run should fail for missing file"
fi

# broken program should fail strongly
BROKEN="$TMPDIR/broken.anvil"
cat > "$BROKEN" <<'EOF'
Open Nunction Main()
{
    Print(unknown_var);
}
EOF
if $BIN compile "$BROKEN" linux >/dev/null 2>&1; then
    fail "compile should fail for invalid ForgeLang code"
fi

echo "CLI tests passed"
