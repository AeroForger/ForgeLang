# ForgeLang Alpha 2 Documentation

Welcome to ForgeLang Alpha 2. ForgeLang is a strictly typed, system level
programming language designed for flexibility and performance. It compiles
to native machine code using LLVM.

This document is the authoritative source for Alpha 1 syntax, architecture,
and usage.

## 1. The Furnace Compiler

The compiler for ForgeLang is called Furnace. It handles lexing, parsing,
abstract syntax tree generation, and LLVM intermediate representation
generation, then invokes clang to produce a native executable.

### 1.1 File Extensions
ForgeLang source files use the `.anvil` extension.

### 1.2 Installation

Requires Python 3.10+ and clang.

```bash
pip install furnace-compiler
```

### 1.3 Compilation Commands
```bash
furnace main.anvil             # compile to main.out
furnace --emit-llvm main.anvil # print LLVM IR to stdout instead
furnace --keep-ll main.anvil   # keep the intermediate .ll file
furnace --version              # print compiler version
```

Status messages go to stderr. The `--emit-llvm` output on stdout is always
pure IR, suitable for piping.

## 2. General Program Structure

A ForgeLang source file consists of zero or more top level declarations.
These can be imports, data declarations, function declarations, object
declarations, or standard statements.

Example:
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
    Print(\V"Hello {John.name}!");
}
```

## 3. Comments

ForgeLang supports block style comments using `-[` and `]-`. They can span
multiple lines and are ignored by the lexer.

```forge
-[ This is a single line comment ]-

-[ 
    This is a
    multi line
    comment
]-
```

Comments may appear anywhere whitespace is permitted.

## 4. Imports

Imports parse in Alpha 1 but have no linking effect yet.

### 4.1 Namespace Import
```forge
Use Math;
Use System;
```

### 4.2 Member Import
```forge
Using System.Sort: MergeSort;
```

## 5. Keywords and Reserved Words

ForgeLang is strictly case sensitive. `Open` is not the same as `open`.

Access Modifiers: `Open`, `Closed`, `Showcase`
Declarations: `Number`, `Weld`, `Ore`, `Materials`, `Data`
Functions: `function`, `Nunction`, `Return`
Control Flow: `If`, `Else`, `Switch`, `Deal`, `Base`, `Do`, `Fail`, `Final`
Logic: `And`, `Or`, `Xor`
Imports: `Use`, `Using`
Other: `New`, `Print`, `Input`
Types: `Int`, `Float`, `Generic`

## 6. Access Modifiers

Declarations may optionally begin with an access modifier.

`Open`: Public accessibility.
`Closed`: Private accessibility.
`Showcase`: Read only or const like declaration.

```forge
Open Data Player { ... }
Closed Data InternalData { ... }
Showcase Number Int MaxPlayers = 100;
```

Modifier enforcement is a semantic analysis stage and is not yet active.

## 7. Primitive Types

### 7.1 Number
The `Number` type family requires an explicit subtype. The deprecated
syntax `Number X;` or `Int.Number X;` is rejected by the parser.

Valid subtypes are `Int`, `Float`, and `Generic`.

```forge
Number Int x = 10;
Number Float y = 3.14;
Number Generic z = 100;
```

`Int` compiles to a 32-bit integer. `Float` compiles to a 64-bit double.
Mixing `Int` and `Float` in an expression promotes the integer to double.

### 7.2 Weld
`Weld` represents a string.

```forge
Weld name = "ForgeLang";
```

Strings support escaped quotes: `"He said \"hi\""`.

### 7.3 Ore
`Ore` is a dynamic, general purpose variable type. It can also declare
fixed size arrays.

```forge
Ore result = x + y;
Ore[5] array;
```
Array sizes must be integer literals in Alpha 1.

### 7.4 Materials
`Materials` represents a collection or list type, conceptually similar to
`List<T>` in other languages.

```forge
Materials Int numbers;
Materials Int New IntList;
```
Full list operations are not yet implemented.

## 8. Variables and Assignment

Variables must be declared with a type. They can optionally be initialized.

```forge
Number Int x = 2;
Weld name = "John";
Ore result = x + y;
```

Assignment after declaration uses the equals sign.

```forge
Z = Y + X;
John.name = "John";
```

## 9. Operators

### 9.1 Arithmetic
`+`, `-`, `*`, `/`, `**`

`**` is right-associative: `2 ** 3 ** 2` is `2 ** 9` = 512.
Integer division truncates: `7 / 2` is `3`.

### 9.2 Unary
`+x`, `-x`

Unary minus binds looser than `**`: `-2 ** 2` is `-(2 ** 2)` = -4,
matching mathematical convention.

### 9.3 Comparison
`==`, `!=`, `<`, `>`, `<=`, `>=`

### 9.4 Logical
`And`, `Or`, `Xor`

Logical operators are non-short-circuit: both sides are always evaluated.
Non-zero values are truthy. The result is a boolean.

### 9.5 Operator Precedence
From highest to lowest:
1. Parentheses, function calls, member access
2. Exponentiation `**` (right-associative)
3. Unary `+` / `-`
4. Multiplication and Division
5. Addition and Subtraction
6. Comparison Operators
7. `And`
8. `Or`
9. `Xor`

### 9.6 Escape Sequences
Inside strings: `\n` newline, `\t` tab, `\r` carriage return, `\0` null,
`\\` backslash, `\"` quote. A literal `%` is printed as `%`; the compiler
handles printf escaping.

