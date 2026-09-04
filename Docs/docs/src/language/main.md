# The Main Function

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

If a program does not contain a `Main` function, Furnace cannot produce an executable. The semantic analyzer checks that the `Main` function has the correct shape before code generation begins.

The `Main` function is treated specially: the `Stop` statement is not allowed inside it, since there is no enclosing loop or conditional from which to break early. To terminate a program early, use [`Program.Stop()`](program-namespace.md).

---

[← Previous](functions.md)
[Next →](variables.md)
