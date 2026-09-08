# Backend regression tests

These tests specify intended behavior; missing compiler support is a failure, not
an expected failure. No implementation, parser, grammar, or existing fixture was
changed. Run the four groups independently with `--group numeric`, `input`,
`collections`, or `validation`. There are 59 cases, each run on both backends.

```sh
cargo test --offline --locked --target-dir /tmp/furnace-backend-audit-target -- --test-threads=1
python3 -B -m unittest discover -s tests -p test_backend_harness.py
python3 -B tests/backend_regressions.py --binary /tmp/furnace-backend-audit-target/debug/furnace --suite all --report /tmp/furnace-regression-reports/all.json
```

The regression command exits 1 if any case fails. JSON contains each backend,
expected behavior, compile result, and run result. Each subprocess result records
`exit_status`, `signal`, `timeout`, `launch_error`, `stdout`, and `stderr`.
`run: null` means the program was not run, not that it succeeded. Negative tests
never execute unexpectedly accepted programs. Compiler warnings are retained but
do not alone fail a successful compilation. Successful runtime cases require
the expected exit status, exact stdout, and empty stderr.

The runner uses temporary directories, file-backed input/output, process groups,
and timeouts. Core dumps are disabled. It does not retry or suppress failures.
Six harness unit tests verify reporting and prevent verifier failures from being
mistaken for valid semantic diagnostics. No new Python dependency is required.

`--suite existing` runs all 57 existing feature fixtures, 33 error fixtures, and
the unchanged CLI script. Existing trailing-newline tolerance is preserved;
stdout and stderr are now separate, and crashes/nonzero runtime exits cannot
pass merely because their output matches. The CLI script retains its original
assertions and is represented as a single subprocess result.

## Mapping to the original 13 failures

Feature paths below are relative to `stuff/features/`.

| Existing failing test | New coverage |
| --- | --- |
| arrays.anvil | empty collections, bounds, array parameter mutation |
| floats.anvil | precision, mixed reassignment, comparisons, power/negation |
| for_increment_float.anvil | executed increments and decrements; original stops before increment |
| function_returns_ore.anvil | returned tuple/array/list values surviving further calls |
| input.anvil | sequential mixed input, string input, returned strings, bounded strings |
| input_arithmetic.anvil | sequential integer reads and precise float input |
| lists.anvil | float/string elements, alias growth, removal positions, empty lists |
| materials.anvil | mutation and growth visible to caller and alias |
| ore_array.anvil | indexed mutation visible to caller |
| ore_tuple_parameter.anvil | adjacent mixed tuple fields; existing parameter fixture retained |
| strings.anvil | multi-field Data object, member interpolation |
| tuples.anvil | adjacent typed fields and float field interpolation |
| ../cli.sh | original assertions run under each backend, including the known native mismatch |

Numeric tests additionally cover float parameters/returns, mixed ABI calls, and
seven live call results. Integer call-pressure and six/seven-parameter cases
protect existing native calling behavior. Width characterization deliberately
expects native 64-bit arithmetic and Cranelift 32-bit arithmetic; it does not
silently choose a new language-wide integer width.

Validation tests require controlled bounds failures (exit 1, the existing
`Index out of bounds` output). Wrong element/field assignments are required to
report `Type mismatch`, following existing diagnostic wording. Extra arguments
must report `Add expects 1 argument`. The invalid-receiver test rejects backend
unsupported-expression and verifier diagnostics as substitutes for validation.

The bounded string case characterizes the current documented implementation's
255-byte scanf limit. The lifetime case requires two returned input strings to
retain their separate contents. A passing execution is not a proof of absence
of use-after-return: the source audit still applies.

## Coverage limitations requiring a later decision or test seam

- No deterministic allocation-failure injection exists. Adding a runtime seam
  would violate the current tests-only scope. Capacity overflow cannot be
  reached reliably through practical fixture sizes.
- EOF and malformed numeric-input behavior have no specified language contract.
  No zero-value or error-message policy is invented by these tests. Long string
  input and sequential input do have concrete expectations here.
- NaN/infinity formatting, heterogeneous arrays, null/uninitialized collection
  behavior, and automatic reclamation likewise need an explicit contract.
- Tests expose observable layout/lifetime failures; they do not instrument
  generated machine code for memory-safety analysis or prove leak freedom.
- CLI assertions remain unchanged, including their Cranelift-specific messages.
  The native CLI failure remains visible rather than being fixed in this work.
