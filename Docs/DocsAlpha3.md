# ForgeLang Alpha 3.2 Language Documentation

**ForgeLang** is a statically typed systems programming language that compiles to native machine code.

ForgeLang source files use the `.anvil` extension. If you drop one on the floor, it will not make a sound.

The compiler is **Furnace**.

> **Status:** Alpha 3.2
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
35. [Alpha 3.2 Roadmap](#35-alpha-32-roadmap)

---

# 1. Introduction

ForgeLang is a statically typed programming language that compiles source code to native machine code.

Compiled ForgeLang programs do not require a virtual machine or interpreter at runtime.

The compiler pipeline is:

```text
ForgeLang source -> pest -> AST -> Semantic Analysis -> Cranelift -> Native Object -> System Linker -> Executable
```

Furnace is written in Rust.

Alpha 3.2 uses:

* **Rust** for the compiler
* **pest** for parsing
* **Cranelift** for code generation
* **Rayon** for parallel semantic analysis
* **cc** for linking

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

A simple program looks like:

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

Top-level declarations can include:

* Function definitions
* Variable declarations
* Data declarations
* Import statements

The exact behavior of some declarations depends on the current compiler implementation.

---

# 4. Functions

ForgeLang currently has two function forms:

* `Nunction`
* `function`

The current backend can inline zero-argument `Nunction` calls.

Parameterized functions and return-value functions are accepted by parts of the frontend but are not yet generated as native function calls.

## 4.1 `Nunction`

A `Nunction` is a function that does not return a value.

```forge
Nunction Tick()
{
    Print("tick");
}
```

It can be called with:

```forge
Tick();
```

Zero-argument `Nunction` calls can currently be expanded directly into the caller during code generation.

## 4.2 `function`

`function` is reserved for functions that return a value.

Example syntax:

```forge
function Add(Number Int A, Number Int B)
{
    return A + B;
}
```

A returned value can be used like this:

```forge
Number Int Result = Add(10, 20);
```

Return-value code generation is not currently implemented.

The syntax exists because eventually the compiler will have to deal with functions that actually return things. Otherwise `function` would be a rather optimistic keyword.

## 4.3 Function Parameters

The grammar accepts parameter declarations:

```forge
Nunction PrintNumber(Number Int Value)
{
    Print(\V"{Value}");
}
```

A call can be written as:

```forge
PrintNumber(42);
```

Furnace checks the number of supplied arguments during semantic analysis.

Parameterized calls are currently rejected before native code generation.

The parser is currently ahead of the backend in this area. The parser has seen the future. The backend has not.

## 4.4 Zero-Argument Call Inlining

The current backend does not yet use a native calling convention for user-defined functions.

Instead, zero-argument `Nunction` calls are expanded before code generation.

For example:

```forge
Nunction Tick()
{
    Print("tick");
}

Open Nunction Main()
{
    Tick();
}
```

The compiler can replace the `Tick()` call with the statements inside `Tick`.

The `expand_function_calls` pass in `src/codegen.rs` handles this transformation.

The pass can also process calls inside:

* `If`
* `While`
* `For`

Recursive inlining is not supported.

The callee must be a zero-argument `Nunction`.

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

`Main` is used as the root of executable code generation.

Furnace searches for a function named `Main`.

---

# 6. Variables

Variables are declared using a type, a name, and optionally an initial value.

Example:

```forge
Number Int I = 0;
```

A variable can later be assigned:

```forge
I = 10;
```

Variables can be modified multiple times:

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

Alpha 3.2 currently includes primitive types, arrays, tuples, lists, and a limited generic type.

## 7.1 Primitive Types

The primary primitive types are:

| Type           | Purpose               |
| -------------- | --------------------- |
| `Number Int`   | Integer values        |
| `Number Float` | Floating-point values |
| `Weld`         | String values         |

ForgeLang uses static type checking.

## 7.2 Arrays

Arrays use the `Ore` keyword followed by a size.

A fixed-size array:

```forge
Ore[3] FixedNums = [10, 20, 30,];
```

An array with an inferred size:

```forge
Ore[EMPTY] InferredNums = [100, 200, 300, 400,];
```

Array literals use square brackets.

A trailing comma is allowed.

Elements can be accessed by index:

```forge
Print(FixedNums[0]);
```

Elements can be assigned by index:

```forge
FixedNums[1] = 55;
```

Arrays expose `.Length`:

```forge
Print(FixedNums.Length);
```

The semantic analyzer checks that an explicitly declared array size matches the initializer count.

Arrays have a fixed number of elements after creation.

Your array will not spontaneously grow because it has decided that four elements are not enough. That job belongs to `Materials`.

## 7.3 Tuples

Tuples use `Ore` followed by named fields.

Example:

```forge
Ore(Int Number1, Int Number2) TwoNumbers = {1, 2};
```

A tuple containing different field types:

```forge
Ore(Int Age, Weld Name) Person = {14, "Den"};
```

Fields are accessed by name:

```forge
Print(Person.Age);
Print(Person.Name);
```

Fields can be assigned:

```forge
Person.Age = 15;
```

The semantic analyzer checks the number and types of tuple initializer values.

## 7.4 Lists

Lists use the `Materials` keyword.

A list initialized with values:

```forge
Materials Int Numbers = (10, 20, 30,);
```

An empty list:

```forge
Materials Int new EmptyList;
```

An empty list starts with zero elements and an initial capacity of four.

Elements can be read and assigned by index:

```forge
Print(Numbers[0]);
Numbers[1] = 50;
```

Lists expose `.Length` and `.Len`:

```forge
Print(Numbers.Length);
```

Available methods include:

| Method             | Description                                  |
| ------------------ | -------------------------------------------- |
| `.Add(value)`      | Adds an element to the list                  |
| `.Remove(index)`   | Removes an element and shifts later elements |
| `.RemoveAt(index)` | Alias for `.Remove(index)`                   |

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

`Generic` can be used as the element type of a list.

Example:

```forge
Materials Generic new Items;

Items.Add(999);
Items.Add(1234);

Print(Items[0]);
Print(Items[1]);
```

The current implementation stores generic list elements as integers internally.

Mixed-type generic storage is not currently implemented as a type-checked feature.

---

# 8. Integers

Integer values use:

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

Integer variables can be modified:

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

String literals use double quotes:

```forge
"Hello"
```

Strings can be passed to `Print`:

```forge
Print("Hello, World!");
```

## 10.1 String Escapes

String literals support:

| Sequence | Meaning         |
| -------- | --------------- |
| `\n`     | Newline         |
| `\t`     | Tab             |
| `\r`     | Carriage return |
| `\0`     | Null byte       |
| `\\`     | Backslash       |
| `\"`     | Double quote    |

Example:

```forge
Weld Line = "Hello\nWorld";
```

---

# 11. String Interpolation

ForgeLang supports interpolation using the `\V` string form.

Example:

```forge
Number Int I = 42;

Print(\V"{I}");
```

The value of `I` is inserted into the string.

Multiple values can be used:

```forge
Number Int A = 10;
Number Int B = 20;

Print(\V"A = {A}, B = {B}");
```

Member access is also supported:

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

Alpha 3.2 supports arithmetic, comparison, bitwise, unary, and loop increment operators.

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

The `**` operator performs exponentiation.

```forge
Number Int Result = 2 ** 8;
```

Power expressions are right-associative.

For example:

```text
A ** B ** C
```

is interpreted as:

```text
A ** (B ** C)
```

The current implementation lowers power operations through the C `pow` function.

## 13.3 Increment and Decrement

`++` and `--` are postfix operators currently used in the increment section of a `For` loop.

Example:

```forge
For (Number Int I = 0; I < 10; I++)
{
    Print(\V"{I}");
}
```

Decrementing is also supported:

```forge
For (Number Int I = 10; I > 0; I--)
{
    Print(\V"{I}");
}
```

---

# 14. Unary Operators

Alpha 3.2 supports unary negation and unary plus.

Example:

```forge
Number Int Value = -10;
```

Unary negation can also be applied to an expression:

```forge
Number Int Result = -(A + B);
```

Unary plus is a no-op:

```forge
Number Int Value = +42;
```

It exists because sometimes a language designer looks at unary minus and thinks, "why should minus get all the attention?"

---

# 15. Comparisons

Comparison operators can be used in conditions and loops.

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

ForgeLang currently provides `And`, `Or`, and `Xor` as bitwise operators for integer values.

| Operator | Operation   |
| -------- | ----------- |
| `And`    | Bitwise AND |
| `Or`     | Bitwise OR  |
| `Xor`    | Bitwise XOR |

Precedence is:

```text
And > Or > Xor
```

Example:

```forge
Number Int A = 12;
Number Int B = 10;

Number Int C = A And B;
Number Int D = A Or B;
Number Int E = A Xor B;
```

These operators currently operate on integer operands.

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

Conditions must evaluate to a valid condition for the current compiler.

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

The condition is evaluated before every iteration.

## 18.1 Nested Loops

`While` loops can be nested:

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

Nested loops are compiled to native control flow.

Normal ForgeLang `While` loops execute on one thread.

The compiler using multiple CPU cores for semantic analysis does not make the generated loop multicore. The compiler cannot simply yell "parallel!" at a loop and hope for the best.

---

# 19. For Loops

ForgeLang supports C-style `For` loops.

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

A `For` loop has three parts:

1. **Init**
   Runs once before the loop.

2. **Condition**
   Is evaluated before each iteration.

3. **Increment**
   Runs after each iteration.

The increment expression currently uses `++` or `--`.

Example:

```forge
For (Number Int I = 10; I > 0; I--)
{
    Print(\V"{I}");
}
```

A `For` loop without braces produces an error.

---

# 20. The `Stop` Statement

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

* `Stop` cannot be used inside `Main`
* `Stop` must be inside a loop or `If` statement

In the compiler, `Stop` is represented as `Statement::Stop` in `src/ast.rs`.

The semantic checks are implemented in `src/semantic.rs`.

Code generation is handled by `FunctionCompiler::compile_statement` in `src/codegen.rs`.

`Stop` is not an exception mechanism. It represents a structured early exit.

---

# 21. The `Program` Namespace

The `Program` namespace provides runtime operations.

The currently implemented member is:

```forge
Program.Stop();
```

This terminates the program by calling the C `exit` function with a status of `0`.

`Program.Stop()` is treated as a namespace operation rather than a normal user-defined function call.

The semantic analyzer checks that:

* It is used inside a function
* It has zero arguments
* The namespace and operation are valid

Example:

```forge
Open Nunction Main()
{
    Print("before");
    Program.Stop();
    Print("after");
}
```

The second `Print` is never reached.

---

# 22. Function Calls

Functions are called using their name followed by parentheses.

A zero-argument call:

```forge
Tick();
```

A call with arguments:

```forge
PrintNumber(42);
```

A call with multiple arguments:

```forge
Add(10, 20);
```

Currently, only zero-argument `Nunction` calls can be expanded into executable code by the backend.

Calls with arguments can be parsed and checked by the frontend, but are not yet lowered to native function calls.

Furnace checks function argument counts during semantic analysis.

---

# 23. Recursion

The grammar can represent recursive functions.

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

The current backend does not yet generate parameterized function calls or return-value functions.

As a result, general recursive functions are not currently executable.

Native recursive calls are planned once the compiler has a function calling convention.

---

# 24. Scope

Variables declared inside a block are intended to belong to that block.

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

`V` is declared inside the `If` block.

The AST represents nested blocks, while the current backend uses a function-level variable map.

Full lexical scope, shadowing, and capture rules are still under development.

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

ForgeLang uses visibility modifiers to control declaration visibility.

The current visibility keywords are:

* `Open`
* `Closed`
* `Showcase`

Example:

```forge
Open Nunction Main()
{
}
```

The complete module and visibility system is still under development.

## 26.1 `Showcase`

`Showcase` is a third visibility modifier.

It is currently parsed and stored in the AST but does not affect code generation.

It is reserved for future functionality related to exported declarations, documentation, or REPL introspection.

Example:

```forge
Showcase Nunction Helper()
{
    Print("helper");
}
```

---

# 27. Data Declarations

ForgeLang supports the `Data` keyword for declaring structured types.

Example:

```forge
Data Person
{
    Number Int Age;
    Weld Name;
}
```

A `Data` declaration contains typed fields.

The `Data` declaration can use a visibility modifier:

```forge
Open Data Point
{
    Number Int X;
    Number Int Y;
}
```

Data declarations are currently parsed and represented in the AST.

The current backend does not generate executable code for `Data` declarations.

---

# 28. Object Instantiation

Instances of `Data` types use object declaration syntax.

Example:

```forge
Person Den
{
    Age = 14;
    Name = "Den";
}
```

Members can also be separated using commas:

```forge
Person Den { Age = 14, Name = "Den"; }
```

Nested member paths are supported by the parser:

```forge
Person Den { Address.City = "Den", Age = 14; }
```

Object declarations are represented as `Statement::ObjectDecl`.

The current backend does not generate executable code for object declarations.

They are currently checked for structural validity.

---

# 29. Imports

ForgeLang provides `Use` and `Using` syntax for imports.

## 29.1 Use

```forge
Use std.io;
```

`Use` specifies a module path.

## 29.2 Using

```forge
Using std.io: Print;
```

`Using` specifies an item from a module.

The module system is not yet fully implemented.

These statements can be parsed and stored in the AST, but they do not currently provide a working module system during code generation.

---

# 30. Semantic Analysis

Furnace performs semantic analysis before code generation.

The semantic stage checks the parsed AST for invalid programs.

It currently handles checks including:

* Undefined variables
* Invalid function calls
* Incorrect function argument counts
* Invalid `Main` parameters
* Invalid shared-member access
* Forbidden shared-member mutation
* Invalid `Stop` usage
* Invalid `Program.Stop()` usage
* Unknown collection methods
* Array size mismatches
* Tuple field count mismatches
* List element type mismatches
* Invalid empty-list declarations
* Other language-level errors

Semantic analysis happens before Cranelift code generation.

This keeps invalid programs from being passed directly to the backend.

## 30.1 Parallel Semantic Analysis

Furnace uses **Rayon** for parallel semantic analysis.

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

# 31. Compiler Architecture

Furnace is divided into several stages.

## 31.1 Parser

The parser uses **pest**, a PEG parser generator for Rust.

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

## 31.2 AST

The AST is represented using Rust structures.

It acts as the representation shared between parsing, semantic analysis, and code generation.

The compiler works with the AST rather than passing raw source text between compiler stages.

## 31.3 Semantic Analysis

`src/semantic.rs` validates the AST before code generation.

This stage handles language-level checks such as:

* Type compatibility
* Variable lookup
* Function lookup
* Function argument counts
* Collection operations
* Scope-related checks
* Control-flow restrictions

## 31.4 Code Generation

`src/codegen.rs` converts supported ForgeLang constructs into **Cranelift IR**.

Cranelift handles:

* Instruction selection
* Register allocation
* Machine code generation
* Target-specific code generation
* Object-file generation

Furnace produces a native object file from the generated code.

### 31.4.1 Function Call Expansion

Before generating code for `Main`, Furnace runs `expand_function_calls`.

The pass is located in `src/codegen.rs`.

It replaces eligible zero-argument `Nunction` calls with the statements contained in the called function.

The pass also searches inside:

* `If`
* `While`
* `For`

blocks.

Parameterized calls and `Return` statements are not handled by this pass.

### 31.4.2 Collection Layout

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

## 31.5 Linking

Furnace generates a native object file.

The object file is linked using the system C compiler.

For example:

```fish
cc main.o -o main -lm
```

The linker produces the final executable.

---

# 32. Compilation

## 32.1 Build Furnace

Furnace is built using Cargo:

```fish
cargo build --release
```

The release compiler is located at:

```text
target/release/furnace
```

## 32.2 Compile a ForgeLang Program

The current CLI command is:

```fish
./target/debug/furnace compile main.anvil linux
```

The compile process:

1. Checks that the input file uses the `.anvil` extension.
2. Reads the source file.
3. Parses the source.
4. Builds the AST.
5. Performs semantic analysis.
6. Expands eligible zero-argument `Nunction` calls.
7. Generates a native object file.
8. Invokes the platform linker.
9. Produces the executable.

Example output:

```text
Compiling main.anvil...
Linking...
Build successful!
Output: ./main
```

The CLI uses the `Platform` enum in `Cli/platform.rs`.

The currently supported target is:

```text
linux
```

Additional targets can be added as compiler support is implemented.

## 32.3 Run a ForgeLang Program

The CLI can compile and execute a program directly:

```fish
./target/debug/furnace run main.anvil
```

This command:

1. Compiles the source.
2. Links the generated object.
3. Executes the resulting binary.
4. Forwards the program's standard output and standard error.
5. Returns the child process exit code.

## 32.4 Version and Help

Furnace exposes its version through centralized compiler metadata.

The current version is:

```rust
pub const VERSION: &str = "Alpha 3.2";
```

Version information can be requested with:

```fish
./target/debug/furnace -version
```

Help can be requested with:

```fish
./target/debug/furnace -help
```

Example version output:

```text
Furnace Alpha 3.2
```

Usage:

```text
Usage:
    Furnace compile <file>.anvil <platform>
    Furnace run <file>.anvil
    Furnace -version
    Furnace -help
```

## 32.5 Link the Object Manually

Furnace produces a native object file that can be linked separately:

```fish
cc main.o -o main -lm
```

The exact libraries required may depend on the generated program and target platform.

## 32.6 Run

The resulting executable can be started normally:

```fish
./main
```

---

# 33. Complete Example

The following program demonstrates several features available in Alpha 3.2:

* `Nunction`
* Variables
* Arrays
* Tuples
* Lists
* `While`
* `For`
* `If`
* `Else`
* Arithmetic
* String interpolation
* `Stop`
* Bitwise operations
* Zero-argument function calls

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

    // Zero-argument Nunction call
    Tick();

    // Bitwise operations
    Number Int A = 12;
    Number Int B = 10;

    Print(A And B);
    Print(A Or B);
    Print(A Xor B);
}
```

---

# 34. Current Limitations

Alpha 3.2 is an early development release.

The following features are not currently fully implemented in the backend:

* Parameterized function code generation
* Native calls to parameterized user-defined functions
* Function return-value code generation
* `Return` statements
* `Data` declaration code generation
* Object instantiation code generation
* `Switch` / `Deal` / `Base` pattern matching
* `Do` / `Fail` / `Final` error handling
* Functional `Use` / `Using` imports
* Complete module system
* Complete lexical scope handling
* Multicore ForgeLang program execution
* Garbage collection
* Self-hosting Furnace
* Complete systems-level standard library

Some of these features are already represented in the grammar or AST.

There is an important distinction:

**Parsed** means Furnace can recognize the syntax.

**Semantically checked** means Furnace can inspect the construct and report certain errors.

**Code generated** means Furnace can produce executable native code for the construct.

A feature being parsed does not mean that it can currently be used in an executable program.

This distinction saves everyone from discovering that the compiler supports something only after the compiler politely refuses to compile it.

---

# 35. Alpha 3.2 Roadmap

Alpha 3.2 continues development of the Rust-based Furnace compiler.

## Short Term

### `Switch` / Pattern Matching

Planned constructs include:

```text
Switch
Deal
Base
```

These are intended to provide pattern matching.

### Error Handling

Planned constructs include:

```text
Do
Fail
Final
```

These are intended to provide structured error handling.

### Generic Data Types

`Generic` is currently limited.

Future versions are planned to support generic data types and generic function parameters.

## Mid Term

### Module System

ForgeLang will gain a working module system based around:

```text
Use
Using
```

The module system will interact with:

```text
Open
Closed
Showcase
```

visibility rules.

### Parameterized Functions

The current backend handles zero-argument `Nunction` calls through inlining.

Future versions will introduce native function calls with:

* Parameter passing
* Return values
* Stack management
* Function frames
* A defined calling convention

### Multicore Runtime

Alpha 3.2 uses Rayon for compiler-side parallel analysis.

Future versions are planned to provide mechanisms for ForgeLang programs to execute work on multiple CPU cores.

Possible constructs include:

```text
Spawn
Join
```

The exact syntax and safety rules are not final.

The compiler's parallel analysis and a program's parallel execution are separate features.

## Long-Term Ecosystem

### Scrap

**Scrap** is planned as an optional garbage collector for ForgeLang.

It is intended to be disabled by default.

The default memory model is intended to keep memory management explicit.

Scrap would provide another memory-management option without making garbage collection mandatory.

### Ironwork

**Ironwork** is planned as the ForgeLang package manager.

Its planned responsibilities include:

* Package management
* Dependency resolution
* Library distribution
* Project management
* ForgeLang package integration

### Self-Hosting

A long-term goal is to rewrite Furnace in ForgeLang itself.

This is targeted for the **2.0 generation** of ForgeLang.

The compiler will need sufficient language features, standard library support, and tooling before this becomes practical.

---

# Alpha 3.2 Implementation Notes

Alpha 3.2 uses a different compiler implementation from the earlier experimental versions of ForgeLang.

Earlier versions used:

```text
Python
ANTLR
LLVM
```

Alpha 3.2 uses:

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

Alpha 3.2 should not be treated as a finished language specification.

Some syntax exists before its backend implementation.

Some AST structures exist before their code generation.

Some planned language features are already represented in the parser even though the compiler cannot execute them yet.

That is normal for a compiler under active development.

<<<<<<< HEAD
For now, Furnace can compile a growing subset of ForgeLang to native code while the rest of the language catches up.
=======
For now, Furnace can compile a growing subset of ForgeLang to native code while the rest of the language catches up.
>>>>>>> 42d5220 (Alpha 3.2 Docs and Readme Update)
