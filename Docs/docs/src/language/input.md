# Input

ForgeLang provides input through `Input`.

## Integer Input

```forge
Number Int Value = Input(Int);
```

This reads input as a string and parses it as an integer. An input that cannot be parsed as an integer produces an input error.

## Floating-Point Input

```forge
Number Float Value = Input(Float);
```

This reads input as a string and parses it as a floating-point value. An input that cannot be parsed as a floating-point value produces an input error.

## String Input

```forge
Weld Value = Input();
```

This reads a string from standard input.

The current implementation uses standard C input facilities internally.

---

[← Previous](interpolation.md)
[Next →](operators.md)
