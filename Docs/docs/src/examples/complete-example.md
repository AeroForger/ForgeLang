# Complete Example

The following program demonstrates several features available in Alpha 3.4:

- `Nunction`
- Variables
- Arrays
- Tuples
- Lists
- `While`
- `For`
- `If`
- `Else`
- Arithmetic
- String interpolation
- `Stop`
- Bitwise operations
- Zero-argument function calls
- Boolean values

```forge
Nunction Tick()
{
    Print("tick");
}

Open Nunction Main()
{
    // Array
    Ore[3] FixedNums = [10, 20, 30,];

    Print(FixedNums[0]);

    FixedNums[1] = 55;

    Print(FixedNums[1]);
    Print(FixedNums.Length);

    // Tuple
    Ore(Int Age, Weld Name) Person = {14, "Den"};

    Print(Person.Age);
    Print(Person.Name);

    Person.Age = 15;

    Print(Person.Age);

    // List
    Materials Int Numbers = (10, 20, 30,);

    Print(Numbers[0]);
    Print(Numbers.Length);

    Numbers.Add(40);

    Print(Numbers[3]);

    Numbers.Remove(0);

    Print(Numbers[0]);

    // While loop
    Number Int I = 0;

    While (I < 5)
    {
        Print(\V"{I}");
        I = I + 1;
    }

    // For loop
    For (Number Int J = 0; J < 3; J++)
    {
        Print(\V"J = {J}");
    }

    // Boolean
    Bool Flag = true;
    Print(Flag);
    Flag = false;
    Print(Flag);

    // Stop inside a loop
    Number Int K = 0;

    While (K < 10)
    {
        If (K == 3)
        {
            Stop;
        }

        Print(\V"{K}");
        K = K + 1;
    }

    // Zero-argument Nunction call
    Tick();

    // Bitwise operations
    Number Int A = 12;
    Number Int B = 10;

    Print(A And B);
    Print(A Or B);
    Print(A Xor B);
}
```

This example is intentionally representative of the current Alpha 3.4 subset. Some advanced features are still planned or parser-only, so it is best thought of as a snapshot of what the compiler can already handle reliably in the present implementation.

---

[← Previous](../reference/keywords.md)
[Next →](../limitations.md)
