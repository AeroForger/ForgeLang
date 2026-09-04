# Functions

ForgeLang currently has two function forms:

* `Nunction`
* `function`

The current backend can inline zero-argument `Nunction` calls.

Parameterized functions and return-value functions are accepted by parts of the frontend but are not yet generated as native function calls.

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

Zero-argument `Nunction` calls can currently be expanded directly into the caller during code generation.

## `function`

`function` is reserved for functions that return a value.

Example syntax:

```forge
function Add(Number Int A, Number Int B)
{
    return A + B;
}
```

A returned value can be used like this:

```forge
Number Int Result = Add(10, 20);
```

Return-value code generation is not currently implemented.

The syntax exists because eventually the compiler will have to deal with functions that actually return things. Otherwise `function` would be a rather optimistic keyword.

## Function Parameters

The grammar accepts parameter declarations:

```forge
Nunction PrintNumber(Number Int Value)
{
    Print(\V"{Value}");
}
```

A call can be written as:

```forge
PrintNumber(42);
```

Furnace checks the number of supplied arguments during semantic analysis.

Parameterized calls are currently rejected before native code generation.

The parser is currently ahead of the backend in this area. The parser has seen the future. The backend has not.

## Zero-Argument Call Inlining

The current backend does not yet use a native calling convention for user-defined functions.

Instead, zero-argument `Nunction` calls are expanded before code generation.

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

The compiler can replace the `Tick()` call with the statements inside `Tick`.

The `expand_function_calls` pass in `src/codegen.rs` handles this transformation.

The pass can also process calls inside:

* `If`
* `While`
* `For`

Recursive inlining is not supported.

The callee must be a zero-argument `Nunction`.

See [Function Calls](function-calls.md) for details on how calls are written, and [Recursion](recursion.md) for the current state of recursive functions.

---

[← Previous](../getting-started/compilation.md)
[Next →](main.md)
