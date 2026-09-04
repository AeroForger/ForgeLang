# Operators

Alpha 3.4 supports arithmetic, comparison, bitwise, unary, and loop increment operators.

## Arithmetic

| Operator | Operation      |
| -------- | -------------- |
| `+`      | Addition       |
| `-`      | Subtraction    |
| `*`      | Multiplication |
| `/`      | Division       |
| `**`     | Power          |

Example:

```forge
Number Int A = 10;
Number Int B = 5;

Number Int C = A + B;
Number Int D = A - B;
Number Int E = A * B;
Number Int F = A / B;
```

## Power

The `**` operator performs exponentiation.

```forge
Number Int Result = 2 ** 8;
```

Power expressions are right-associative.

For example:

```text
A ** B ** C
```

is interpreted as:

```text
A ** (B ** C)
```

The current implementation lowers power operations through the C `pow` function.

## Increment and Decrement

`++` and `--` are postfix operators currently used in the increment section of a `For` loop.

Example:

```forge
For (Number Int I = 0; I < 10; I++)
{
    Print(\V"{I}");
}
```

Decrementing is also supported:

```forge
For (Number Int I = 10; I > 0; I--)
{
    Print(\V"{I}");
}
```

See [Unary Operators](unary.md), [Comparisons](comparisons.md), and [Logical Operators](logical.md) for the related operator families.

---

[← Previous](input.md)
[Next →](unary.md)
