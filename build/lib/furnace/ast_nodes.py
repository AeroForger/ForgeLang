class ASTNode:
    pass


class VariableDecl(ASTNode):
    def __init__(self, var_type, name, value):
        self.var_type = var_type
        self.name = name
        self.value = value

    def __repr__(self):
        return f"VarDecl(type='{self.var_type}', name='{self.name}', value={self.value})"


class DataDecl(ASTNode):
    def __init__(self, name, members):
        self.name = name
        self.members = members

    def __repr__(self):
        return f"DataDecl(name='{self.name}', members={self.members})"


class ObjectDecl(ASTNode):
    def __init__(self, type_name, name, body):
        self.type_name = type_name
        self.name = name
        self.body = body

    def __repr__(self):
        return f"ObjectDecl(type='{self.type_name}', name='{self.name}', body={self.body})"



class AssignmentNode(ASTNode):
    def __init__(self, target, value):
        self.target = target
        self.value = value

    def __repr__(self):
        return f"Assignment(target='{self.target}', value={self.value})"


class PrintNode(ASTNode):
    def __init__(self, value):
        self.value = value

    def __repr__(self):
        return f"PrintNode(value={self.value})"


# --- Expression nodes ---

class NumberLiteral(ASTNode):
    def __init__(self, value):
        self.value = value  # int or float

    def __repr__(self):
        return f"{self.value}"


class StringLiteral(ASTNode):
    def __init__(self, raw):
        self.raw = raw  # keeps quotes: "..." or \V"..."

    def __repr__(self):
        return self.raw


class IdentifierExpr(ASTNode):
    def __init__(self, name):
        self.name = name

    def __repr__(self):
        return self.name


class MemberAccess(ASTNode):
    def __init__(self, obj, member):
        self.obj = obj
        self.member = member

    def __repr__(self):
        return f"{self.obj}.{self.member}"


class BinaryOp(ASTNode):
    def __init__(self, op, left, right):
        self.op = op
        self.left = left
        self.right = right

    def __repr__(self):
        return f"({self.left} {self.op} {self.right})"
class UnaryOp(ASTNode):
    def __init__(self, op, operand):
        self.op = op          # '+' or '-'
        self.operand = operand

    def __repr__(self):
        return f"({self.op}{self.operand})"


class CallExpr(ASTNode):
    def __init__(self, name, args):
        self.name = name
        self.args = args

    def __repr__(self):
        return f"{self.name}({', '.join(str(a) for a in self.args)})"
class IfNode(ASTNode):
    def __init__(self, branches, else_body):
        self.branches = branches    # list of (condition_expr, body_stmts)
        self.else_body = else_body  # list of stmts, or None

    def __repr__(self):
        return f"If({self.branches}, else={self.else_body})"


class BoolLiteral(ASTNode):
    def __init__(self, value):
        self.value = value  # True/False

    def __repr__(self):
        return str(self.value)
class FunctionDecl(ASTNode):
    # (replaces the old one - note the new fields)
    def __init__(self, name, return_type, params, body):
        self.name = name
        self.return_type = return_type  # 'Nunction' | 'function' | type text
        self.params = params            # list of (type_text, name)
        self.body = body

    def __repr__(self):
        return f"FunctionDecl(name='{self.name}', returns='{self.return_type}', params={self.params}, body={len(self.body)} stmts)"


class ReturnNode(ASTNode):
    def __init__(self, value):
        self.value = value  # expr or None

    def __repr__(self):
        return f"Return({self.value})"