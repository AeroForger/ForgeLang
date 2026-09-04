# Alpha 3.4 Roadmap

Alpha 3.4 continues development of the Rust-based Furnace compiler.

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

- Parameter passing
- Return values
- Stack management
- Function frames
- A defined calling convention

### Multicore Runtime

Alpha 3.4 uses Rayon for compiler-side parallel analysis.

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

- Package management
- Dependency resolution
- Library distribution
- Project management
- ForgeLang package integration

### Self-Hosting

A long-term goal is to rewrite Furnace in ForgeLang itself.

This is targeted for the **2.0 generation** of ForgeLang.

The compiler will need sufficient language features, standard library support, and tooling before this becomes practical.

---

[← Previous](limitations.md)
[Next →](implementation-notes.md)
