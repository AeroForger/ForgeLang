<p align="center">
  <img src="ForgeLangLogo128.png" width="160" alt="ForgeLang logo">
</p>

# ForgeLang

A systems programming language, compiled to native machine code through LLVM.

ForgeLang is forged by **Furnace**, its compiler. Source files use the `.anvil` extension.

```forge
Open Data Player
{
    Weld name;
}

Player John
{
    John.name = "John";
}

Open Nunction Main()
{
    Number Int x = 10;
    Weld name = "ForgeLang";
    Print(\V"Hello {John.name}!");
}
```

## Status: Alpha 0.2

- Variables (`Number Int`, `Number Float`, `Weld`, `Ore`)
- Arithmetic with correct precedence, unary minus, `**` exponentiation
- Strings with escapes and `\V"..."` interpolation
- `Data` structs, object instances, member access
- `If` / `Else If` / `Else`, `And` / `Or` / `Xor`
- Native executables via LLVM IR + clang

## Install

Requires: Python 3.10+, clang

```bash
pip install furnace-compiler
```

## Compile

```bash
furnace hello.anvil        # -> hello.out
furnace --emit-llvm hello.anvil   # print LLVM IR instead
```

## Architecture

```
.anvil source
    ↓  ANTLR lexer/parser
Parse tree
    ↓  AST builder (visitor)
ForgeLang AST
    ↓  llvmlite
LLVM IR
    ↓  clang
Native executable
```

The parser is generated from a formal grammar. The AST is a separate,
stable representation — the grammar may change without breaking the
compiler's later stages. Backend-independent by design.

## Roadmap

- [x] Alpha 0.1 — variables, arithmetic, strings, print
- [x] Alpha 0.2 — control flow (If/Else)
- [ ] Alpha 0.2.1 — functions, parameters, recursion
- [ ] Alpha 0.3 — Switch/Deal/Base, Do/Fail/Final
- [ ] 1.0 — first stable release
- [ ] 2.0 — self-hosting (Furnace written in ForgeLang)

## License

Apache 2.0
