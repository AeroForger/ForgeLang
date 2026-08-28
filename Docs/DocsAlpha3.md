# ForgeLang Alpha 3 - Language Documentation

**ForgeLang** is a strictly typed, native systems programming language designed around explicit control, predictable execution, and low-level performance.

ForgeLang source files use the `.anvil` extension.

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
5. [The Main Function](#5-the-main-function)
6. [Variables](#6-variables)
7. [Types](#7-types)
8. [Integers](#8-integers)
9. [Floating-Point Numbers](#9-floating-point-numbers)
10. [Strings](#10-strings)
11. [String Interpolation](#11-string-interpolation)
12. [Input](#12-input)
13. [Operators](#13-operators)
14. [Unary Operators](#14-unary-operators)
15. [Comparisons](#15-comparisons)
16. [Conditionals](#16-conditionals)
17. [While Loops](#17-while-loops)
18. [Function Calls](#18-function-calls)
19. [Recursion](#19-recursion)
20. [Scope](#20-scope)
21. [Comments](#21-comments)
22. [Visibility](#22-visibility)
23. [Semantic Analysis](#23-semantic-analysis)
24. [Compiler Architecture](#24-compiler-architecture)
25. [Compilation](#25-compilation)
26. [Complete Example](#26-complete-example)
27. [Current Limitations](#27-current-limitations)
28. [Alpha 3 Roadmap](#28-alpha-3-roadmap)

---

# 1. Introduction

ForgeLang is designed to provide a strongly typed programming environment while compiling directly to native machine code.

Unlike interpreted languages, ForgeLang does not require a virtual machine or interpreter to execute a compiled program.

The basic compilation pipeline is:


ForgeLang source -> pest -> AST -> Semantic Analysis -> Cranelift -> Native Object -> System Linker
-> Executable


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

---

# 4. Functions

ForgeLang supports function declarations. The current backend fully supports zero-return `Nunction` procedures called from `Main`. Parameterized functions and value-returning functions are parsed and checked, but are not yet lowered to native code.

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

`Main` is the function used as the executable entry point.

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

Example:

```forge
Number Int Age = 14;
Number Float Height = 181.0;
Weld Name = "ForgeLang";
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

Alpha 3 currently provides the following primary types:

| Type           | Purpose                |
| -------------- | ---------------------- |
| `Number Int`   | Integer numbers        |
| `Number Float` | Floating-point numbers |
| `Weld`         | Strings                |

ForgeLang is strictly typed.

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

This allows values to be converted into printable strings without manually constructing the output.

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

The current implementation uses standard C input facilities internally.

---

# 13. Operators

Alpha 3 supports arithmetic operators including:

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

## 13.1 Power

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

Power operations are lowered through the compiler's power implementation.

---

# 14. Unary Operators

Alpha 3 supports unary negation.

Example:

```forge
Number Int Value = -10;
```

It can also be applied to an expression:

```forge
Number Int Result = -(A + B);
```

---

# 15. Comparisons

Comparison operators can be used in conditional expressions and loops.

The supported comparison operators include:

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

# 16. Conditionals

ForgeLang supports:

* `If`
* `Else If`
* `Else`

## 16.1 If

```forge
If (I < 10)
{
    Print("Less than ten");
}
```

## 16.2 Else

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

## 16.3 Else If

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

# 17. While Loops

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

## 17.1 Nested Loops

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

Nested loops can be used for computational workloads and are compiled into native control flow.

---

# 18. Function Calls

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

Furnace validates function calls during semantic analysis.

Invalid argument counts are rejected before code generation.

---

# 19. Recursion

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

# 20. Scope

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

# 21. Comments

ForgeLang supports single-line comments using `//`.

Example:

```forge
// This is a comment

Number Int I = 0;
```

Comments are ignored by the compiler.

---

# 22. Visibility

ForgeLang uses visibility modifiers to control which declarations are externally accessible.

The primary visibility keywords are:

* `Open`
* `Closed`

An externally visible function can be declared:

```forge
Open Nunction Main()
{
}
```

A declaration without external visibility is not automatically exposed as part of the public interface.

The module system and complete visibility rules are still under development.

---

# 23. Semantic Analysis

Before code generation, Furnace performs semantic validation.

Alpha 3 introduces a dedicated semantic-analysis stage.

The semantic stage is responsible for detecting errors such as:

* Undefined variables
* Invalid function calls
* Incorrect function argument counts
* Invalid `Main` parameters
* Invalid access to shared members
* Forbidden shared-member mutation
* Other semantic violations

Semantic analysis occurs **before Cranelift code generation**.

This prevents invalid programs from reaching the backend.

## 23.1 Parallel Semantic Analysis

Furnace uses **Rayon** to provide a parallel semantic-analysis infrastructure.

The architecture is designed so independent portions of the AST can be analyzed concurrently.

Parallelism here refers to **compiler analysis**. Independent function declarations are analyzed concurrently with Rayon. Normal ForgeLang `While` loops still execute on one thread; a loop does not become multicore merely because the compiler owns a fast laptop.

---

# 24. Compiler Architecture

Furnace is divided into several major stages.

## 24.1 Parser

The parser is implemented using **pest**, a PEG parser generator for Rust.

The parser converts ForgeLang source code into structured syntax.

.anvil source ->
   pest ->
    AST


The grammar uses explicit precedence rules rather than left-recursive expression rules.

## 24.2 AST

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


## 24.3 Semantic Analysis

`semantic.rs` validates the AST before code generation.

This stage is responsible for language-level correctness.

## 24.4 Code Generation

`codegen.rs` lowers ForgeLang constructs into **Cranelift IR**.

Cranelift handles:

* Machine-code generation
* Instruction selection
* Register allocation
* Target-specific code generation
* Object-file generation

The resulting output is a native object file.

## 24.5 Linking

The generated object file is linked using the system C compiler/linker.

For example:

```fish
cc main.o -o main -lm
```

The linker combines the generated object with the required system libraries and produces the final executable.

---

# 25. Compilation

## 25.1 Build Furnace

Furnace is built using Cargo.

```fish
cargo build --release
```

The resulting compiler is located at:

```text
target/release/furnace
```

## 25.2 Compile a ForgeLang Program

Given:

```text
main.anvil
```

run:

```fish
./target/release/furnace main.anvil
```

Without `-o`, Furnace generates:

```text
main.o
```

When an executable output is requested, Furnace invokes the system linker automatically:

```fish
./target/release/furnace main.anvil -o main -lm
```

## 25.3 Link the Object Manually

Use the system linker:

```fish
cc main.o -o main -lm
```

## 25.4 Run

```fish
./main
```

---

# 26. Complete Example

The following program demonstrates the intended Alpha 3 syntax for several features. Parameterized function calls remain ahead of the current backend.

* Functions
* `Nunction`
* Parameters
* Variables
* `While`
* `If`
* `Else`
* Arithmetic
* Function calls
* String interpolation

```forge
Nunction Tick()
{
    Print("tick");
}

Nunction PrintNumber(Number Int Value)
{
    Print(\V"{Value}");
}

Open Nunction Main()
{
    Number Int I = 0;
    Number Int A = 0;
    Number Int B = 0;

    While (I < 10)
    {
        A = 0;
        B = 0;

        While (A < 100)
        {
            If (A < 50)
            {
                B = B + 3;
            }
            Else
            {
                B = B + 7;
            }

            A = A + 1;
        }

        Tick();
        PrintNumber(B);

        I = I + 1;
    }

    Print(\V"Final I = {I}");
}
```

---

# 27. Current Limitations

Alpha 3 is an early development release.

The following features are **not yet part of the complete Alpha 3 language**:

* `For` loops
* Parameterized function code generation
* Function return-value code generation
* General native function calls
* Pattern matching
* `Switch`
* `Deal`
* `Base`
* `Do` / `Fail` / `Final` error-handling blocks
* Complete module system
* Full `Use` / `Using` implementation
* Multicore ForgeLang runtime execution
* Garbage collection
* Self-hosting Furnace
* Complete systems-level standard library

These features are planned for future releases.

---

# 28. Alpha 3 Roadmap

Alpha 3 establishes the new Rust-based compiler architecture.

## Short Term

### `For` Loops

Add a dedicated `For` loop construct.

Possible syntax is still being designed.

### Pattern Matching

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

---

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
```

visibility rules.

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

Explicit `Spawn`/`Join` syntax is planned. The intended safety model uses isolated tasks with explicit arguments and results rather than shared mutable globals competing for the same memory.
```

---

# Long-Term Ecosystem

## Scrap

**Scrap** will be ForgeLang's garbage collector.

Scrap is planned to be **disabled by default**.

The default ForgeLang model is intended to retain explicit control over memory rather than requiring garbage collection for every program.

Scrap can therefore eventually provide a higher-level memory-management option without making garbage collection mandatory.

---

## Ironwork

**Ironwork** will be the ForgeLang package manager.

Its purpose will be to provide:

* Package management
* Dependency resolution
* Library distribution
* Project management
* ForgeLang ecosystem integration

---

## Self-Hosting

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
