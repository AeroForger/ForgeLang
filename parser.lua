local function expect(tokens, pos, expectedType, errMsg)
    local t = tokens[pos]
    if not t or t.type ~= expectedType then
        error(errMsg or ("Expected " .. expectedType .. " at token " .. tostring(pos)))
    end
    return t
end

local function parsePrimary(tokens, pos)
    local token = tokens[pos]
    if not token then
        error("Unexpected end of input")
    end

    if token.type == "NUMBER" then
        return {
            type = "number_literal",
            value = tonumber(token.value)
        }, pos + 1
    end

    if token.type == "STRING" then
        return {
            type = "string_literal",
            value = token.value
        }, pos + 1
    end

    if token.type == "IDENTIFIER" then
        return {
            type = "identifier",
            name = token.value
        }, pos + 1
    end

    error("Expected value at token " .. tostring(pos) .. ", got " .. tostring(token.type))
end

local function parseExpression(tokens, pos)
    local expr, nextPos = parsePrimary(tokens, pos)

    while nextPos <= #tokens and tokens[nextPos].type == "PLUS" do
        local right, afterRight = parsePrimary(tokens, nextPos + 1)
        expr = {
            type = "addition",
            left = expr,
            right = right
        }
        nextPos = afterRight
    end

    return expr, nextPos
end

function Parse(tokens)
    local a = 1
    local AST = {}
    local Variables = {}

    while a <= #tokens do
        local token = tokens[a]

        if token.type == "NUMBER_TYPE" or token.type == "STRING_TYPE" then
            local varType = token.type == "NUMBER_TYPE" and "number" or "string"
            local name = expect(tokens, a + 1, "IDENTIFIER", "Expected identifier after type")
            expect(tokens, a + 2, "EQUALS", "Missing equals after identifier")

            local valueExpr, valueEnd = parseExpression(tokens, a + 3)
            expect(tokens, valueEnd, "SEMICOLON", "Missing semicolon after assignment")

            table.insert(AST, {
                type = "var_decl",
                varType = varType,
                name = name.value,
                value = valueExpr
            })
            Variables[name.value] = nil
            a = valueEnd + 1

        elseif token.type == "PRINT" then
            expect(tokens, a + 1, "LPAREN", "Expected '(' after Print")

            local valueExpr, valueEnd = parseExpression(tokens, a + 2)
            expect(tokens, valueEnd, "RPAREN", "Expected closing )")
            expect(tokens, valueEnd + 1, "SEMICOLON", "Expected a semicolon")

            table.insert(AST, {
                type = "print",
                value = valueExpr
            })
            a = valueEnd + 2
        else
            error("Unexpected token: " .. tostring(token.type))
        end
    end

    return { ast = AST, variables = Variables }
end
return {
    Parse = Parse
}