## 10. Control Flow

### 10.1 If / Else If / Else
```forge
If (x < y)
{
    Print("X is smaller");
}
Else If (x == y)
{
    Print("X equals Y");
}
Else
{
    Print("X is larger");
}
```

All three forms compile and execute in Alpha 0.2. `If` without `Else` is
valid; the body simply does not run when the condition is false.

### 10.2 Switch
Parses. Code generation is planned for Alpha 0.3.

### 10.3 Do / Fail / Final
Parses. Code generation is planned for Alpha 0.3.

## 11. Functions

Function declarations parse with parameters and return types. Calls with
arguments and return value code generation are the current work in progress
(end of Alpha 0.2). The declaration syntax is stable:

### 11.1 Typed Functions
```forge
Open Int Add(Int x, Int y)
{
    Return x + y;
}
```

### 11.2 Dynamic Return Functions
```forge
Open function ReturnAnything(Ore x)
{
    Return x;
}
```

### 11.3 No Return Functions
`Nunction` functions return nothing. Returning a value from a `Nunction`
is a semantic error, enforced by semantic analysis, not the parser.

```forge
Open Nunction Main()
{
    Print("Hello ForgeLang");
}
```

## 12. Data and Objects

### 12.1 Data Declarations
`Data` defines a user defined structure.

```forge
Open Data Player
{
    Weld name;
    Weld nickname;
    Number Int age;
}
```

### 12.2 Object Instantiation
Objects are instantiated with the type, a name, and an initialization block.

```forge
Player John
{
    John.name = "John";
    John.age = 25;
}
```

### 12.3 Member Access
Members are accessed with the dot operator, in assignments, expressions,
and string interpolation.

```forge
John.name = "John";
Print(John.name);
Print(\V"Hello {John.name}!");
```

## 13. Strings and Interpolation

Standard strings use double quotes. Interpolation uses `\V"` with curly
braces.

```forge
Weld name = "Forge";
Print(\V"Hello {name}!");
```

Only `\V"..."` strings interpolate. Plain `"..."` strings print `{name}`
literally. Interpolated names may be variables or object members like
`{John.name}`.

## 14. Built-in Functions

### 14.1 Print
Outputs to standard output, followed by a newline.
```forge
Print("Hello World");
Print(result);
Print(\V"Hello {name}!");
Print(x + y);
```

`Input` is a reserved keyword; its implementation is pending.

## 15. Compiler Architecture

The pipeline is strictly separated to ensure backend independence.

### 15.1 The Pipeline
1. Source File (`.anvil`)
2. Lexer (ANTLR generated)
3. Parser (ANTLR generated)
4. Parse Tree
5. AST Builder (visitor pattern)
6. ForgeLang AST
7. Semantic Analysis (seam; populated in Beta)
8. LLVM Code Generator (llvmlite)
9. LLVM IR
10. clang
11. Native Executable

### 15.2 AST Boundary
The generated parser produces a Parse Tree. A visitor constructs a clean
Abstract Syntax Tree containing no syntax noise. The AST is a stable,
independent representation: grammar changes do not affect later stages.

### 15.3 LLVM Code Generation
- `Number Int` maps to `i32`
- `Number Float` maps to `double`
- `Weld` maps to `i8*`
- `Data` maps to `LiteralStructType`
- Objects map to `GlobalVariable`
- Member access uses `getelementptr`
- `If` compiles to basic blocks with conditional branches

### 15.4 Backend Independence
By targeting LLVM IR, ForgeLang can use any LLVM backend. Future backends
may include direct C code generation or WebAssembly.

### 15.5 Testing
The compiler ships with a regression suite (`tests/run.sh`). Every language
feature is covered by a test that compiles, runs, and verifies output.
Contributions are expected to add tests.

## 16. Scope and Variable Resolution

The compiler maintains a scope stack.
- Global scope is index 0.
- Functions push a new local scope.
- Lookup checks local scope first, then global scope.

```forge
Player John
{
    John.name = "John";
}

Open Nunction Main()
{
    Print(\V"Hello {John.name}!");
}
```

## 17. Standard Library

Alpha 1 interfaces with the C standard library for basic operations. The
`printf` function is declared externally and backs `Print` and string
interpolation. `pow` from libm backs `**`.

## 18. Semantic Analysis (Roadmap)

Semantic analysis is a planned stage. Future checks include:
- Type checking (preventing `Weld` and `Number` addition)
- Scope validation (variables declared before use)
- Access modifier enforcement
- `Nunction` return value validation
- Argument count and type checking for calls

## 19. TBD Features

Pending final syntax decisions:
- `For` and `While` loop syntax
- Boolean literals
- Null or None values
- Scientific notation for floats
- Complete `Materials` initialization
- Array and list indexing syntax
- Increment and decrement operators
- Generic types beyond `Number Generic`
- Error and exception semantics for `Do`, `Fail`, `Final`
- Member function calls (`Object.Method()`)

Do not infer syntax from other languages for these features. They will be
explicitly defined in future specifications.

## 20. Changelog

- v0.2.1: pip installable package, CLI flags, clean stdout contract
- v0.2.0: If/Else If/Else codegen, And/Or/Xor logic, grammar v2
- v0.1.0: variables, arithmetic, strings, interpolation, structs