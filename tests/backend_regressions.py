#!/usr/bin/env python3
import argparse
import collections
import json
import os
from pathlib import Path
import resource
import signal
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def execute(command, cwd, stdin="", timeout=10, env=None):
    # Files avoid pipe deadlocks and let timeout reports retain partial output.
    with tempfile.TemporaryFile() as inp, tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
        inp.write(stdin.encode())
        inp.seek(0)
        try:
            proc = subprocess.Popen(command, cwd=cwd, stdin=inp, stdout=out,
                                    stderr=err, env=env, start_new_session=True)
        except OSError as exc:
            return dict(exit_status=None, signal=None, timeout=False,
                        stdout="", stderr="", launch_error=str(exc))
        timed_out = False
        try:
            proc.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(proc.pid, signal.SIGKILL)
            proc.wait()
        out.seek(0)
        err.seek(0)
        return dict(exit_status=proc.returncode if proc.returncode >= 0 else None,
                    signal=signal.Signals(-proc.returncode).name if proc.returncode < 0 else None,
                    timeout=timed_out, stdout=out.read().decode(errors="replace"),
                    stderr=err.read().decode(errors="replace"), launch_error=None)


def normal(result, status):
    return (result["exit_status"] == status and not result["signal"]
            and not result["timeout"] and not result["launch_error"])


def run_case(binary, backend, case, timeout):
    record = dict(name=case["name"], backend=backend, group=case["group"],
                  expected=case.get("expected"), compile=None, run=None, passed=False)
    env = dict(os.environ, FURNACE_BACKEND=backend)
    with tempfile.TemporaryDirectory(prefix="furnace-regression-") as directory:
        if case.get("cli"):
            env["FURNACE"] = str(binary)
            record["run"] = execute(["bash", str(ROOT / "language-tests/cli.sh")], directory,
                                    timeout=max(timeout, 30), env=env)
            record["passed"] = normal(record["run"], 0)
            return record
        source = Path(directory) / "program.anvil"
        source.write_text(case["source"])
        compiled = execute([str(binary), "compile", str(source), "linux", "--backend", backend],
                           directory, timeout=timeout, env=env)
        record["compile"] = compiled
        expected = case["expected"]
        if "backends" in expected:
            expected = expected["backends"][backend]
        record["expected"] = expected
        if expected.get("reject"):
            record["passed"] = (normal(compiled, 1)
                                and expected.get("diagnostic", "error:") in compiled["stderr"]
                                and not any(fragment in compiled["stderr"] for fragment in
                                            expected.get("forbidden_diagnostics", [])))
            return record
        if not normal(compiled, 0):
            return record
        result = execute([str(Path(directory) / "program")], directory,
                         stdin=case.get("stdin", ""), timeout=timeout, env=env)
        record["run"] = result
        stdout = result["stdout"]
        wanted = expected["stdout"]
        if case.get("legacy_newlines"):
            stdout, wanted = stdout.rstrip("\n"), wanted.rstrip("\n")
        record["passed"] = (normal(result, expected.get("exit_status", 0))
                            and stdout == wanted and result["stderr"] == "")
    return record


def existing_cases():
    for source in sorted((ROOT / "stuff/features").glob("*.anvil")):
        stdin = source.with_suffix(".stdin")
        yield dict(name=source.stem, group="existing/features", source=source.read_text(),
                   stdin=stdin.read_text() if stdin.exists() else "", legacy_newlines=True,
                   expected=dict(stdout=source.with_suffix(".expected").read_text()))
    for source in sorted((ROOT / "stuff/errors").glob("*.anvil")):
        diagnostic = source.with_suffix(".error")
        yield dict(name=source.stem, group="existing/errors", source=source.read_text(),
                   expected=dict(reject=True, diagnostic=diagnostic.read_text().rstrip("\n")
                                 if diagnostic.exists() else "error:"))
    yield dict(name="cli", group="existing/cli", cli=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/furnace")
    parser.add_argument("--backend", choices=["native", "cranelift", "both"], default="both")
    parser.add_argument("--suite", choices=["existing", "new", "all"], default="new")
    parser.add_argument("--group", help="Run only this new regression group")
    parser.add_argument("--timeout", type=float, default=10)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error("--timeout must be positive")
    if args.group and args.suite == "existing":
        parser.error("--group selects new regressions only")
    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error(f"compiler missing: {binary}")
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    cases = list(existing_cases()) if args.suite in ("existing", "all") else []
    if args.suite in ("new", "all"):
        for manifest in sorted((ROOT / "tests/regressions").glob("*.json")):
            group = manifest.stem
            if args.group and args.group != group:
                continue
            for case in json.loads(manifest.read_text()):
                cases.append(dict(case, group=group))
    if not cases:
        parser.error("no matching tests")
    records = []
    for backend in (["native", "cranelift"] if args.backend == "both" else [args.backend]):
        for case in cases:
            record = run_case(binary, backend, case, args.timeout)
            records.append(record)
            print(f"{'PASS' if record['passed'] else 'FAIL'} {backend} {case['group']}/{case['name']}", flush=True)
            if not record["passed"]:
                for stage in ("compile", "run"):
                    result = record[stage]
                    print(f"  {stage}: {json.dumps(result, ensure_ascii=True) if result else 'not run'}", flush=True)
    counts = collections.defaultdict(lambda: dict(passed=0, failed=0))
    for record in records:
        counts[record["backend"]]["passed" if record["passed"] else "failed"] += 1
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(dict(binary=str(binary), summary=dict(counts), results=records), indent=2) + "\n")
    print(json.dumps(dict(counts)), flush=True)
    return int(any(not record["passed"] for record in records))


if __name__ == "__main__":
    raise SystemExit(main())
