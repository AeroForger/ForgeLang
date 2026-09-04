# Implementation Notes

Alpha 3.4 uses a different compiler implementation from the earlier experimental versions of ForgeLang.

Earlier versions used:

Alpha 1.x and 2.x used:

```text
Python
ANTLR
LLVM
```

Alpha 3.4 uses:

```text
Rust
pest
Cranelift
```

The current compiler pipeline is:

```text
ForgeLang source -> pest -> AST -> Semantic Analysis -> Cranelift -> Native Object Code
```

The change to Rust also makes the compiler itself part of the ForgeLang project's systems-level development work.

Alpha 3.4 should not be treated as a finished language specification.

Some syntax exists before its backend implementation.

Some AST structures exist before their code generation.

Some planned language features are already represented in the parser even though the compiler cannot execute them yet.

That is normal for a compiler under active development.

For now, Furnace can compile a growing subset of ForgeLang to native code while the rest of the language catches up.

---

[← Previous](roadmap.md)
