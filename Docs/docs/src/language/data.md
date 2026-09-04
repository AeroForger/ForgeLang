# Data Declarations

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

`Data` declarations are currently parsed and represented in the AST.

The current backend does not generate executable code for `Data` declarations.

---

[← Previous](visibility.md)
[Next →](objects.md)
