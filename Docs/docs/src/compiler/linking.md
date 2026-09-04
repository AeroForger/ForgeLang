# Linking

Furnace generates a native object file.

The object file is linked using the system C compiler.

For example:

```fish
cc main.o -o main -lm
```

The linker produces the final executable.

This is the last stage of the current Alpha 3.4 pipeline. Once the object file is generated, the program becomes a native binary like any other C or Rust program built for the target platform.

The currently supported target is `linux`.

Additional targets can be added as compiler support is implemented.

---

[← Previous](code-generation.md)
[Next →](../reference/types.md)
