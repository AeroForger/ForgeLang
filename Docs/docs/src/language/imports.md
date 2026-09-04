# Imports

ForgeLang provides `Use` and `Using` syntax for imports.

## `Use`

```forge
Use System.Math;
```

`Use` specifies a module path.

## `Using`

```forge
Using System.Math: Sqrt();
```

`Using` specifies an item from a module.

The module system is not yet fully implemented.

These statements can be parsed and stored in the AST, but they do not currently provide a working module system during code generation.

---

[← Previous](objects.md)
[Next →](../compiler/semantic-analysis.md)
