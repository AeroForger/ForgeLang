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

You can also install Furnace using Cargo:

```fish
cd ForgeLang
cargo install --path .
```

After installation, Cargo places `furnace` in its executable path, allowing you to invoke it directly without specifying `target/release/furnace`.

For example:

```fish
furnace compile main.anvil linux
furnace run main.anvil
furnace backend native
furnace backend cranelift
furnace new console -n Project
furnace -help
furnace --help
furnace -version
furnace --version
```

This means you can use `furnace` directly from any directory, provided Cargo's binary directory is available in your `PATH`.

Documentation will use `cargo build --release` instead of `cargo install --path .` 

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
6. Selects the direct native path or the Cranelift path.
7. Generates functions and calls.
8. Writes an ELF64 executable directly, or creates an object file and invokes the platform linker.
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
2. Writes the direct executable or links the generated object.
3. Executes the resulting binary.
4. Forwards the program's standard output and standard error.
5. Returns the child process exit code.

## Check a Backend

Use the `backend` command to check either available backend:

```fish
./target/debug/furnace backend native
./target/debug/furnace backend cranelift
```

Available backends:

- `native` writes a direct x86-64 ELF64 executable.
- `cranelift` writes a native object file and links it with `cc`.

Use `--backend` with `compile` or `run` to choose the path for that command:

```fish
./target/debug/furnace compile main.anvil linux --backend native
./target/debug/furnace run main.anvil --backend cranelift
```

## Version and Help

Furnace exposes its version through centralized compiler metadata.

The current version is:

```rust
pub const VERSION: &str = "Alpha 5";
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
Furnace Alpha 5
```

Usage:

```text
Usage:
    Furnace compile <file>.anvil <platform> [--backend native|cranelift]
    Furnace run <file>.anvil [--backend native|cranelift]
    Furnace backend <native|cranelift>
    Furnace new <APP_TYPE> -n <NAME>
    Furnace -version
    Furnace -help

Available backends:
    native: direct x86-64 ELF64 executable
    cranelift: native object file linked with cc
```

## Create a Project

Create a console project with:

```fish
./target/debug/furnace new console -n Project
```

The command creates `Project/Project.anvil` with this source:

```forge
Open Nunction Main()
{
}
```

The supported application type is `console`. Furnace rejects unknown types, empty names, and existing project directories.

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
