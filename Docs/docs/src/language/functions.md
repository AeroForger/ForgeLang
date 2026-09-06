# Functions

ForgeLang currently has two function forms:

* `Nunction`
* `function`

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

## `function`

`function` declares a function that returns a value.

Example syntax:

```forge
function Add(Int A, Int B)
{
    return A + B;
}
```

A returned value can be used like this:

```forge
Int Result = Add(10, 20);
```

The returned value can be used in an expression. The return expression must match the declared return type.

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
