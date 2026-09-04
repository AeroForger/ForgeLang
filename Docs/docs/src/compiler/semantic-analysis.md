# Semantic Analysis

Furnace performs semantic analysis before code generation.

The semantic stage checks the parsed AST for invalid programs.

It currently handles checks including:

- Undefined variables
- Invalid function calls
- Incorrect function argument counts
- Invalid `Main` parameters
- Invalid shared-member access
- Forbidden shared-member mutation
- Invalid `Stop` usage
- Invalid `Program.Stop()` usage
- Unknown collection methods
- Array size mismatches
- Tuple field count mismatches
- List element type mismatches
- Invalid empty-list declarations
- Boolean type compatibility for `Bool` and `Boolean` variables
- Other language-level errors

Semantic analysis happens before Cranelift code generation.

This keeps invalid programs from being passed directly to the backend.

## Parallel Semantic Analysis

Furnace uses Rayon for parallel semantic analysis.

Independent portions of the AST can be analyzed concurrently.

For example, independent function declarations can be processed concurrently.

This applies to compiler analysis only.

A ForgeLang program containing:

```forge
While (Condition)
{
    // work
}
```

still executes that loop on one thread unless future language features explicitly introduce parallel execution.

---

[← Previous](../language/imports.md)
[Next →](architecture.md)
