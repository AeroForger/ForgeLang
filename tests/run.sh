#!/bin/bash
# ForgeLang regression suite: compiles every tests/features/*.anvil
# and diffs the output against its .expected file.
# Trailing-newline-insensitive: .expected files may or may not end in \n.

DIR="$(cd "$(dirname "$0")/features" && pwd)"
ROOT="$(dirname "$(dirname "$0")")"
TMP=$(mktemp -d)
trap "rm -rf $TMP" EXIT

pass=0; fail=0
for anvil in "$DIR"/*.anvil; do
    name=$(basename "$anvil" .anvil)
    expected="$DIR/$name.expected"

    # Compile (IR to stdout via --emit-llvm, machine-clean)
    if ! python3 "$ROOT/main.py" --emit-llvm "$anvil" > "$TMP/$name.ll" 2> "$TMP/$name.err"; then
        echo "FAIL $name (compile error)"; cat "$TMP/$name.err"; fail=$((fail+1)); continue
    fi
    if ! clang "$TMP/$name.ll" -o "$TMP/$name.out" -lm 2> "$TMP/$name.clangerr"; then
        echo "FAIL $name (clang error)"; cat "$TMP/$name.clangerr"; fail=$((fail+1)); continue
    fi

    # Run and compare.
    # $(...) strips ALL trailing newlines from both sides, so it does not
    # matter whether the .expected file ends with 0, 1, or 3 newlines.
    actual=$("$TMP/$name.out")
    wanted=$(cat "$expected")

    if [ "$actual" = "$wanted" ]; then
        echo "PASS $name"; pass=$((pass+1))
    else
        echo "FAIL $name (output mismatch)"
        diff -u <(echo "$wanted") <(echo "$actual") | tail -n +3
        fail=$((fail+1))
    fi
done

echo ""
echo "----- $pass passed, $fail failed -----"
[ $fail -eq 0 ]