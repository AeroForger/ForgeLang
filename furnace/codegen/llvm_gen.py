from llvmlite import ir
from furnace.ast_nodes import (
    FunctionDecl, VariableDecl, PrintNode, ObjectDecl, DataDecl,
    AssignmentNode, NumberLiteral, StringLiteral, IdentifierExpr,
    MemberAccess, BinaryOp, UnaryOp, CallExpr, ReturnNode, IfNode
)

I32 = ir.IntType(32)
I8P = ir.IntType(8).as_pointer()
F64 = ir.DoubleType()

ESCAPES = {
    'n': '\n',
    't': '\t',
    'r': '\r',
    '0': '\0',
    '\\': '\\',
    '"': '"',
}


def process_escapes(text):
    """Converts source-level escapes (\n, \t, ...) into real characters."""
    out = []
    i = 0
    while i < len(text):
        ch = text[i]
        if ch == '\\' and i + 1 < len(text) and text[i + 1] in ESCAPES:
            out.append(ESCAPES[text[i + 1]])
            i += 2
        else:
            out.append(ch)
            i += 1
    return ''.join(out)


def printf_escape(text):
    """Doubles % so printf prints it literally instead of reading a format spec."""
    return text.replace('%', '%%')


class LLVMCodeGenerator:
    def __init__(self):
        self.module = ir.Module(name="ForgeLangModule")
        self.builder = None
        self.scopes = [{}]

        self.structs = {}
        self.struct_fields = {}
        self.obj_types = {}
        self.global_inits = []

        # Caches: identical strings share one global, and printing the
        # same format twice no longer crashes with DuplicatedNameError
        self.string_globals = {}
        self.fmt_globals = {}

        printf_ty = ir.FunctionType(I32, [I8P], var_arg=True)
        self.printf = ir.Function(self.module, printf_ty, name="printf")

        pow_ty = ir.FunctionType(F64, [F64, F64])
        self.pow_fn = ir.Function(self.module, pow_ty, name="pow")

    # ---------- helpers ----------

    def lookup_var(self, name):
        for scope in reversed(self.scopes):
            if name in scope:
                return scope[name]
        return None

    def make_global_string(self, raw):
        if raw.startswith('\\V"'):
            content = raw[3:-1]
        elif raw.startswith('"'):
            content = raw[1:-1]
        else:
            content = raw
        content = process_escapes(content) + "\0"
        if content in self.string_globals:
            return self.builder.bitcast(self.string_globals[content], I8P)
        c = ir.Constant(ir.ArrayType(ir.IntType(8), len(content)), bytearray(content, "utf-8"))
        g = ir.GlobalVariable(self.module, c.type, name=f"str_{len(self.string_globals)}")
        g.linkage = 'internal'
        g.global_constant = True
        g.initializer = c
        self.string_globals[content] = g
        return self.builder.bitcast(g, I8P)

    def emit_printf(self, fmt, args):
        full = fmt + "\n\0"  # Print always ends the line
        if full not in self.fmt_globals:
            c = ir.Constant(ir.ArrayType(ir.IntType(8), len(full)), bytearray(full, "utf-8"))
            g = ir.GlobalVariable(self.module, c.type, name=f"fmt_{len(self.fmt_globals)}")
            g.linkage = 'internal'
            g.global_constant = True
            g.initializer = c
            self.fmt_globals[full] = g
        ptr = self.builder.bitcast(self.fmt_globals[full], I8P)
        self.builder.call(self.printf, [ptr] + args)

    def llvm_type_of(self, var_type_text):
        if "Weld" in var_type_text:
            return I8P
        if "Float" in var_type_text:
            return F64
        return I32

    def coerce(self, value, from_ty, to_ty):
        if from_ty == to_ty:
            return value
        if from_ty == I32 and to_ty == F64:
            return self.builder.sitofp(value, to_ty)
        if from_ty == F64 and to_ty == I32:
            return self.builder.fptosi(value, to_ty)
        if from_ty == ir.IntType(1) and to_ty == I32:
            return self.builder.zext(value, to_ty)
        return value

    # ---------- top level ----------

    def generate(self, ast_nodes):
        for node in ast_nodes:
            if isinstance(node, DataDecl):
                self.gen_data_decl(node)

        for node in ast_nodes:
            if isinstance(node, ObjectDecl):
                self.gen_object_decl(node)

        main_func = None
        for node in ast_nodes:
            if isinstance(node, FunctionDecl):
                func = self.gen_function(node)
                if node.name == "Main":
                    main_func = func

        c_main_type = ir.FunctionType(I32, [])
        c_main = ir.Function(self.module, c_main_type, name="main")
        block = c_main.append_basic_block(name="entry")
        self.builder = ir.IRBuilder(block)

        for init_node in self.global_inits:
            if isinstance(init_node, AssignmentNode):
                self.gen_assignment(init_node)

        if main_func:
            self.builder.call(main_func, [])
        self.builder.ret(ir.Constant(I32, 0))

        return str(self.module)

    def gen_data_decl(self, node):
        fields = []
        field_map = {}
        for i, member in enumerate(node.members):
            if isinstance(member, VariableDecl):
                fields.append(self.llvm_type_of(member.var_type))
                field_map[member.name] = i
        struct_ty = ir.LiteralStructType(fields)
        self.structs[node.name] = struct_ty
        self.struct_fields[node.name] = field_map

    def gen_object_decl(self, node):
        struct_ty = self.structs.get(node.type_name)
        if not struct_ty:
            return
        obj_ptr = ir.GlobalVariable(self.module, struct_ty, name=node.name)
        obj_ptr.linkage = 'internal'
        obj_ptr.initializer = ir.Constant(struct_ty, None)
        self.scopes[0][node.name] = (obj_ptr, struct_ty)
        self.obj_types[node.name] = node.type_name
        self.global_inits.extend(node.body)

    def resolve_return_type(self, node):
        """FunctionDecl -> (llvm_type, is_void)"""
        if node.return_type == 'Nunction':
            return ir.VoidType(), True
        if node.return_type == 'function':
            return I32, False          # dynamic: default to i32 for now
        return self.llvm_type_of(node.return_type), False

    def gen_function(self, node):
        ret_ty, is_void = self.resolve_return_type(node)
        param_tys = [self.llvm_type_of(t) for t, n in node.params]
        func_type = ir.FunctionType(ret_ty, param_tys)

        func = ir.Function(self.module, func_type, name=node.name)

        # Entry block
        block = func.append_basic_block(name="entry")
        self.builder = ir.IRBuilder(block)

        # New scope: params first
        self.scopes.append({})
        for i, (ptype, pname) in enumerate(node.params):
            arg = func.args[i]
            arg.name = pname
            ptr = self.builder.alloca(arg.type, name=pname)
            self.builder.store(arg, ptr)
            self.scopes[-1][pname] = (ptr, arg.type)

        for stmt in node.body:
            self.gen_statement(stmt)

        # Implicit return if the body didn't end with one
        if is_void:
            self.builder.ret_void()
        else:
            # Implicit 'return 0' - real compilers warn about this
            self.builder.ret(ir.Constant(ret_ty, 0))

        self.scopes.pop()

        # Track for calls
        if not hasattr(self, 'functions'):
            self.functions = {}
        self.functions[node.name] = (func, ret_ty)
        return func

    def gen_statement(self, node):
        if isinstance(node, VariableDecl):
            self.gen_var_decl(node)
        elif isinstance(node, PrintNode):
            self.gen_print(node)
        elif isinstance(node, AssignmentNode):
            self.gen_assignment(node)
        elif isinstance(node, IfNode):
            self.gen_if(node)
        elif isinstance(node, CallExpr):
            raise RuntimeError(
                f"Compile error: function call '{node.name}(...)' is not "
                f"implemented in the code generator yet")
    # ---------- declarations & assignment ----------

    def gen_var_decl(self, node):
        llvm_type = self.llvm_type_of(node.var_type)
        ptr = self.builder.alloca(llvm_type, name=node.name)
        self.scopes[-1][node.name] = (ptr, llvm_type)

        if node.value is not None:
            value, vtype = self.gen_expression(node.value)
            self.builder.store(self.coerce(value, vtype, llvm_type), ptr)

    def gen_assignment(self, node):
        if node.value is None:
            return
        parts = node.target.split('.')
        var_name = parts[0]

        var_data = self.lookup_var(var_name)
        if not var_data:
            return

        var_ptr, var_type = var_data
        value, vtype = self.gen_expression(node.value)

        if len(parts) > 1 and isinstance(var_type, ir.LiteralStructType):
            type_name = self.obj_types.get(var_name)
            field_idx = self.struct_fields.get(type_name, {}).get(parts[1], 0)
            field_ptr = self.builder.gep(
                var_ptr, [ir.IntType(32)(0), ir.IntType(32)(field_idx)])
            fty = field_ptr.type.pointee
            self.builder.store(self.coerce(value, vtype, fty), field_ptr)
        else:
            self.builder.store(self.coerce(value, vtype, var_type), var_ptr)

    # ---------- expressions ----------

    def gen_expression(self, node):
        """Returns (llvm_value, llvm_type)."""
        if isinstance(node, NumberLiteral):
            if isinstance(node.value, float):
                return ir.Constant(F64, node.value), F64
            return ir.Constant(I32, node.value), I32

        if isinstance(node, StringLiteral):
            return self.make_global_string(node.raw), I8P

        if isinstance(node, IdentifierExpr):
            data = self.lookup_var(node.name)
            if data is None:
                return ir.Constant(I32, 0), I32
            ptr, ty = data
            return self.builder.load(ptr), ty

        if isinstance(node, MemberAccess):
            data = self.lookup_var(node.obj)
            if data:
                obj_ptr, obj_type = data
                if isinstance(obj_type, ir.LiteralStructType):
                    type_name = self.obj_types.get(node.obj)
                    idx = self.struct_fields.get(type_name, {}).get(node.member, 0)
                    field_ptr = self.builder.gep(
                        obj_ptr, [ir.IntType(32)(0), ir.IntType(32)(idx)])
                    return self.builder.load(field_ptr), field_ptr.type.pointee
            return ir.Constant(I32, 0), I32
        if isinstance(node, UnaryOp):
            val, ty = self.gen_expression(node.operand)
            if node.op == '+':
                return val, ty
            if ty == F64:
                return self.builder.fsub(ir.Constant(F64, 0.0), val), F64
            return self.builder.sub(ir.Constant(I32, 0), val), I32

        if isinstance(node, CallExpr):
            raise RuntimeError(
                f"Compile error: function call '{node.name}(...)' is not "
                f"implemented in the code generator yet")
        if isinstance(node, BinaryOp):
            return self.gen_binary_op(node)

        return ir.Constant(I32, 0), I32

    def gen_binary_op(self, node):
        left, lty = self.gen_expression(node.left)
        right, rty = self.gen_expression(node.right)
        op = node.op

        if lty == F64 and rty == I32:
            right = self.builder.sitofp(right, F64)
            rty = F64
        elif rty == F64 and lty == I32:
            left = self.builder.sitofp(left, F64)
            lty = F64

        if op in ('==', '!=', '<', '>', '<=', '>='):
            if lty == F64:
                return self.builder.fcmp_ordered(op, left, right), ir.IntType(1)
            return self.builder.icmp_signed(op, left, right), ir.IntType(1)

        if lty == F64 or rty == F64:
            if op == '+':
                return self.builder.fadd(left, right), F64
            if op == '-':
                return self.builder.fsub(left, right), F64
            if op == '*':
                return self.builder.fmul(left, right), F64
            if op == '/':
                return self.builder.fdiv(left, right), F64
            if op == '**':
                lf = left if lty == F64 else self.builder.sitofp(left, F64)
                rf = right if rty == F64 else self.builder.sitofp(right, F64)
                return self.builder.call(self.pow_fn, [lf, rf]), F64
        else:
            if op == '+':
                return self.builder.add(left, right), I32
            if op == '-':
                return self.builder.sub(left, right), I32
            if op == '*':
                return self.builder.mul(left, right), I32
            if op == '/':
                return self.builder.sdiv(left, right), I32
            if op == '**':
                lf = self.builder.sitofp(left, F64)
                rf = self.builder.sitofp(right, F64)
                return self.builder.call(self.pow_fn, [lf, rf]), F64
        # Logical ops: non-short-circuit, C-style semantics.
        # Result is i1 (true/false), operands coerced to i32 first.
        if op in ('And', 'Or', 'Xor'):
            li = self.coerce(left, lty, I32)
            ri = self.coerce(right, rty, I32)
            nz_l = self.builder.icmp_signed('!=', li, ir.Constant(I32, 0))
            nz_r = self.builder.icmp_signed('!=', ri, ir.Constant(I32, 0))
            if op == 'And':
                return self.builder.and_(nz_l, nz_r), ir.IntType(1)
            if op == 'Or':
                return self.builder.or_(nz_l, nz_r), ir.IntType(1)
            return self.builder.xor(nz_l, nz_r), ir.IntType(1)

        return ir.Constant(I32, 0), I32

    # ---------- print ----------

    def resolve_name_value(self, name):
        data = self.lookup_var(name)
        if data:
            ptr, ty = data
            val = self.builder.load(ptr)
            if ty == I8P:
                return ("%s", val)
            if ty == F64:
                return ("%f", val)
            return ("%d", val)

        if '.' in name:
            parts = name.split('.')
            obj_data = self.lookup_var(parts[0])
            if obj_data:
                obj_ptr, obj_type = obj_data
                if isinstance(obj_type, ir.LiteralStructType):
                    type_name = self.obj_types.get(parts[0])
                    idx = self.struct_fields.get(type_name, {}).get(parts[1], 0)
                    field_ptr = self.builder.gep(
                        obj_ptr, [ir.IntType(32)(0), ir.IntType(32)(idx)])
                    fty = field_ptr.type.pointee
                    val = self.builder.load(field_ptr)
                    if fty == I8P:
                        return ("%s", val)
                    if fty == F64:
                        return ("%f", val)
                    return ("%d", val)
        return None

    def gen_interpolated_print(self, content):
        fmt_parts = []
        args = []
        current = ""
        i = 0
        while i < len(content):
            if content[i] == '{':
                j = content.find('}', i)
                if j != -1:
                    if current:
                        fmt_parts.append(printf_escape(process_escapes(current)))
                        current = ""
                    name = content[i + 1:j]
                    resolved = self.resolve_name_value(name)
                    if resolved:
                        spec, val = resolved
                        fmt_parts.append(spec)
                        args.append(val)
                    else:
                        fmt_parts.append(printf_escape("{" + name + "}"))
                    i = j + 1
                else:
                    current += content[i]
                    i += 1
            else:
                current += content[i]
                i += 1
        if current:
            fmt_parts.append(printf_escape(process_escapes(current)))
        self.emit_printf("".join(fmt_parts), args)

    def gen_print(self, node):
        value = node.value
        if value is None:
            return

        if isinstance(value, StringLiteral) and value.raw.startswith('\\V"'):
            self.gen_interpolated_print(value.raw[3:-1])
            return

        if isinstance(value, StringLiteral):
            self.emit_printf(printf_escape(process_escapes(value.raw[1:-1])), [])
            return

        llvm_val, llvm_ty = self.gen_expression(value)
        if llvm_ty == I8P:
            self.emit_printf("%s", [llvm_val])
        elif llvm_ty == F64:
            self.emit_printf("%f", [llvm_val])
        else:
            self.emit_printf("%d", [llvm_val])
    # ---------- control flow ----------

    def gen_if(self, node):
        """Compiles If / Else If / Else chains into LLVM basic blocks.

        Every branch gets its own block; every branch jumps to a shared
        merge block at the end. Else If chains are compiled recursively:
        each 'no' path contains the next If as a nested decision.
        """
        func = self.builder.function

        def cond_to_i1(cond_expr):
            """Any expression -> i1 (boolean) for use as a branch condition."""
            val, ty = self.gen_expression(cond_expr)
            if ty == ir.IntType(1):
                return val
            if ty == F64:
                return self.builder.fcmp_ordered(
                    '!=', val, ir.Constant(F64, 0.0))
            return self.builder.icmp_signed(
                '!=', val, ir.Constant(ty, 0))

        def emit_branch(i, merge_bb):
            """Emit branch i. When its condition is false, fall to the
            next branch (or else, or straight to merge)."""
            cond_expr, body = node.branches[i]
            cond_i1 = cond_to_i1(cond_expr)

            then_bb = func.append_basic_block(f"if.then.{i}")
            next_bb = func.append_basic_block(f"if.next.{i}")

            self.builder.cbranch(cond_i1, then_bb, next_bb)

            # THEN block: run body, jump to merge
            self.builder.position_at_end(then_bb)
            for stmt in body:
                self.gen_statement(stmt)
            self.builder.branch(merge_bb)

            # NEXT block: either the next Else If, the Else, or nothing
            self.builder.position_at_end(next_bb)
            if i + 1 < len(node.branches):
                emit_branch(i + 1, merge_bb)
            elif node.else_body is not None:
                for stmt in node.else_body:
                    self.gen_statement(stmt)
                self.builder.branch(merge_bb)
            else:
                self.builder.branch(merge_bb)

        merge_bb = func.append_basic_block("if.end")
        emit_branch(0, merge_bb)
        self.builder.position_at_end(merge_bb)