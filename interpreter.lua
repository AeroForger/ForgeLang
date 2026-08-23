function Evaluate(expression, variables)
    if expression.type == "number_literal" then
        return expression.value
    end

    if expression.type == "string_literal" then
        return expression.value
    end

    if expression.type == "identifier" then
        if variables[expression.name] == nil then
            error("Variable '" .. expression.name .. "' does not exist")
        end

        return variables[expression.name]
    end

    if expression.type == "addition" then
        local left = Evaluate(expression.left, variables)
        local right = Evaluate(expression.right, variables)

        if type(left) == "number" and type(right) == "number" then
            return left + right
        end

        return tostring(left) .. tostring(right)
    end

    error("Unknown expression type: " .. tostring(expression.type))
end

function FunctionsFL(node, variables)
    for _, stmt in ipairs(node) do
        if stmt.type == "var_decl" then
            variables[stmt.name] = Evaluate(stmt.value, variables)
        elseif stmt.type == "print" then
            local result = Evaluate(stmt.value, variables)
            print(result)
        end
    end
end

return {
    FunctionsFL = FunctionsFL,
    Evaluate = Evaluate
}