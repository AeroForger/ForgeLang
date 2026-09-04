# Logical Operators

ForgeLang currently provides `And`, `Or`, and `Xor` as bitwise operators for integer values.

| Operator | Operation   |
| -------- | ----------- |
| `And`    | Bitwise AND |
| `Or`     | Bitwise OR  |
| `Xor`    | Bitwise XOR |

Precedence is:

```text
And > Or > Xor
```

Example:

```forge
Number Int A = 12;
Number Int B = 10;

Number Int C = A And B;
Number Int D = A Or B;
Number Int E = A Xor B;
```

These operators currently operate on integer operands. They are not yet a general-purpose boolean logic system for non-integer values.

---

[← Previous](comparisons.md)
[Next →](conditionals.md)
