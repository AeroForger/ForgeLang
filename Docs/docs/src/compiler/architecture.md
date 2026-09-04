# Compiler Architecture

Furnace is divided into several stages.

## Parser

The parser uses pest, a PEG parser generator for Rust.

It converts ForgeLang source into the AST.

The grammar uses explicit precedence rules rather than left-recursive expression rules.

Operator precedence, from highest to lowest, is:

```text
primary -> postfix -> power -> unary -> multiplicative -> additive -> comparison -> and -> or -> xor
```

A documented language rule is that unary operators bind looser than `**`.

Therefore:

```forge
-2 ** 2
```

is interpreted as:

```text
-(2 ** 2)
```

which produces:

```text
-4
```

## AST

The AST is represented using Rust structures.

It acts as the representation shared between parsing, semantic analysis, and code generation.

The compiler works with the AST rather than passing raw source text between compiler stages.

## Semantic Analysis

`src/semantic.rs` validates the AST before code generation.

This stage handles language-level checks such as:

- Type compatibility
- Variable lookup
- Function lookup
- Function argument counts
- Collection operations
- Scope-related checks
- Control-flow restrictions

## Code Generation

`src/codegen.rs` converts supported ForgeLang constructs into Cranelift IR.

Cranelift handles:

- Instruction selection
- Register allocation
- Machine code generation
- Target-specific code generation
- Object-file generation

Furnace produces a native object file from the generated code.

### Function Call Expansion

Before generating code for `Main`, Furnace runs `expand_function_calls`.

The pass is located in `src/codegen.rs`.

It replaces eligible zero-argument `Nunction` calls with the statements contained in the called function.

The pass also searches inside:

- `If`
- `While`
- `For`

blocks.

Parameterized calls and `Return` statements are not handled by this pass.

### Collection Layout

The current backend uses `malloc` for heap allocation of arrays, tuples, and lists.

Their layouts are:

**Array (`Ore`)**

| Offset    | Contents     |
| --------- | ------------ |
| 0         | Length       |
| 8         | Element size |
| 16 onward | Element data |

Each element currently occupies 8 bytes.

**List (`Materials`)**

| Offset | Contents       |
| ------ | -------------- |
| 0      | Length         |
| 8      | Capacity       |
| 16     | Buffer pointer |

**Tuple (`Ore` with named fields)**

Fields are stored starting at offset 0 in declaration order.

Each field currently occupies 8 bytes.

These layouts are implementation details of the current backend and may change in future compiler versions.

## Linking

Furnace generates a native object file.

The object file is linked using the system C compiler.

For example:

```fish
cc main.o -o main -lm
```

The linker produces the final executable.

---

[← Previous](semantic-analysis.md)
[Next →](code-generation.md)
