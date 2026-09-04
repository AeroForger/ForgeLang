# Types Reference

This page collects the core ForgeLang types and points to the detailed language sections.

| Type | Description | Details |
| ---- | ----------- | ------- |
| `Number Int` | Integer value | See [Integers](../language/integers.md) |
| `Number Float` | Floating-point value | See [Floating-Point Numbers](../language/floating-point.md) |
| `Weld` | String value | See [Strings](../language/strings.md) |
| `Bool` / `Boolean` | Boolean value | See [Types](../language/types.md) |
| `Ore[...]` | Fixed-size array | See [Types](../language/types.md) |
| `Ore(...)` | Tuple with named fields | See [Types](../language/types.md) |
| `Materials T` | List type | See [Types](../language/types.md) |
| `Generic` | Generic list element type | See [Types](../language/types.md) |

## Notes

- `Bool` and `Boolean` are treated as the same type.
- Arrays use `Ore` with a fixed size.
- Lists use `Materials` and support methods such as `.Add(...)`, `.Remove(...)`, and `.RemoveAt(...)`.
- The current implementation is still limited: generic storage is constrained, and data/object declarations are not yet executable in the backend.

---

[← Previous](../compiler/linking.md)
[Next →](operators.md)
