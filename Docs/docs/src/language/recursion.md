# Recursion

The grammar can represent recursive functions.

Example:

```forge
function Countdown(Int I)
{
    If (I > 0)
    {
        Print(\V"{I}");
        Countdown(I - 1);
    }
}
```

Recursive calls use the same native function call path as other calls. A recursive function can have parameters and can return a value.

---

[← Previous](function-calls.md)
[Next →](scope.md)
