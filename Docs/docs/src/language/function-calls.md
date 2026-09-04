# Function Calls

Functions are called using their name followed by parentheses.

A zero-argument call:

```forge
Tick();
```

A call with arguments:

```forge
PrintNumber(42);
```

A call with multiple arguments:

```forge
Add(10, 20);
```

Currently, only zero-argument `Nunction` calls can be expanded into executable code by the backend.

Calls with arguments can be parsed and checked by the frontend, but are not yet lowered to native function calls.

Furnace checks function argument counts during semantic analysis.

See [Functions](functions.md) and [Recursion](recursion.md) for the current implementation status.

---

[← Previous](program-namespace.md)
[Next →](recursion.md)
