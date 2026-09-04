# Strings

Strings use the `Weld` type.

Example:

```forge
Weld Name = "ForgeLang";
```

String literals use double quotes:

```forge
"Hello"
```

Strings can be passed to `Print`:

```forge
Print("Hello, World!");
```

## String Escapes

String literals support:

| Sequence | Meaning         |
| -------- | --------------- |
| `\n`     | Newline         |
| `\t`     | Tab             |
| `\r`     | Carriage return |
| `\0`     | Null byte       |
| `\\`     | Backslash       |
| `\"`     | Double quote    |

Example:

```forge
Weld Line = "Hello\nWorld";
```

See [String Interpolation](interpolation.md) for embedding values into strings.

---

[← Previous](floating-point.md)
[Next →](interpolation.md)
