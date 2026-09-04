# Current Limitations

Alpha 3.4 is an early development release.

The following features are not currently fully implemented in the backend:

- Parameterized function code generation
- Native calls to parameterized user-defined functions
- Function return-value code generation
- `Return` statements
- `Data` declaration code generation
- Object instantiation code generation
- `Switch` / `Deal` / `Base` pattern matching
- `Do` / `Fail` / `Final` error handling
- Functional `Use` / `Using` imports
- Complete module system
- Complete lexical scope handling
- Multicore ForgeLang program execution
- Garbage collection
- Self-hosting Furnace
- Complete systems-level standard library

Some of these features are already represented in the grammar or AST.

There is an important distinction:

**Parsed** means Furnace can recognize the syntax.

**Semantically checked** means Furnace can inspect the construct and report certain errors.

**Code generated** means Furnace can produce executable native code for the construct.

A feature being parsed does not mean that it can currently be used in an executable program.

This distinction saves everyone from discovering that the compiler supports something only after the compiler politely refuses to compile it.

---

[← Previous](examples/complete-example.md)
[Next →](roadmap.md)
