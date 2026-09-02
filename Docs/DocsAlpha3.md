# ForgeLang Alpha 3 - Language Documentation

**ForgeLang** is a strictly typed, native systems programming language designed around explicit control, predictable execution, and low-level performance.

ForgeLang source files use the `.anvil` extension. If you drop one on the floor, it will not make a sound.

The compiler is **Furnace**.

> **Status:** Alpha 3
> **Compiler:** Furnace
> **Implementation:** Rust
> **Parser:** pest
> **Backend:** Cranelift
> **Output:** Native object code

---

# Table of Contents

1. [Introduction](#1-introduction)
2. [Hello World](#2-hello-world)
3. [Program Structure](#3-program-structure)
4. [Functions](#4-functions)
   4.1. [`Nunction`](#41-nunction)
   4.2. [`function`](#42-function)
   4.3. [Function Parameters](#43-function-parameters)
   4.4. [Zero-Argument Call Inlining](#44-zero-argument-call-inlining)
5. [The Main Function](#5-the-main-function)
6. [Variables](#6-variables)
   6.1. [Declaration](#61-declaration)
   6.2. [Assignment](#62-assignment)
7. [Types](#7-types)
   7.1. [Primitive Types](#71-primitive-types)
   7.2. [Arrays](#72-arrays)
   7.3. [Tuples](#73-tuples)
   7.4. [Lists](#74-lists)
   7.5. [Generic Types](#75-generic-types)
8. [Integers](#8-integers)
9. [Floating-Point Numbers](#9-floating-point-numbers)
10. [Strings](#10-strings)
    10.1. [String Escapes](#101-string-escapes)
11. [String Interpolation](#11-string-interpolation)
12. [Input](#12-input)
13. [Operators](#13-operators)
    13.1. [Arithmetic](#131-arithmetic)
    13.2. [Power](#132-power)
    13.3. [Increment and Decrement](#133-increment-and-decrement)
14. [Unary Operators](#14-unary-operators)
15. [Comparisons](#15-comparisons)
16. [Logical Operators](#16-logical-operators)
17. [Conditionals](#17-conditionals)
18. [While Loops](#18-while-loops)
19. [For Loops](#19-for-loops)
20. [The `Stop` Statement](#20-the-stop-statement)
21. [The `Program` Namespace](#21-the-program-namespace)
22. [Function Calls](#22-function-calls)
23. [Recursion](#23-recursion)
24. [Scope](#24-scope)
25. [Comments](#25-comments)
26. [Visibility](#26-visibility)
    26.1. [`Showcase`](#261-showcase)
27. [Data Declarations](#27-data-declarations)
28. [Object Instantiation](#28-object-instantiation)
29. [Imports](#29-imports)
30. [Semantic Analysis](#30-semantic-analysis)
31. [Compiler Architecture](#31-compiler-architecture)
32. [Compilation](#32-compilation)
33. [Complete Example](#33-complete-example)
34. [Current Limitations](#34-current-limitations)
35. [Alpha 3 Roadmap](#35-alpha-3-roadmap)

---

# 1. Introduction

ForgeLang is designed to provide a strongly typed programming environment while compiling directly to native machine code.

Unlike interpreted languages, ForgeLang does not require a virtual machine or interpreter to execute a compiled program.

The basic compilation pipeline is:

ForgeLang source -> pest -> AST -> Semantic Analysis -> Cranelift -> Native Object -> System Linker -> Executable

Furnace is written in Rust.

---

# 2. Hello World

A minimal ForgeLang program is:

```forge
Open Nunction Main()
{
    Print("Hello, World!");
}
```

`Main` is the entry point of the program.

`Print` writes a value to standard output.

---

# 3. Program Structure

A ForgeLang program consists of declarations and statements.

A typical program looks like:

```forge
Open Nunction Main()
{
    Number Int I = 0;

    While (I < 10)
    {
        Print(\V"{I}");
        I = I + 1;
    }
}
```

ForgeLang uses braces `{}` to delimit function and control-flow bodies.

Statements are terminated with `;`.

Declarations include function definitions, variable declarations, data type definitions, and import statements. These can appear at the top level of a program file.

---

# 4. Functions

ForgeLang supports function declarations. The current backend fully supports zero-argument `Nunction` procedures through call inlining. Parameterized functions and value-returning functions are parsed and checked, but are not yet lowered to native code.

There are currently two function forms:

* `function`
* `Nunction`

## 4.1 `Nunction`

A `Nunction` is a function that does not return a value.

```forge
Nunction Tick()
{
    Print("tick");
}
```

It can be called from `Main` with:

```forge
Tick();
```

## 4.2 `function`

`function` is reserved for functions that return a value. Return-value code generation is not implemented in the current compiler.

```forge
function Add(Number Int A, Number Int B)
{
    return A + B;
}
```

A returned value can be used in an expression:

```forge
Number Int Result = Add(10, 20);
```

> Function return syntax and return-type syntax are still part of the evolving Alpha specification.

## 4.3 Function Parameters

The grammar accepts parameter declarations:

```forge
Nunction PrintNumber(Number Int Value)
{
    Print(\V"{Value}");
}
```

Call the function with:

```forge
PrintNumber(42);
```

Furnace validates function argument counts during semantic analysis. Parameterized calls are currently rejected before code generation. The parser is slightly ahead of the backend, as parsers occasionally are.

## 4.4 Zero-Argument Call Inlining

The current backend does not implement a native calling convention for user-defined functions. Instead, zero-argument `Nunction` calls are inlined directly into the caller's body during code generation. This means `Tick()` above will work, but `PrintNumber(42)` will not.

This inlining is transparent. The `expand_function_calls` pass in `src/codegen.rs` walks the `Main` function body and replaces each eligible call with the callee's statements. Recursive inlining is not supported, and the callee must be a zero-argument `Nunction`. For loops are the only thing that goes up but never comes down, unless you use `--` instead.

---

# 5. The Main Function

Every executable ForgeLang program requires a `Main` entry point.

The standard form is:

```forge
Open Nunction Main()
{
    // program
}
```

`Open` controls visibility.

`Main` is the function used as the executable entry point. The compiler searches for a function named `Main` and uses it as the root of code generation.

---

# 6. Variables

Variables are declared using a type followed by a name and optional initial value.

Example:

```forge
Number Int I = 0;
```

A variable can subsequently be assigned:

```forge
I = 10;
```

Variables may be modified multiple times:

```forge
I = I + 1;
I = I + 5;
```

## 6.1 Declaration

General form:

```text
Type Name = Value;
```

Examples:

```forge
Number Int Age = 14;
Number Float Height = 181.0;
Weld Name = "ForgeLang";
Ore[3] Numbers = [10, 20, 30,];
Materials Int List = (1, 2, 3,);
```

## 6.2 Assignment

General form:

```text
Name = Value;
```

Example:

```forge
Age = Age + 1;
```

The assigned value must be compatible with the variable's type.

---

# 7. Types

Alpha 3 provides primitive types, fixed-size arrays, named-field tuples, growable lists, and generic collections.

## 7.1 Primitive Types

The primary primitive types are:

| Type           | Purpose                |
| -------------- | ---------------------- |
| `Number Int`   | Integer numbers        |
| `Number Float` | Floating-point numbers |
| `Weld`         | Strings                |

ForgeLang is strictly typed.

## 7.2 Arrays

Arrays are declared with the `Ore` keyword followed by a size in brackets.

A fixed-size array with an explicit size:

```forge
Ore[3] FixedNums = [10, 20, 30,];
```

An array with an inferred size uses the `EMPTY` keyword:

```forge
Ore[EMPTY] InferredNums = [100, 200, 300, 400,];
```

Array literals are written in square brackets. A trailing comma is permitted.

Elements can be read by index:

```forge
Print(FixedNums[0]);
```

Elements can be assigned by index:

```forge
FixedNums[1] = 55;
```

Arrays expose a `.Length` property:

```forge
Print(FixedNums.Length);
```

The semantic analyzer rejects arrays whose initializer count does not match the declared size. Arrays are fixed in size, much like your patience at a compiler bug.

## 7.3 Tuples

Tuples are declared with `Ore` followed by a parenthesized list of named fields, each specifying a subtype and a field name.

```forge
Ore(Int Number1, Int Number2) TwoNumbers = {1, 2};
```

Tuple elements are initialized with curly-brace literals:

```forge
Ore(Int Age, Weld Name) Person = {14, "Den"};
```

Individual fields are accessed by name:

```forge
Print(Person.Age);
Print(Person.Name);
```

Fields can also be assigned directly:

```forge
Person.Age = 15;
```

The semantic analyzer checks that the number of initializer elements matches the number of declared fields and that each element's type is compatible with its declared field.

## 7.4 Lists

Lists are growable, heap-allocated collections declared with the `Materials` keyword followed by an element subtype.

A list initialized with values uses parenthesized syntax:

```forge
Materials Int Numbers = (10, 20, 30,);
```

An empty list is declared with the `new` keyword:

```forge
Materials Int new EmptyList;
```

`EmptyList` starts with zero elements and a capacity of four. It can grow at runtime, unlike your disk space when you forget to clean up temp files.

Elements are read and assigned the same way as arrays:

```forge
Print(Numbers[0]);
Numbers[1] = 50;
```

Lists expose `.Length` (or `.Len`) for the current element count:

```forge
Print(Numbers.Length);
```

List methods:

| Method          | Description                                        |
| --------------- | -------------------------------------------------- |
| `.Add(value)`   | Appends an element, growing capacity if needed     |
| `.Remove(index)` | Removes the element at `index` and shifts remaining elements |
| `.RemoveAt(index)` | Alias for `Remove`                              |

Example:

```forge
Numbers.Add(40);
Print(Numbers[3]);
Print(Numbers.Length);

Numbers.Remove(0);
Print(Numbers[0]);
Print(Numbers.Length);
```

## 7.5 Generic Types

The `Generic` subtype allows type-erased collections. A generic list:

```forge
Materials Generic new Items;
Items.Add(999);
Items.Add(1234);
Print(Items[0]);
Print(Items[1]);
```

Generic lists store all elements as integers internally. They do not yet support mixed-type storage in a type-checked way.

---

# 8. Integers

Integer variables use:

```forge
Number Int
```

Example:

```forge
Number Int Counter = 0;
```

Integer arithmetic supports:

```forge
Number Int A = 10;
Number Int B = 5;

Number Int Add = A + B;
Number Int Subtract = A - B;
Number Int Multiply = A * B;
Number Int Divide = A / B;
```

Integer variables can also be modified directly:

```forge
Counter = Counter + 1;
```

---

# 9. Floating-Point Numbers

Floating-point values use:

```forge
Number Float
```

Example:

```forge
Number Float Temperature = 21.5;
```

Floating-point arithmetic supports the standard arithmetic operators:

```forge
Number Float A = 10.5;
Number Float B = 2.0;

Number Float Result = A + B;
```

---

# 10. Strings

Strings use the `Weld` type.

Example:

```forge
Weld Name = "ForgeLang";
```

A string literal is written using double quotes:

```forge
"Hello"
```

Strings can be passed to `Print`:

```forge
Print("Hello, World!");
```

## 10.1 String Escapes

String literals support the following escape sequences:

| Sequence | Meaning          |
| -------- | ---------------- |
| `\n`     | Newline          |
| `\t`     | Tab              |
| `\r`     | Carriage return  |
| `\0`     | Null byte        |
| `\\`     | Backslash        |
| `\"`     | Double quote     |

Example:

```forge
Weld Line = "Hello\nWorld";
```

---

# 11. String Interpolation

ForgeLang supports variable interpolation using the `\V` string form.

Example:

```forge
Number Int I = 42;

Print(\V"{I}");
```

The value of `I` is inserted into the string.

Multiple values can be interpolated:

```forge
Number Int A = 10;
Number Int B = 20;

Print(\V"A = {A}, B = {B}");
```

This allows values to be converted into printable strings without manually constructing the output. Member access is also supported in interpolation expressions:

```forge
Ore(Int Age, Weld Name) Person = {14, "Den"};
Print(\V"{Person.Name} is {Person.Age}");
```

---

# 12. Input

ForgeLang provides typed input through `Input`.

## 12.1 Integer Input

```forge
Number Int Value = Input(Int);
```

This reads an integer from standard input.

## 12.2 Floating-Point Input

```forge
Number Float Value = Input(Float);
```

This reads a floating-point value.

## 12.3 String Input

```forge
Weld Value = Input(Weld);
```

This reads a string from standard input.

The current implementation uses standard C input facilities internally.

---

# 13. Operators

Alpha 3 supports arithmetic operators including:

## 13.1 Arithmetic

| Operator | Operation      |
| -------- | -------------- |
| `+`      | Addition       |
| `-`      | Subtraction    |
| `*`      | Multiplication |
| `/`      | Division       |
| `**`     | Power          |

Example:

```forge
Number Int A = 10;
Number Int B = 5;

Number Int C = A + B;
Number Int D = A - B;
Number Int E = A * B;
Number Int F = A / B;
```

## 13.2 Power

The `**` operator represents exponentiation.

```forge
Number Int Result = 2 ** 8;
```

Power expressions are right-associative.

Conceptually:

```text
A ** B ** C
```

is interpreted as:

```text
A ** (B ** C)
```

Power operations are lowered through the compiler's power implementation, which calls the C `pow` function.

## 13.3 Increment and Decrement

The `++` and `--` operators are postfix increment and decrement operators. They are currently only valid in the increment clause of a `For` loop header.

```forge
For (Number Int I = 0; I < 10; I++)
{
    Print(\V"{I}");
}
```

Using `--` decrements the loop variable:

```forge
For (Number Int I = 10; I > 0; I--)
{
    Print(\V"{I}");
}
```

---

# 14. Unary Operators

Alpha 3 supports unary negation and unary plus.

Example:

```forge
Number Int Value = -10;
```

It can also be applied to an expression:

```forge
Number Int Result = -(A + B);
```

Unary plus is a no-op that exists for symmetry:

```forge
Number Int Value = +42;
```

---

# 15. Comparisons

Comparison operators can be used in conditional expressions and loops.

The supported comparison operators are:

| Operator | Meaning               |
| -------- | --------------------- |
| `<`      | Less than             |
| `>`      | Greater than          |
| `<=`     | Less than or equal    |
| `>=`     | Greater than or equal |
| `==`     | Equal                 |
| `!=`     | Not equal             |

Example:

```forge
If (A < B)
{
    Print("A is smaller");
}
```

---

# 16. Logical Operators

Alpha 3 supports bitwise `And`, `Or`, and `Xor` operators. These operate on integer operands as bitwise operations.

| Operator | Operation       |
| -------- | --------------- |
| `And`    | Bitwise AND     |
| `Or`     | Bitwise OR      |
| `Xor`    | Bitwise XOR     |

Precedence is: `And` binds tighter than `Or`, and `Or` binds tighter than `Xor`.

Example:

```forge
Number Int A = 12;
Number Int B = 10;
Number Int C = A And B;
Number Int D = A Or B;
Number Int E = A Xor B;
```

---

# 17. Conditionals

ForgeLang supports:

* `If`
* `Else If`
* `Else`

## 17.1 If

```forge
If (I < 10)
{
    Print("Less than ten");
}
```

## 17.2 Else

```forge
If (I < 10)
{
    Print("Small");
}
Else
{
    Print("Large");
}
```

## 17.3 Else If

```forge
If (I < 10)
{
    Print("Small");
}
Else If (I < 100)
{
    Print("Medium");
}
Else
{
    Print("Large");
}
```

Conditions must evaluate to a valid boolean condition.

---

# 18. While Loops

ForgeLang supports `While` loops.

Basic syntax:

```forge
While (Condition)
{
    // body
}
```

Example:

```forge
Number Int I = 0;

While (I < 10)
{
    I = I + 1;
}
```

The condition is evaluated before each iteration.

## 18.1 Nested Loops

`While` loops can be nested.

```forge
Number Int I = 0;
Number Int V = 0;

While (I < 100)
{
    V = 0;

    While (V < 100)
    {
        V = V + 1;
    }

    I = I + 1;
}
```

Nested loops can be used for computational workloads and are compiled into native control flow. Normal ForgeLang `While` loops still execute on one thread. A loop does not become multicore merely because the compiler owns a fast laptop.

---

# 19. For Loops

ForgeLang supports `For` loops with a C-style header.

Basic syntax:

```forge
For (Init; Condition; Increment)
{
    // body
}
```

Example:

```forge
For (Number Int I = 0; I < 10; I++)
{
    Print(\V"{I}");
}
```

The `For` loop has three parts separated by semicolons:

1. **Init** - a variable declaration that runs once before the loop starts
2. **Condition** - an expression evaluated before each iteration; the loop continues while it is non-zero
3. **Increment** - a variable name followed by `++` or `--`, applied after each iteration body

Decrementing is also supported:

```forge
For (Number Int I = 10; I > 0; I--)
{
    Print(\V"{I}");
}
```

A `For` loop without braces produces an error.

---

# 20. The `Stop` Statement

The `Stop` statement provides a structured early exit from a loop or conditional block. It behaves like a `break` that jumps to the exit of the nearest enclosing loop or top-level `If` statement.

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

The semantic analyzer enforces two rules for `Stop`:

* it cannot be used inside `Main`
* it can only be used inside a loop or `If` statement

In the current implementation, `Stop` is represented as `Statement::Stop` in `src/ast.rs`, checked by `src/semantic.rs`, and lowered by `FunctionCompiler::compile_statement` in `src/codegen.rs`.

This is not a general-purpose exception system. It is a structured early exit encoded by the compiler's block-flow logic.

---

# 21. The `Program` Namespace

The `Program` namespace provides access to runtime-level operations.

Currently, the only member is:

```forge
Program.Stop();
```

This terminates the program immediately by calling the C `exit` function with a status of 0.

This is treated as a namespace call, not a normal function call. The semantic pass enforces two rules:

* it can only be used inside a function scope (not at the top level)
* it must be called with zero arguments

This is a deliberate early-exit mechanism for the runtime, and it is validated before code generation.

---

# 22. Function Calls

Functions are called using their name followed by parentheses.

A function with no arguments:

```forge
Tick();
```

A function with arguments:

```forge
PrintNumber(42);
```

Multiple arguments:

```forge
Add(10, 20);
```

Zero-argument calls are inlined during code generation. Calls with arguments are parsed and semantically validated but are not yet lowered to native code.

Furnace validates function calls during semantic analysis. Invalid argument counts are rejected before code generation.

---

# 23. Recursion

The grammar can represent recursive functions, but the current backend does not yet generate parameterized or return-value functions. Recursion is therefore planned rather than executable in this release.

Example:

```forge
function Countdown(Number Int I)
{
    If (I > 0)
    {
        Print(\V"{I}");
        Countdown(I - 1);
    }
}
```

Native recursive calls will use Cranelift's function calling mechanism once the function ABI is implemented.

---

# 24. Scope

Variables declared inside a block belong to that scope.

Example:

```forge
Open Nunction Main()
{
    Number Int I = 10;

    If (I > 0)
    {
        Number Int V = 20;
    }
}
```

`V` is local to the block in which it is declared.

The AST represents nested blocks, but the current backend uses a function-level variable map. Full lexical scope, shadowing, and capture rules remain under development.

---

# 25. Comments

ForgeLang supports single-line comments using `//`.

Example:

```forge
// This is a comment

Number Int I = 0;
```

Comments are ignored by the compiler.

---

# 26. Visibility

ForgeLang uses visibility modifiers to control which declarations are externally accessible.

The primary visibility keywords are:

* `Open`
* `Closed`
* `Showcase`

An externally visible function can be declared:

```forge
Open Nunction Main()
{
}
```

A declaration without external visibility is not automatically exposed as part of the public interface.

The module system and complete visibility rules are still under development.

## 26.1 `Showcase`

`Showcase` is a third visibility modifier. It is currently parsed and stored in the AST but does not yet affect code generation. It is reserved for future use, where it will mark declarations as suitable for export to documentation or REPL introspection.

```forge
Showcase Nunction Helper()
{
    Print("helper");
}
```

---

# 27. Data Declarations

ForgeLang supports declaring structured data types using the `Data` keyword.

```forge
Data Person
{
    Number Int Age;
    Weld Name;
}
```

A `Data` declaration defines a named type with a set of typed fields. Each field is declared with a type and a name, terminated by `;`.

The `Data` keyword can be preceded by a visibility modifier:

```forge
Open Data Point
{
    Number Int X;
    Number Int Y;
}
```

Data declarations are parsed and included in the AST, but the current backend does not yet generate code for them. They are validated for well-formedness but cannot yet be instantiated or used in code generation.

---

# 28. Object Instantiation

Once a `Data` type is declared, instances are created using object declaration syntax. The type name is followed by the variable name, then a brace block of member initializations.

```forge
Person Den
{
    Age = 14;
    Name = "Den";
}
```

Members are initialized by name, separated by semicolons or commas:

```forge
Person Den { Age = 14, Name = "Den"; }
```

Nested member paths are also supported:

```forge
Person Den { Address.City = "Den", Age = 14; }
```

Object declarations are parsed and included in the AST as `Statement::ObjectDecl`, but the current backend ignores them for code generation. They are validated for well-formedness.

---

# 29. Imports

ForgeLang supports importing modules using `Use` and `Using`.

## 29.1 Use

```forge
Use std.io;
```

`Use` imports an entire module by its qualified path.

## 29.2 Using

```forge
Using std.io: Print;
```

`Using` imports a specific item from a module. The item name follows the module path, separated by `:`.

The module system and complete `Use` / `Using` implementation are still under development. These statements are parsed and stored in the AST but do not yet affect code generation.

---

# 30. Semantic Analysis

Before code generation, Furnace performs semantic validation.

Alpha 3 introduces a dedicated semantic-analysis stage.

The semantic stage is responsible for detecting errors such as:

* Undefined variables
* Invalid function calls
* Incorrect function argument counts
* Invalid `Main` parameters
* Invalid access to shared members
* Forbidden shared-member mutation
* `Stop` used outside a loop or `If`
* `Stop` used inside `Main`
* `Program.Stop()` used outside a function
* `Program.Stop()` called with arguments
* `Program.Stop()` used with an unknown namespace
* Unknown methods on collections
* Array size mismatches
* Tuple field count mismatches
* List element type mismatches
* Empty lists declared with `new` that have initializers
* Other semantic violations

Semantic analysis occurs **before** Cranelift code generation.

This prevents invalid programs from reaching the backend.

## 30.1 Parallel Semantic Analysis

Furnace uses **Rayon** to provide a parallel semantic-analysis infrastructure.

The architecture is designed so independent portions of the AST can be analyzed concurrently. Independent function declarations are analyzed concurrently with Rayon. Normal ForgeLang `While` loops still execute on one thread. A loop does not become multicore merely because the compiler owns a fast laptop.

---

# 31. Compiler Architecture

Furnace is divided into several major stages.

## 31.1 Parser

The parser is implemented using **pest**, a PEG parser generator for Rust.

The parser converts ForgeLang source code into structured syntax.

.anvil source ->
   pest ->
    AST

The grammar uses explicit precedence rules rather than left-recursive expression rules. Operator precedence, from highest to lowest, is:

primary -> postfix -> power -> unary -> multiplicative -> additive -> comparison -> and -> or -> xor

A key documented decision: unary binds looser than `**`, so `-2 ** 2` equals `-4`.

## 31.2 AST

The AST is represented by strongly typed Rust structures.

It provides a stable intermediate representation between parsing and code generation.

Conceptually:

Source
->
Parser
->
AST
->
Semantic Analysis
->
Codegen

## 31.3 Semantic Analysis

`src/semantic.rs` validates the AST before code generation.

This stage is responsible for language-level correctness.

## 31.4 Code Generation

`src/codegen.rs` lowers ForgeLang constructs into **Cranelift IR**.

Cranelift handles:

* Machine-code generation
* Instruction selection
* Register allocation
* Target-specific code generation
* Object-file generation

The resulting output is a native object file.

### 31.4.1 Function Call Expansion

Before generating code for `Main`, the compiler runs `expand_function_calls` from `src/codegen.rs`. This pass inlines zero-argument `Nunction` calls into the call site. It also recurses into `If`, `While`, and `For` bodies to expand calls within those constructs. Parameterized calls and `Return` statements are not yet handled by this pass.

### 31.4.2 Collection Layout

Arrays, tuples, and lists are heap-allocated using `malloc`. Their in-memory layouts are:

**Array** (`Ore`):
* Offset 0: length (pointer-sized)
* Offset 8: element size (pointer-sized)
* Offset 16 onwards: element data (each element is 8 bytes)

**List** (`Materials`):
* Offset 0: length (pointer-sized)
* Offset 8: capacity (pointer-sized)
* Offset 16: buffer pointer (pointer-sized)

**Tuple** (`Ore` with named fields):
* Offset 0 onwards: field data (each field is 8 bytes, in declaration order)

## 31.5 Linking

The generated object file is linked using the system C compiler/linker.

For example:

```fish
cc main.o -o main -lm
```

The linker combines the generated object with the required system libraries and produces the final executable.

---

# 32. Compilation

## 32.1 Build Furnace

Furnace is built using Cargo.

```fish
cargo build --release
```

The resulting compiler is located at:

```text
target/release/furnace
```

## 32.2 Compile a ForgeLang Program

The current CLI flow is implemented explicitly in the `Cli/` modules and uses the library compiler under the hood.

```fish
./target/debug/furnace compile main.anvil linux
```

This workflow does the following:

1. validates the file ends in `.anvil`
2. reads and parses the source
3. runs semantic validation
4. runs function-call expansion (inlining zero-arg Nunctions)
5. emits a native object file
6. invokes the platform linker with `cc -lm`
7. prints the final executable path

Example output:

```text
Compiling main.anvil...
Linking...
Build successful!
Output: ./main
```

The CLI uses the `Platform` enum in `Cli/platform.rs` so new targets can be added without rewriting the command layer. At the moment, the supported target is `linux`.

## 32.3 Run a ForgeLang Program

The CLI also supports direct execution:

```fish
./target/debug/furnace run main.anvil
```

This command compiles the `.anvil` file, links it, and then executes the generated binary while forwarding stdout, stderr, and the child exit code.

## 32.4 Version and Help

The compiler exposes centralized version metadata from `src/lib.rs`:

```rust
pub const VERSION: &str = "Alpha 3";
```

The commands are:

```fish
./target/debug/furnace -version
./target/debug/furnace -help
```

Which emit:

```text
Furnace Alpha 3
```

And the usage summary:

```text
Usage:
    Furnace compile <file>.anvil <platform>
    Furnace run <file>.anvil
    Furnace -version
    Furnace -help
```

## 32.5 Link the Object Manually

The underlying object generation still produces a native object, and the linker is still the system C toolchain:

```fish
cc main.o -o main -lm
```

## 32.6 Run

```fish
./main
```

---

# 33. Complete Example

The following program demonstrates the intended Alpha 3 syntax for several features.

* Functions
* `Nunction`
* Variables
* `While`
* `For`
* `If`
* `Else`
* Arithmetic
* Function calls (zero-argument, inlined)
* String interpolation
* Arrays
* Tuples
* Lists
* `Stop`

```forge
Nunction Tick()
{
    Print("tick");
}

Open Nunction Main()
{
    // Array
    Ore[3] FixedNums = [10, 20, 30,];
    Print(FixedNums[0]);
    FixedNums[1] = 55;
    Print(FixedNums[1]);
    Print(FixedNums.Length);

    // Tuple
    Ore(Int Age, Weld Name) Person = {14, "Den"};
    Print(Person.Age);
    Print(Person.Name);
    Person.Age = 15;
    Print(Person.Age);

    // List
    Materials Int Numbers = (10, 20, 30,);
    Print(Numbers[0]);
    Print(Numbers.Length);
    Numbers.Add(40);
    Print(Numbers[3]);
    Numbers.Remove(0);
    Print(Numbers[0]);

    // While loop
    Number Int I = 0;
    While (I < 5)
    {
        Print(\V"{I}");
        I = I + 1;
    }

    // For loop
    For (Number Int J = 0; J < 3; J++)
    {
        Print(\V"J = {J}");
    }

    // Stop inside a loop
    Number Int K = 0;
    While (K < 10)
    {
        If (K == 3)
        {
            Stop;
        }
        Print(\V"{K}");
        K = K + 1;
    }

    // Zero-argument Nunction call (inlined)
    Tick();

    Number Int A = 12;
    Number Int B = 10;
    Print(A And B);
    Print(A Or B);
}
```

---

# 34. Current Limitations

Alpha 3 is an early development release.

The following features are **not yet part of the complete Alpha 3 language**:

* Parameterized function code generation (only zero-argument calls are inlined)
* Function return-value code generation
* `Return` statements
* `Data` declaration code generation (parsed but not emitted)
* `Object` instantiation code generation (parsed but not emitted)
* `Switch` / `Deal` / `Base` pattern matching (parsed but not emitted)
* `Do` / `Fail` / `Final` error-handling blocks (parsed but not emitted)
* `Use` / `Using` module imports (parsed but not functional)
* Complete module system
* Multicore ForgeLang runtime execution
* Garbage collection (Scrap is planned)
* Self-hosting Furnace
* Complete systems-level standard library

These features are planned for future releases.

---

# 35. Alpha 3 Roadmap

Alpha 3 establishes the new Rust-based compiler architecture.

## Short Term

### `Switch` / Pattern Matching

Planned constructs include:

```text
Switch
Deal
Base
```

These will eventually provide pattern-matching functionality.

### Error Handling

Planned constructs:

```text
Do
Fail
Final
```

These will provide structured error handling.

### Generic Data Types

The `Generic` subtype is currently limited to lists. Full generic data type support, including generic function parameters, is planned.

## Mid Term

### Module System

ForgeLang will gain a module/import system based around:

```text
Use
Using
```

Modules will integrate with:

```text
Open
Closed
Showcase
```

visibility rules.

### Parameterized Functions

Current code generation inlines only zero-argument `Nunction` calls. A proper calling convention with stack frames, parameter passing, and return values is planned.

### Multicore Runtime

Alpha 3 currently uses Rayon for **parallel compiler analysis**.

Future versions will introduce explicit mechanisms for ForgeLang programs to execute work across multiple CPU cores.

The goal is to distinguish:

```text
Parallel compilation
```

from:

```text
Parallel program execution
```

Explicit `Spawn` / `Join` syntax is planned. The intended safety model uses isolated tasks with explicit arguments and results rather than shared mutable globals competing for the same memory.

## Long-Term Ecosystem

### Scrap

**Scrap** will be ForgeLang's garbage collector.

Scrap is planned to be **disabled by default**.

The default ForgeLang model is intended to retain explicit control over memory rather than requiring garbage collection for every program.

Scrap can therefore eventually provide a higher-level memory-management option without making garbage collection mandatory.

### Ironwork

**Ironwork** will be the ForgeLang package manager.

Its purpose will be to provide:

* Package management
* Dependency resolution
* Library distribution
* Project management
* ForgeLang ecosystem integration

### Self-Hosting

The long-term goal is for Furnace to eventually be rewritten in ForgeLang itself.

This is targeted for the **2.0 generation** of ForgeLang.

The intended progression is:

Furnace written in Rust
        ->
ForgeLang becomes more capable
        ->
ForgeLang standard library matures
        ->
Compiler components become expressible in ForgeLang
        ->
Furnace rewritten in ForgeLang
        ->
Self-hosted ForgeLang compiler

---

# Alpha 3 Philosophy

Alpha 3 represents a fundamental change in ForgeLang's implementation.

Earlier experimental implementations relied on:

```text
Python
ANTLR
LLVM
```

Alpha 3 moves to:

```text
Rust
pest
Cranelift
```

The result is a substantially smaller and more direct compiler architecture:

ForgeLang
->
Rust
->
pest
->
AST
->
Semantic Analysis
->
Cranelift
->
Native Object Code

The goal of ForgeLang is to provide a language and ecosystem capable of supporting:

* Strong static typing
* Native execution
* Low-level control
* Explicit memory management
* Optional garbage collection
* Multicore execution
* A native package ecosystem
* Eventually, a self-hosted compiler

Alpha 3 is an architectural foundation, not a claim that every feature is finished. The compiler can currently parse more of the language than it can generate, which is a traditional way for a young compiler to keep its developers humble.