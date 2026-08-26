import sys
import subprocess
import os
import shutil

from furnace.pipeline import compile_file
from furnace.errors import ForgeError

VERSION = "0.2.0"


def main(argv=None):
    if argv is None:
        argv = sys.argv
    args = [a for a in argv[1:] if not a.startswith("-")]
    flags = [a for a in argv[1:] if a.startswith("-")]

    if "--version" in flags:
        print(f"furnace {VERSION}")
        return 0

    if len(args) != 1:
        print("Usage: furnace [--emit-llvm] [--keep-ll] <file.anvil>", file=sys.stderr)
        return 2

    source = args[0]
    base = source.rsplit(".", 1)[0]
    emit_llvm = "--emit-llvm" in flags
    keep_ll = "--keep-ll" in flags

    try:
        ir = compile_file(source)
    except ForgeError as e:
        print(e.render(source), file=sys.stderr)
        return 1

    if emit_llvm:
        print(ir)
        return 0

    # Write the IR to a .ll file
    ll_path = base + ".ll"
    with open(ll_path, "w") as f:
        f.write(ir)

    # Link with clang
    exe_path = base + ".out"
    clang = shutil.which("clang")
    if clang is None:
        print("error: clang not found in PATH (required for linking)", file=sys.stderr)
        return 1

    result = subprocess.run(
        [clang, ll_path, "-o", exe_path, "-lm", "-Wno-override-module"],
        capture_output=True, text=True,
    )
    if result.returncode != 0:
        print(f"error: clang failed:\n{result.stderr}", file=sys.stderr)
        return 1

    if not keep_ll:
        os.remove(ll_path)

    # Status goes to stderr: stdout stays clean for machine consumption
    print(f"Successfully forged executable: {exe_path}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))