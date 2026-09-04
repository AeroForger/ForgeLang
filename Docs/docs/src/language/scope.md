# Scope

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

This is an example of a feature that can be represented at the AST level without being fully executable in the backend yet.

---

[← Previous](recursion.md)
[Next →](comments.md)
