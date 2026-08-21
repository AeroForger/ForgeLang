local lex = require("lexer")

local file, err = io.open("ForgeLang.anvil", "r")

if not file then
    print("Could not open file: " .. err)
    return
end

local content = file:read("*a")
file:close()

local tokens = lex.tokenize(content)

for i, token in ipairs(tokens) do
    print(i, token)
end