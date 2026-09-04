# For Loops

ForgeLang supports C-style `For` loops.

Basic syntax:

```forge
For (Init; Condition; Increment)
{
    // body
}
```

Example:

```forge
For (Number Int I = 0; I < 10; I++)
{
    Print(\V"{I}");
}
```

A `For` loop has three parts:

1. `Init` - runs once before the loop.
2. `Condition` - is evaluated before each iteration.
3. `Increment` - runs after each iteration.

The increment expression currently uses `++` or `--`.

Example:

```forge
For (Number Int I = 10; I > 0; I--)
{
    Print(\V"{I}");
}
```

The increment variable must be a numeric variable. The loop variable is not in scope while its initializer is being evaluated.

`Skip` continues with the next iteration of the loop. `Stop` exits the current loop. `Skip` can only be used inside a loop.

A `For` loop without braces produces an error.

---

[← Previous](while.md)
[Next →](stop.md)
