# Program Structure

A ForgeLang program consists of declarations and statements.

A simple program looks like:

```forge
Open Nunction Main()
{
    Int I = 0;

    While (I < 10)
    {
        Print(\V"{I}");
        I = I + 1;
    }
}
```

ForgeLang uses braces `{}` to delimit function and control-flow bodies.

Statements are terminated with `;`.

## Top-Level Declarations

Top-level declarations can include:

* Function definitions
* Variable declarations
* Data declarations
* Import statements

The exact behavior of some declarations depends on the current compiler
implementation. `Use` and `Using` imports are resolved before semantic analysis
and work with either backend. `Data` and object code generation still has
limitations; see [Current Limitations](../limitations.md) for details.

## Statement Terminators

Every statement ends with a semicolon. This includes variable declarations, assignments, function calls, control-flow statements, and the `Stop` statement. The semicolon tells the parser where one statement ends and the next begins.

## Braces

Braces group statements into a single block. They are used for:

* Function bodies
* `If` / `Else If` / `Else` branches
* `While` loop bodies
* `For` loop bodies
* `Data` declaration bodies
* Object initializer bodies

---

[← Previous](hello-world.md)
[Next →](compilation.md)
