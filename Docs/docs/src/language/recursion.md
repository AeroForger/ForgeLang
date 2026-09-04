# Recursion

The grammar can represent recursive functions.

Example:

```forge
function Countdown(Number Int I)
{
    If (I > 0)
    {
        Print(\V"{I}");
        Countdown(I - 1);
    }
}
```

The current backend does not yet generate parameterized function calls or return-value functions.

As a result, general recursive functions are not currently executable.

Native recursive calls are planned once the compiler has a function calling convention.

---

[← Previous](function-calls.md)
[Next →](scope.md)
