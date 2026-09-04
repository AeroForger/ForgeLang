# String Interpolation

ForgeLang supports interpolation using the `\V` string form.

Example:

```forge
Number Int I = 42;

Print(\V"{I}");
```

The value of `I` is inserted into the string.

Multiple values can be used:

```forge
Number Int A = 10;
Number Int B = 20;

Print(\V"A = {A}, B = {B}");
```

Member access is also supported:

```forge
Ore(Int Age, Weld Name) Person = {14, "Den"};

Print(\V"{Person.Name} is {Person.Age}");
```

The interpolation form is recognized by the lexer through the `\V` prefix. Inside the string, expressions enclosed in `{ }` are evaluated and rendered as their string representation.

---

[← Previous](strings.md)
[Next →](input.md)
