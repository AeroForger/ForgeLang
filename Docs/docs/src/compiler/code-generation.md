# Code Generation

`src/codegen.rs` converts supported ForgeLang constructs into Cranelift IR.

Cranelift handles:

- Instruction selection
- Register allocation
- Machine code generation
- Target-specific code generation
- Object-file generation

The compiler pipeline is:

```text
ForgeLang source -> pest -> AST -> Semantic Analysis -> Cranelift -> Native Object Code -> System Linker -> Executable
```

## Supported Constructs

Currently supported code generation includes the subset of the language that the backend can lower cleanly to native instructions, such as:

- variables and assignments
- primitive arithmetic
- loops and conditionals
- zero-argument `Nunction` call expansion
- collection access for supported layouts

## Unsupported and Planned Constructs

The backend does not currently generate executable code for:

- parameterized function calls
- return-value functions
- `Return` statements
- `Data` declarations
- object instantiation
- module imports
- full lexical scope handling

This distinction matters: being parsed or semantically checked does not mean it is fully executable.

See [Current Limitations](../limitations.md) for the full status matrix.

---

[← Previous](architecture.md)
[Next →](linking.md)
