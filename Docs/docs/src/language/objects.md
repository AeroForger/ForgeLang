# Object Instantiation

Instances of `Data` types use object declaration syntax.

Example:

```forge
Person Den
{
    Age = 14;
    Name = "Den";
}
```

Members can also be separated using commas:

```forge
Person Den { Age = 14, Name = "Den"; }
```

Nested member paths are supported by the parser:

```forge
Person Den { Address.City = "Den", Age = 14; }
```

Object declarations are represented as `Statement::ObjectDecl`.

The current backend does not generate executable code for object declarations.

They are currently checked for structural validity.

---

[← Previous](data.md)
[Next →](imports.md)
