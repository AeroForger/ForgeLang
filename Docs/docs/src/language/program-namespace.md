# The Program Namespace

The `Program` namespace provides runtime operations.

The currently implemented member is:

```forge
Program.Stop();
```

This terminates the program by calling the C `exit` function with a status of `0`.

`Program.Stop()` is treated as a namespace operation rather than a normal user-defined function call.

The semantic analyzer checks that:

- It is used inside a function
- It has zero arguments
- The namespace and operation are valid

Example:

```forge
Open Nunction Main()
{
    Print("before");
    Program.Stop();
    Print("after");
}
```

The second `Print` is never reached.

---

[← Previous](stop.md)
[Next →](function-calls.md)
