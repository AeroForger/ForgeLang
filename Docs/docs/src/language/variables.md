# Variables

Variables are declared using a type, a name, and optionally an initial value.

Example:

```forge
Number Int I = 0;
```

A variable can later be assigned:

```forge
I = 10;
```

Variables can be modified multiple times:

```forge
I = I + 1;
I = I + 5;
```

## Declaration

General form:

```text
Type Name = Value;
```

Examples:

```forge
Number Int Age = 14;
Number Float Height = 181.0;
Weld Name = "ForgeLang";
Ore[3] Numbers = [10, 20, 30,];
Materials Int List = (1, 2, 3,);
```

A trailing comma is allowed in array and list literals.

## Assignment

General form:

```text
Name = Value;
```

Example:

```forge
Age = Age + 1;
```

The assigned value must be compatible with the variable's type. The semantic analyzer rejects assignments whose right-hand side does not match the declared type. For example, assigning a `Weld` value to a `Bool` variable produces a type error.

For more details on the available types, see [Types](types.md).

---

[← Previous](main.md)
[Next →](types.md)
