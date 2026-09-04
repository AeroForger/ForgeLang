# Integers

Integer values use:

```forge
Number Int
```

Example:

```forge
Number Int Counter = 0;
```

Integer arithmetic supports:

```forge
Number Int A = 10;
Number Int B = 5;

Number Int Add = A + B;
Number Int Subtract = A - B;
Number Int Multiply = A * B;
Number Int Divide = A / B;
```

Integer variables can be modified:

```forge
Counter = Counter + 1;
```

Integer values are also used as the underlying storage for [Generic](types.md#generic-types) list elements in the current implementation.

See [Operators](operators.md) for the full list of arithmetic, power, and bitwise operations.

---

[← Previous](types.md)
[Next →](floating-point.md)
