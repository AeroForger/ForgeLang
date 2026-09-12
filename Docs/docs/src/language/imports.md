# Imports

Sydrogen provides `Use` and `Using` for importing modules and symbols. Imports
are resolved before semantic analysis and work with both Native and Cranelift.

## `Use`

```forge
Use File;

Open Nunction Main()
{
    File.Function1();
}
```

`Use` imports a module while preserving its namespace. It does not place the
module's functions directly into the importing scope, so `Function1()` alone is
not made valid by `Use File;`.

## `Using`

```forge
Using File: Function1;

Open Nunction Main()
{
    Function1();
}
```

`Using` imports one specific symbol directly into the current module's scope.
It does not expose the rest of the module.

The difference is:

```forge
Use File;
File.Function1();

Using File: Function1;
Function1();
```

## Module discovery

During a `.blower` build, the module table contains only `.anvil` files matched
by the project's `Files.location` entries. A file that exists on disk but is
excluded from the project cannot be imported. Each module name initially comes
from its filename: `src/File.anvil` defines module `File`. Duplicate filenames
that would create the same module name are rejected.

During standalone compilation, imports are loaded relative to the importing
source file, not the shell's working directory. For example, `Use File;` in
`Main.anvil` loads the sibling `File.anvil`.

Qualified paths are structural and may identify nested source paths:

```forge
Use System.Math;
```

For standalone compilation this resolves `System/Math.anvil` beside the
importing file. In a project it must match a declared project source whose path
ends in `System/Math.anvil`. This does not imply that a standard-library module
exists.

## Visibility and conflicts

Only `Open` declarations can be imported across a module boundary. Private or
`Closed` declarations remain usable inside their own module but cannot be
reached through `Use` or `Using`. Functions are callable through either import
form; existing named declarations such as `Data` types can be selected with
`Using` where their normal language syntax permits their use.

Repeated identical imports are idempotent. Furnace rejects selective imports
that give two different symbols the same local name, imports that conflict with
a local declaration, and ambiguous duplicate module names.

Missing modules, missing symbols, and inaccessible symbols are reported before
code generation. Imports must appear at module top level.

## Dependency graph

Furnace builds one dependency graph for the loaded modules. Nested dependencies
are resolved once and keep their scope: importing `File` does not re-export
symbols that `File` imported internally. Direct and longer cycles are rejected,
for example:

```text
error: circular import detected: A -> B -> C -> A
```

Imports do not download packages or resolve external registries. All imported
modules must be local standalone sources or declared `.blower` project members.

---

[← Previous](objects.md)
[Next →](../compiler/semantic-analysis.md)
