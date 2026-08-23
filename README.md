# ForgeLang-Pre-Alpha

Current version - 0.0.0.2

Currently written in Lua

## Progress

it can do

```ForgeLang
Number X = 1;
Number Y = 2;
Print(X + Y);
```
out put will be 3

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
