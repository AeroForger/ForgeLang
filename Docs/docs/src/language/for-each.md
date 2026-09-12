# ForEach Loops

Sydrogen supports `ForEach` loops that iterate over a collection.

Basic syntax:

```forge
ForEach (type item in collection)
{
    // body
}
```

Example:

```forge
Array[Int] nums = {1, 2, 3, 4, 5};

ForEach (Int num in nums)
{
    Print(num);
}
```

A `ForEach` loop has two parts:

1. `type item` - declares the loop variable and its type.
2. `collection` - the collection being iterated.

The loop variable is in scope only inside the loop body. It is not in scope while the collection expression is being evaluated.

The collection must be an `Array` or `List`. A type mismatch produces a semantic error.

`Skip` continues with the next iteration of the loop. `Stop` exits the current loop. `Skip` can only be used inside a loop.

A `ForEach` loop without braces produces an error.

---

[← Previous](for.md)
[Next →](stop.md)