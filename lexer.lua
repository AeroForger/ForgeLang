local KeyWords = {
    ["Number"] = "NUMBER_TYPE" 
}
local Symbols = {
    ["="] = "EQUALS",
    [";"] = "SEMICOLON"
}
function Tokenize(source)
    local tokens = {}

    local i = 1
    while i <= #source do
        local ch = source:sub(i, i)

        if ch == " " or ch == "\n" or ch == "\t" then
            i = i + 1
        elseif ch == "=" then
            table.insert(tokens, "EQUALS")
            i = i + 1
        elseif ch == ";" then
            table.insert(tokens, "SEMICOLON")
            i = i + 1
        else
            local word = ""
            while i <= #source and source:sub(i, i) ~= " " and source:sub(i, i) ~= "\n" and source:sub(i, i) ~= "\t" and source:sub(i, i) ~= "=" and source:sub(i, i) ~= ";" do
                word = word .. source:sub(i, i)
                i = i + 1
            end

            if word == "" then
                i = i + 1
            elseif KeyWords[word] then
                table.insert(tokens, KeyWords[word])
            elseif string.match(word, "^%d+$") then
                table.insert(tokens, 'NUMBER("' .. word .. '")')
            else
                table.insert(tokens, 'IDENTIFIER("' .. word .. '")')
            end
        end
    end
    return tokens
end

return {
    tokenize = Tokenize
}
