# Generated from ForgeLangAlpha1.g4 by ANTLR 4.13.1
from antlr4 import *
if "." in __name__:
    from .ForgeLangAlpha1Parser import ForgeLangAlpha1Parser
else:
    from ForgeLangAlpha1Parser import ForgeLangAlpha1Parser

# This class defines a complete generic visitor for a parse tree produced by ForgeLangAlpha1Parser.

class ForgeLangAlpha1Visitor(ParseTreeVisitor):

    # Visit a parse tree produced by ForgeLangAlpha1Parser#program.
    def visitProgram(self, ctx:ForgeLangAlpha1Parser.ProgramContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#topLevelItem.
    def visitTopLevelItem(self, ctx:ForgeLangAlpha1Parser.TopLevelItemContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#useImport.
    def visitUseImport(self, ctx:ForgeLangAlpha1Parser.UseImportContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#usingImport.
    def visitUsingImport(self, ctx:ForgeLangAlpha1Parser.UsingImportContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#qualifiedName.
    def visitQualifiedName(self, ctx:ForgeLangAlpha1Parser.QualifiedNameContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#modifier.
    def visitModifier(self, ctx:ForgeLangAlpha1Parser.ModifierContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#typeDeclaration.
    def visitTypeDeclaration(self, ctx:ForgeLangAlpha1Parser.TypeDeclarationContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#numberType.
    def visitNumberType(self, ctx:ForgeLangAlpha1Parser.NumberTypeContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#numberKind.
    def visitNumberKind(self, ctx:ForgeLangAlpha1Parser.NumberKindContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#oreType.
    def visitOreType(self, ctx:ForgeLangAlpha1Parser.OreTypeContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#materialsType.
    def visitMaterialsType(self, ctx:ForgeLangAlpha1Parser.MaterialsTypeContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#variableDeclaration.
    def visitVariableDeclaration(self, ctx:ForgeLangAlpha1Parser.VariableDeclarationContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#standardVarDecl.
    def visitStandardVarDecl(self, ctx:ForgeLangAlpha1Parser.StandardVarDeclContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#materialsConstructorDecl.
    def visitMaterialsConstructorDecl(self, ctx:ForgeLangAlpha1Parser.MaterialsConstructorDeclContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#initialization.
    def visitInitialization(self, ctx:ForgeLangAlpha1Parser.InitializationContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#dataDeclaration.
    def visitDataDeclaration(self, ctx:ForgeLangAlpha1Parser.DataDeclarationContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#dataMember.
    def visitDataMember(self, ctx:ForgeLangAlpha1Parser.DataMemberContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#objectDeclaration.
    def visitObjectDeclaration(self, ctx:ForgeLangAlpha1Parser.ObjectDeclarationContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#functionDeclaration.
    def visitFunctionDeclaration(self, ctx:ForgeLangAlpha1Parser.FunctionDeclarationContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#typeReturnType.
    def visitTypeReturnType(self, ctx:ForgeLangAlpha1Parser.TypeReturnTypeContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#dynamicReturnType.
    def visitDynamicReturnType(self, ctx:ForgeLangAlpha1Parser.DynamicReturnTypeContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#noReturnType.
    def visitNoReturnType(self, ctx:ForgeLangAlpha1Parser.NoReturnTypeContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#parameterList.
    def visitParameterList(self, ctx:ForgeLangAlpha1Parser.ParameterListContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#parameter.
    def visitParameter(self, ctx:ForgeLangAlpha1Parser.ParameterContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#statement.
    def visitStatement(self, ctx:ForgeLangAlpha1Parser.StatementContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#assignment.
    def visitAssignment(self, ctx:ForgeLangAlpha1Parser.AssignmentContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#returnStatement.
    def visitReturnStatement(self, ctx:ForgeLangAlpha1Parser.ReturnStatementContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#printStatement.
    def visitPrintStatement(self, ctx:ForgeLangAlpha1Parser.PrintStatementContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#expressionStatement.
    def visitExpressionStatement(self, ctx:ForgeLangAlpha1Parser.ExpressionStatementContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#ifStatement.
    def visitIfStatement(self, ctx:ForgeLangAlpha1Parser.IfStatementContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#switchStatement.
    def visitSwitchStatement(self, ctx:ForgeLangAlpha1Parser.SwitchStatementContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#doFailFinal.
    def visitDoFailFinal(self, ctx:ForgeLangAlpha1Parser.DoFailFinalContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#block.
    def visitBlock(self, ctx:ForgeLangAlpha1Parser.BlockContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#inputCallExpr.
    def visitInputCallExpr(self, ctx:ForgeLangAlpha1Parser.InputCallExprContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#orExpr.
    def visitOrExpr(self, ctx:ForgeLangAlpha1Parser.OrExprContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#xorExpr.
    def visitXorExpr(self, ctx:ForgeLangAlpha1Parser.XorExprContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#cmpExpr.
    def visitCmpExpr(self, ctx:ForgeLangAlpha1Parser.CmpExprContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#unaryExpr.
    def visitUnaryExpr(self, ctx:ForgeLangAlpha1Parser.UnaryExprContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#addExpr.
    def visitAddExpr(self, ctx:ForgeLangAlpha1Parser.AddExprContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#literalExpr.
    def visitLiteralExpr(self, ctx:ForgeLangAlpha1Parser.LiteralExprContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#functionCallExpr.
    def visitFunctionCallExpr(self, ctx:ForgeLangAlpha1Parser.FunctionCallExprContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#mulExpr.
    def visitMulExpr(self, ctx:ForgeLangAlpha1Parser.MulExprContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#memberAccessExpr.
    def visitMemberAccessExpr(self, ctx:ForgeLangAlpha1Parser.MemberAccessExprContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#groupingExpr.
    def visitGroupingExpr(self, ctx:ForgeLangAlpha1Parser.GroupingExprContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#powExpr.
    def visitPowExpr(self, ctx:ForgeLangAlpha1Parser.PowExprContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#andExpr.
    def visitAndExpr(self, ctx:ForgeLangAlpha1Parser.AndExprContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#argumentList.
    def visitArgumentList(self, ctx:ForgeLangAlpha1Parser.ArgumentListContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#inputType.
    def visitInputType(self, ctx:ForgeLangAlpha1Parser.InputTypeContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#intLiteral.
    def visitIntLiteral(self, ctx:ForgeLangAlpha1Parser.IntLiteralContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#floatLiteral.
    def visitFloatLiteral(self, ctx:ForgeLangAlpha1Parser.FloatLiteralContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#stringLiteral.
    def visitStringLiteral(self, ctx:ForgeLangAlpha1Parser.StringLiteralContext):
        return self.visitChildren(ctx)


    # Visit a parse tree produced by ForgeLangAlpha1Parser#interpStringLiteral.
    def visitInterpStringLiteral(self, ctx:ForgeLangAlpha1Parser.InterpStringLiteralContext):
        return self.visitChildren(ctx)



del ForgeLangAlpha1Parser