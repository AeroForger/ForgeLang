local scriptSource = debug.getinfo(1, "S").source
local scriptDir = scriptSource and scriptSource:sub(2):match("(.*/)") or ""
package.path = scriptDir .. "?.lua;" .. package.path

local lex = require("lexer")
local inter = require("interpreter")
local pars = require("parser")

local filename = arg[1]

if not filename then
    print("Usage: Furnace <file.anvil>")
    return
end

local file, err = io.open(filename, "r")

if not file then
    print("Could not open file: " .. err)
    return
end

local content = file:read("*a")
file:close()

local tokens = lex.tokenize(content)

for i,line in ipairs(tokens) do
    print(i, line.type, line.value)
end

local result = pars.Parse(tokens)

local AST = result.ast
local Variables = result.variables

inter.FunctionsFL(AST, Variables)