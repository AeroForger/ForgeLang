from furnace.generated.ForgeLangAlpha1Visitor import ForgeLangAlpha1Visitor
from furnace.ast_nodes import (
    VariableDecl, DataDecl, ObjectDecl, FunctionDecl,
    AssignmentNode, PrintNode,
    NumberLiteral, StringLiteral, IdentifierExpr, MemberAccess,
    BinaryOp, UnaryOp, CallExpr, ReturnNode, IfNode, InputNode
)


class ASTBuilder(ForgeLangAlpha1Visitor):

    def visitProgram(self, ctx):
        nodes = []
        for item in ctx.topLevelItem():
            result = self.visit(item)
            if result:
                nodes.append(result)
        return nodes

    def visitTopLevelItem(self, ctx):
        # Imports parse but have no codegen effect yet (return None).
        if ctx.dataDeclaration(): return self.visit(ctx.dataDeclaration())
        if ctx.functionDeclaration(): return self.visit(ctx.functionDeclaration())
        if ctx.statement(): return self.visit(ctx.statement())
        return None

    # --- Data & Objects ---

    def visitDataDeclaration(self, ctx):
        name = ctx.ID().getText()
        members = []
        for member in ctx.dataMember():
            res = self.visit(member)
            if res: members.append(res)
        return DataDecl(name, members)

    def visitDataMember(self, ctx):
        if ctx.variableDeclaration(): return self.visit(ctx.variableDeclaration())
        if ctx.functionDeclaration(): return self.visit(ctx.functionDeclaration())
        return None

    def visitObjectDeclaration(self, ctx):
        type_name = ctx.ID(0).getText()
        name = ctx.ID(1).getText()
        body = self.visit(ctx.block())
        return ObjectDecl(type_name, name, body)

    # --- Functions ---

    def visitFunctionDeclaration(self, ctx):
        return_type = ctx.returnType().getText()
        name = ctx.ID().getText()

        params = []
        if ctx.parameterList():
            for p in ctx.parameterList().parameter():
                params.append((p.typeDeclaration().getText(), p.ID().getText()))

        body = self.visit(ctx.block())
        return FunctionDecl(name, return_type, params, body)

    def visitReturnStatement(self, ctx):
        value = self.visit(ctx.expression()) if ctx.expression() else None
        return ReturnNode(value)

    def visitBlock(self, ctx):
        statements = []
        for stmt in ctx.statement():
            res = self.visit(stmt)
            if res: statements.append(res)
        return statements

    # --- Statements ---

    def visitStatement(self, ctx):
        if ctx.variableDeclaration(): return self.visit(ctx.variableDeclaration())
        if ctx.objectDeclaration(): return self.visit(ctx.objectDeclaration())
        if ctx.assignment(): return self.visit(ctx.assignment())
        if ctx.printStatement(): return self.visit(ctx.printStatement())
        if ctx.returnStatement(): return self.visit(ctx.returnStatement())
        if ctx.expressionStatement(): return self.visit(ctx.expressionStatement().expression())
        if ctx.ifStatement(): return self.visit(ctx.ifStatement())
        if ctx.switchStatement() or ctx.doFailFinal():
            raise RuntimeError(
                "Switch/Do-Fail-Final parse correctly but are not "
                "implemented yet (planned for Alpha 0.3)")
        return None

    def visitVariableDeclaration(self, ctx):
        if ctx.standardVarDecl(): return self.visit(ctx.standardVarDecl())
        if ctx.materialsConstructorDecl(): return self.visit(ctx.materialsConstructorDecl())
        return None

    def visitStandardVarDecl(self, ctx):
        var_type = ctx.typeDeclaration().getText()
        name = ctx.ID().getText()
        if ctx.initialization():
            value = self.visit(ctx.initialization().expression())
        else:
            value = None
        return VariableDecl(var_type, name, value)

    def visitAssignment(self, ctx):
        target = ctx.qualifiedName().getText()
        value = self.visit(ctx.expression())
        return AssignmentNode(target, value)

    def visitPrintStatement(self, ctx):
        value = self.visit(ctx.expression())
        return PrintNode(value)

    # --- Expressions (unified precedence rule labels) ---

    def visitLiteralExpr(self, ctx):
        return self.visit(ctx.literal())

    def visitIntLiteral(self, ctx):
        return NumberLiteral(int(ctx.getText()))

    def visitFloatLiteral(self, ctx):
        return NumberLiteral(float(ctx.getText()))

    def visitStringLiteral(self, ctx):
        return StringLiteral(ctx.getText())

    def visitInterpStringLiteral(self, ctx):
        return StringLiteral(ctx.getText())

    def visitFunctionCallExpr(self, ctx):
        args = []
        if ctx.argumentList():
            args = [self.visit(e) for e in ctx.argumentList().expression()]
        return CallExpr(ctx.qualifiedName().getText(), args)

    def visitInputCallExpr(self, ctx):
        if ctx.inputType():
            t = ctx.inputType().getText()   # 'Int' | 'Float' | 'Generic'
            if t == 'Generic':
                t = 'Weld'                  # generic input is a string
            return InputNode(t)
        return InputNode('Weld')            # blank = generic = Weld

    def visitMemberAccessExpr(self, ctx):
        names = [t.getText() for t in ctx.qualifiedName().ID()]
        if len(names) == 1:
            return IdentifierExpr(names[0])
        if len(names) == 2:
            return MemberAccess(names[0], names[1])
        # Deeper chains (A.B.C): codegen supports one level for now,
        # flatten the prefix so the AST still reflects the source.
        return MemberAccess('.'.join(names[:-1]), names[-1])

    def visitGroupingExpr(self, ctx):
        return self.visit(ctx.expression())

    def visitUnaryExpr(self, ctx):
        op = ctx.SUB().getText() if ctx.SUB() else ctx.ADD().getText()
        return UnaryOp(op, self.visit(ctx.expression()))

    def _binary(self, ctx, op):
        return BinaryOp(op, self.visit(ctx.expression(0)), self.visit(ctx.expression(1)))

    def visitPowExpr(self, ctx):
        return self._binary(ctx, '**')

    def visitMulExpr(self, ctx):
        return self._binary(ctx, ctx.MUL().getText() if ctx.MUL() else ctx.DIV().getText())

    def visitAddExpr(self, ctx):
        return self._binary(ctx, ctx.ADD().getText() if ctx.ADD() else ctx.SUB().getText())

    def visitCmpExpr(self, ctx):
        for name in ('EQ', 'NEQ', 'LT', 'GT', 'LTE', 'GTE'):
            tok = getattr(ctx, name)()
            if tok is not None:
                return self._binary(ctx, tok.getText())
        return self._binary(ctx, '?')

    def visitAndExpr(self, ctx):
        return self._binary(ctx, 'And')

    def visitOrExpr(self, ctx):
        return self._binary(ctx, 'Or')

    def visitXorExpr(self, ctx):
        return self._binary(ctx, 'Xor')
    # --- Control flow ---

    def visitIfStatement(self, ctx):
        exprs = ctx.expression()
        blocks = ctx.block()

        branches = []
        for i in range(len(exprs)):
            branches.append((self.visit(exprs[i]), self.visit(blocks[i])))

        # blocks = 1 per If/Else If, +1 only when a final Else exists
        else_body = None
        if len(blocks) > len(exprs):
            else_body = self.visit(blocks[-1])

        return IfNode(branches, else_body)