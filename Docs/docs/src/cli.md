# CLI Reference

This page documents the commands exposed by the `furnace` binary.

## Update

The `update` command installs the latest tagged release of Furnace from its source repository.

```fish
furnace update
```

The command:

1. Checks that `git` and `cargo` are available.
2. Clones the Furnace repository into a temporary directory.
3. Fetches tags and selects the latest tag sorted by version.
4. Checks out that tag.
5. Runs `cargo install --path . --locked` in the checked-out tree.
6. Replaces the existing `furnace` binary on the `PATH`.

The repository URL is defined as `FURNACE_REPO` in `Cli/commands/update.rs`.

Example output:

```text
Downloading source...
Latest tag: v0.4.0
Installing Furnace...
Furnace updated successfully.
```

Failure modes:

- `git` or `cargo` is missing from the `PATH`.
- The repository cannot be cloned.
- No tags are found in the repository.
- `cargo install` fails to build or link.

On failure the command exits with a non-zero code and prints an error message.

## Other Commands

```text
Usage:
    furnace compile <file>.anvil <platform> [--backend native|cranelift]
    furnace run <file>.anvil [--backend native|cranelift]
    furnace backend <native|cranelift>
    furnace new <APP_TYPE> -n <NAME>
    furnace update
    furnace -version
    furnace -help
```