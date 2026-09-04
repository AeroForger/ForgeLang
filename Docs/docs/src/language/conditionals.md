# Conditionals

ForgeLang supports:

* `If`
* `Else If`
* `Else`

## If

```forge
If (I < 10)
{
    Print("Less than ten");
}
```

## Else

```forge
If (I < 10)
{
    Print("Small");
}
Else
{
    Print("Large");
}
```

## Else If

```forge
If (I < 10)
{
    Print("Small");
}
Else If (I < 100)
{
    Print("Medium");
}
Else
{
    Print("Large");
}
```

Conditions must evaluate to a valid condition for the current compiler. In other words, a condition must be a type the compiler can evaluate as truthy or falsey in the current backend.

---

[← Previous](logical.md)
[Next →](while.md)
