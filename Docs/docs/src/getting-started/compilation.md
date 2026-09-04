# Compilation

This page covers the practical steps of building Furnace and using it to compile a ForgeLang program.

## Build Furnace

Furnace is built using Cargo:

```fish
cargo build --release
```

The release compiler is located at:

```text
target/release/furnace
```

## Compile a ForgeLang Program

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

## Run a ForgeLang Program

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

## Version and Help

Furnace exposes its version through centralized compiler metadata.

The current version is:

```rust
pub const VERSION: &str = "Alpha 3.4";
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
Furnace Alpha 3.4
```

Usage:

```text
Usage:
    Furnace compile <file>.anvil <platform>
    Furnace run <file>.anvil
    Furnace -version
    Furnace -help
```

## Link the Object Manually

Furnace produces a native object file that can be linked separately:

```fish
cc main.o -o main -lm
```

The exact libraries required may depend on the generated program and target platform.

## Run the Resulting Executable

The resulting executable can be started normally:

```fish
./main
```

---

[← Previous](program-structure.md)
[Next →](../language/functions.md)
