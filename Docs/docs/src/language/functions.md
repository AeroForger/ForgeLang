# Functions

ForgeLang functions are declared with either:

* `Nunction` for a function that does not return a value
* A return type, such as `Int`, `Float`, `Weld`, `Bool`, `Ore`, or `Materials`, for a function that returns data

Furnace compiles each user-defined function as an independent native function. Calls use the function's declared parameter and return types.

## `Nunction`

A `Nunction` is a function that does not return a value.

```forge
Nunction Tick()
{
    Print("tick");
}
```

It can be called with:

```forge
Tick();
```

`Nunction` calls do not return a value. They can take parameters and can call other functions.

## Returning Functions

A returning function declares its return type before its name. It sends a value back to its caller with the `Return` keyword.

For example, this function declares an `Int` return type:

```forge
Int Add(Int A, Int B)
{
    Return A + B;
}
```

A returned value can be used like this:

```forge
Int Result = Add(10, 20);
```

The returned value can be assigned to a compatible variable or used directly in another expression.

Returning functions can also return collection data. The declared shape and element types must match the returned value:

```forge
Ore(Int Number, Weld Name) MakePerson()
{
    Return {14, "Den"};
}
```

## The `Return` Keyword

`Return` ends the current function call and sends its expression back to the caller:

```forge
Weld Greeting()
{
    Return "hello";
}
```

`Return` is case-sensitive and must be written with a capital `R`.

A `Nunction` cannot return data because it has no return type. This is an error:

```forge
Nunction Bad()
{
    Return 42;
}
```

Furnace reports `Void function Bad cannot return a value`.

The returned expression must also be compatible with the function's declared return type. Returning a mismatched data type is an error:

```forge
Int Bad()
{
    Return "wrong";
}
```

Furnace reports `Return type mismatch in Bad: expected Int, got Weld`.

## Function Parameters

The grammar accepts parameter declarations:

```forge
Nunction PrintNumber(Int Value)
{
    Print(\V"{Value}");
}
```

A call can be written as:

```forge
PrintNumber(42);
```

Furnace checks argument count and argument types during semantic analysis. Type aliases such as `String` and `Weld`, and `Boolean` and `Bool`, are treated as equivalent.

## Native Function Calls

For example:

```forge
Nunction Tick()
{
    Print("tick");
}

Open Nunction Main()
{
    Tick();
}
```

The compiler compiles `Tick` as an independent Cranelift function and emits a call from `Main`.

The same calling convention supports parameterized functions, return values, calls inside `If`, `While`, and `For`, and recursive or mutually recursive functions.

See [Function Calls](function-calls.md) for details on how calls are written, and [Recursion](recursion.md) for the current state of recursive functions.

---

[← Previous](../getting-started/compilation.md)
[Next →](main.md)
