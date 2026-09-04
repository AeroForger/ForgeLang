# The Stop Statement

`Stop` provides an early exit from a loop or conditional block.

Example:

```forge
While (I < 10)
{
    If (I == 5)
    {
        Stop;
    }

    I = I + 1;
}
```

The semantic analyzer currently enforces these rules:

- `Stop` cannot be used inside `Main`
- `Stop` must be inside a loop or `If` statement

In the compiler, `Stop` is represented as `Statement::Stop` in `src/ast.rs`.

The semantic checks are implemented in `src/semantic.rs`.

Code generation is handled by `FunctionCompiler::compile_statement` in `src/codegen.rs`.

`Stop` is not an exception mechanism. It represents a structured early exit.

---

[← Previous](for.md)
[Next →](program-namespace.md)
