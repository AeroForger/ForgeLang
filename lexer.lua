local KeyWords = {
    ["Number"] = "NUMBER_TYPE",
    ["Print"] = "PRINT"
}
local Symbols = {
    ["="] = "EQUALS",
    [";"] = "SEMICOLON",
    ["("] = "LPAREN",
    [")"] = "RPAREN",
    ["+"] = "PLUS"
}
function Tokenize(source)
    local tokens = {}

    local i = 1
    while i <= #source do
        local ch = source:sub(i, i)

        if ch == " " or ch == "\n" or ch == "\t" then
            i = i + 1
        elseif Symbols[ch] then
            local inserter = {type = Symbols[ch], value = ch}
            table.insert(tokens, inserter)
            i = i + 1
        else
            local word = ""
            while i <= #source do
                local current = source:sub(i, i)
                if current == " "
                    or current == "\n"
                    or current == "\t"
                    or Symbols[current] then
                    break                       
                end
                word = word .. source:sub(i, i)
                i = i + 1
            end

            if word == "" then
                i = i + 1
            elseif KeyWords[word] then
                local inserter = {type = KeyWords[word], value = word}
                table.insert(tokens, inserter)
            elseif string.match(word, "^%d+$") then
                local inserter = {type = "NUMBER", value = word}
                table.insert(tokens, inserter)
            else
                local inserter = {type = "IDENTIFIER", value = word}
                table.insert(tokens, inserter)
            end
        end
    end
    return tokens
end

return {
    tokenize = Tokenize
}
