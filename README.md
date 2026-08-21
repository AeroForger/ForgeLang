# ForgeLang-Pre-Alpha

Current version - 0.0.0.2

Currently written in Lua

## Progress

- [X] Lexer
- [X] File loading
- [ ] Parser
- [ ] AST
- [ ] Interpreter

## plans

Refactor Lexer so making the parser will be easier

Original lexer:
```lua
  tokens = {
  NUMBER_TYPE
  IDENTIFIER("x")
  EQUALS
  NUMBER("2")
  SEMICOLON
  }
```
Refactored lexers idea:
```lua
tokens = {
    { type = "NUMBER_TYPE" },
    { type = "IDENTIFIER", value = "x" },
    { type = "EQUALS" },
    { type = "NUMBER", value = 2 },
    { type = "SEMICOLON" }
}
```
