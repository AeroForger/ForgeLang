local lex = require("lexer")
local inter = require("interpreter")
local pars = require("parser")


local file, err = io.open("ForgeLang.anvil", "r")

if not file then
    print("Could not open file: " .. err)
    return
end

local content = file:read("*a")
file:close()

local tokens = lex.tokenize(content)

local result = pars.Parse(tokens)

local AST = result.ast
local Variables = result.variables

inter.FunctionsFL(AST, Variables)
