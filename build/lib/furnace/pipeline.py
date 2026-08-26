from antlr4 import FileStream, CommonTokenStream
from furnace.generated.ForgeLangAlpha1Lexer import ForgeLangAlpha1Lexer
from furnace.generated.ForgeLangAlpha1Parser import ForgeLangAlpha1Parser
from furnace.ast_builder import ASTBuilder
from furnace.semantic import check
from furnace.codegen.llvm_gen import LLVMCodeGenerator

def compile_file(path):
    stream = FileStream(path)
    lexer = ForgeLangAlpha1Lexer(stream)
    tokens = CommonTokenStream(lexer)
    parser = ForgeLangAlpha1Parser(tokens)
    tree = parser.program()

    ast = ASTBuilder().visit(tree)
    check(ast)
    return LLVMCodeGenerator().generate(ast)