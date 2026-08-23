local function expect(tokens, pos, expectedType, errMsg)
    local t = tokens[pos]
    if not t or t.type ~= expectedType then
        error(errMsg or ("Expected " .. expectedType .. " at token " .. tostring(pos)))
    end
    return t
end

function Parse(tokens)
    local a = 1
    local AST = {}
    local Variables = {}

    while a <= #tokens do
        local token = tokens[a]

        if token.type == "NUMBER_TYPE" then
            local name = expect(tokens, a + 1, "IDENTIFIER", "Expected identifier after type")
            expect(tokens, a + 2, "EQUALS", "Missing equals after identifier")
            local value = expect(tokens, a + 3, "NUMBER", "Expected number after equals")
            expect(tokens, a + 4, "SEMICOLON", "Missing semicolon after number")

            table.insert(AST, {
                type = "var_decl",
                varType = "number",
                name = name.value,
                value = tonumber(value.value)
            })
            Variables[name.value] = tonumber(value.value)
            a = a + 5

        elseif token.type == "PRINT" then
            expect(tokens, a + 1, "LPAREN", "Expected '(' after Print")
            local value1 = expect(tokens, a + 2, "IDENTIFIER", "Expected identifier after (")
            expect(tokens, a + 3, "PLUS", "Expected a plus sign after first identifier")
            local value2 = expect(tokens, a + 4, "IDENTIFIER", "Expected an identifier after +" )
            expect(tokens, a + 5, "RPAREN", "Expected closing )")
            expect(tokens, a + 6, "SEMICOLON", "Expected a semicolon")
            table.insert(AST, {
                type = "print",
                value = {
                    type = "addition",
                    left = value1.value,
                    right = value2.value
                }
            })
            a = a + 7
        else
            error("Unexpected token: " .. tostring(token.type))
        end
    end

    return { ast = AST, variables = Variables }
end
return {
    Parse = Parse
}
