# Visibility

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

## `Showcase`

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

[← Previous](comments.md)
[Next →](data.md)
