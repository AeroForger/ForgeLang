function Evaluate(expression, variables)
    if expression.type == "addition" then
        local x = variables[expression.left]
        local y = variables[expression.right]

        if x == nil then
            error("Variable '" .. expression.left .. "' does not exist")
        end

        if y == nil then
            error("Variable '" .. expression.right .. "' does not exist")
        end

        return x + y
    end
end
function FunctionsFL(node, variables)
    for _, node in ipairs(node) do
        if node.type == "print" then
            local result = Evaluate(node.value, variables)
            print(result)
        end
    end
end
return {
    FunctionsFL = FunctionsFL
}
