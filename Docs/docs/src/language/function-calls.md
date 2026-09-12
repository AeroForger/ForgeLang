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

Furnace compiles calls as native calls to independently compiled Sydrogen functions.

Semantic analysis checks the function name, argument count, and argument types before code generation.

See [Functions](functions.md) and [Recursion](recursion.md) for the current implementation status.

---

[← Previous](program-namespace.md)
[Next →](recursion.md)
