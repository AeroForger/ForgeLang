"""Harness checks: python3 -B -m unittest discover -s tests -p 'test_backend_harness.py'."""
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from backend_regressions import execute, normal, run_case


class HarnessTests(unittest.TestCase):
    def process(self, code, **kwargs):
        with tempfile.TemporaryDirectory() as directory:
            return execute([sys.executable, "-c", code], directory, **kwargs)

    def test_output_and_nonzero_status_are_separate(self):
        result = self.process("import sys; print('out'); print('err', file=sys.stderr); sys.exit(7)")
        self.assertEqual(result["stdout"], "out\n")
        self.assertEqual(result["stderr"], "err\n")
        self.assertTrue(normal(result, 7))
        self.assertFalse(normal(result, 0))

    def test_stdin_is_preserved(self):
        result = self.process("import sys; sys.stdout.write(sys.stdin.read())", stdin="12\n34\n")
        self.assertEqual(result["stdout"], "12\n34\n")

    def test_signal_is_not_an_exit_code(self):
        result = self.process("import os, signal; os.kill(os.getpid(), signal.SIGTERM)")
        self.assertIsNone(result["exit_status"])
        self.assertEqual(result["signal"], "SIGTERM")
        self.assertFalse(result["timeout"])

    def test_timeout_keeps_partial_output(self):
        result = self.process("import time; print('started', flush=True); time.sleep(30)", timeout=1)
        self.assertTrue(result["timeout"])
        self.assertEqual(result["signal"], "SIGKILL")
        self.assertEqual(result["stdout"], "started\n")

    def test_launch_error_is_not_a_rejection(self):
        with tempfile.TemporaryDirectory() as directory:
            result = execute([str(Path(directory) / "missing")], directory)
        self.assertIsNotNone(result["launch_error"])
        self.assertFalse(normal(result, 1))

    def test_backend_failure_does_not_satisfy_semantic_rejection(self):
        compiled = dict(exit_status=1, signal=None, timeout=False, stdout="",
                        stderr="error: Verifier errors", launch_error=None)
        case = dict(name="invalid", group="test", source="", expected=dict(
            reject=True, diagnostic="error:", forbidden_diagnostics=["Verifier errors"]))
        with patch("backend_regressions.execute", return_value=compiled):
            result = run_case(Path("unused"), "cranelift", case, 1)
        self.assertFalse(result["passed"])
        self.assertIsNone(result["run"])


if __name__ == "__main__":
    unittest.main()
