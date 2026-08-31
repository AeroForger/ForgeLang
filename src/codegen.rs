use cranelift_codegen::ir::{
    types, AbiParam, FuncRef, GlobalValue, InstBuilder, MachMemFlags,
    StackSlotData, StackSlotKind, UserFuncName, Value,
};
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::settings;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{DataDescription, DataId, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::ast::*;
use crate::errors::{ForgeError, ForgeResult};
use std::collections::{HashMap, HashSet};

/// Module-level identifiers for external C libraries and global format strings.
pub struct RuntimeSymbols {
    pub printf_id: FuncId,
    pub puts_id: FuncId,
    pub fputs_id: FuncId,
    pub stdout_id: DataId,
    pub scanf_id: FuncId,
    pub pow_id: FuncId,
    pub exit_id: FuncId,
    pub int_fmt_id: DataId,
    pub float_fmt_id: DataId,
    pub int_inline_fmt_id: DataId,
    pub float_inline_fmt_id: DataId,
    pub int_scanf_id: DataId,
    pub float_scanf_id: DataId,
    pub str_scanf_id: DataId,
}

/// Function-local Cranelift handles (GlobalValues and FuncRefs) bound to a specific function's IR context.
pub struct RuntimeRefs {
    pub printf: FuncRef,
    pub puts: FuncRef,
    pub fputs: FuncRef,
    pub stdout: GlobalValue,
    pub scanf: FuncRef,
    pub pow: FuncRef,
    pub exit: FuncRef,
    pub int_fmt: GlobalValue,
    pub float_fmt: GlobalValue,
    pub int_inline_fmt: GlobalValue,
    pub float_inline_fmt: GlobalValue,
    pub int_scanf: GlobalValue,
    pub float_scanf: GlobalValue,
    pub str_scanf: GlobalValue,
}

/// Owns module-level compilation state and external dependencies.
pub struct CodeGenContext {
    pub module: ObjectModule,
    pub ptr_type: types::Type,
    pub runtime_symbols: RuntimeSymbols,
    pub string_pool: HashMap<String, DataId>,
}

impl CodeGenContext {
    pub fn new() -> ForgeResult<Self> {
        let flag_builder = settings::builder();
        let isa_builder = cranelift_native::builder()
            .map_err(|e| ForgeError::codegen(format!("Native ISA error: {}", e)))?;
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| ForgeError::codegen(format!("ISA finish error: {}", e)))?;

        let builder = ObjectBuilder::new(isa, "furnace.o", cranelift_module::default_libcall_names())
            .map_err(|e| ForgeError::codegen(format!("ObjectBuilder error: {}", e)))?;
        let mut module = ObjectModule::new(builder);

        let ptr_type = module.target_config().pointer_type();

        // Declare external C functions
        let mut printf_sig = module.make_signature();
        printf_sig.params.push(AbiParam::new(ptr_type));
        printf_sig.params.push(AbiParam::new(types::F64));
        printf_sig.returns.push(AbiParam::new(types::I32));
        let printf_id = module
            .declare_function("printf", Linkage::Import, &printf_sig)
            .map_err(|e| ForgeError::codegen(format!("declare printf: {}", e)))?;

        let mut puts_sig = module.make_signature();
        puts_sig.params.push(AbiParam::new(ptr_type));
        puts_sig.returns.push(AbiParam::new(types::I32));
        let puts_id = module
            .declare_function("puts", Linkage::Import, &puts_sig)
            .map_err(|e| ForgeError::codegen(format!("declare puts: {}", e)))?;

        let mut fputs_sig = module.make_signature();
        fputs_sig.params.push(AbiParam::new(ptr_type));
        fputs_sig.params.push(AbiParam::new(ptr_type));
        fputs_sig.returns.push(AbiParam::new(types::I32));
        let fputs_id = module
            .declare_function("fputs", Linkage::Import, &fputs_sig)
            .map_err(|e| ForgeError::codegen(format!("declare fputs: {}", e)))?;

        let stdout_id = module
            .declare_data("stdout", Linkage::Import, false, false)
            .map_err(|e| ForgeError::codegen(format!("declare stdout: {}", e)))?;

        let mut scanf_sig = module.make_signature();
        scanf_sig.params.push(AbiParam::new(ptr_type));
        scanf_sig.params.push(AbiParam::new(ptr_type));
        scanf_sig.returns.push(AbiParam::new(types::I32));
        let scanf_id = module
            .declare_function("scanf", Linkage::Import, &scanf_sig)
            .map_err(|e| ForgeError::codegen(format!("declare scanf: {}", e)))?;

        let mut pow_sig = module.make_signature();
        pow_sig.params.push(AbiParam::new(types::F64));
        pow_sig.params.push(AbiParam::new(types::F64));
        pow_sig.returns.push(AbiParam::new(types::F64));
        let pow_id = module
            .declare_function("pow", Linkage::Import, &pow_sig)
            .map_err(|e| ForgeError::codegen(format!("declare pow: {}", e)))?;

        let mut exit_sig = module.make_signature();
        exit_sig.params.push(AbiParam::new(types::I32));
        let exit_id = module
            .declare_function("exit", Linkage::Import, &exit_sig)
            .map_err(|e| ForgeError::codegen(format!("declare exit: {}", e)))?;

        // Define global format strings
        let int_fmt_id = Self::define_string_in_module(&mut module, "int_fmt", b"%.0f\n\0")?;
        let float_fmt_id = Self::define_string_in_module(&mut module, "float_fmt", b"%f\n\0")?;
        let int_inline_fmt_id = Self::define_string_in_module(&mut module, "int_inline_fmt", b"%.0f\0")?;
        let float_inline_fmt_id = Self::define_string_in_module(&mut module, "float_inline_fmt", b"%f\0")?;
        let int_scanf_id = Self::define_string_in_module(&mut module, "int_scanf", b"%d\0")?;
        let float_scanf_id = Self::define_string_in_module(&mut module, "float_scanf", b"%lf\0")?;
        let str_scanf_id = Self::define_string_in_module(&mut module, "str_scanf", b"%255s\0")?;

        let runtime_symbols = RuntimeSymbols {
            printf_id,
            puts_id,
            fputs_id,
            stdout_id,
            scanf_id,
            pow_id,
            exit_id,
            int_fmt_id,
            float_fmt_id,
            int_inline_fmt_id,
            float_inline_fmt_id,
            int_scanf_id,
            float_scanf_id,
            str_scanf_id,
        };

        Ok(Self {
            module,
            ptr_type,
            runtime_symbols,
            string_pool: HashMap::new(),
        })
    }

    pub fn define_string(&mut self, name: &str, bytes: &[u8]) -> ForgeResult<DataId> {
        Self::define_string_in_module(&mut self.module, name, bytes)
    }

    fn define_string_in_module(module: &mut ObjectModule, name: &str, bytes: &[u8]) -> ForgeResult<DataId> {
        let mut data_ctx = DataDescription::new();
        data_ctx.define(bytes.to_vec().into_boxed_slice());
        let id = module
            .declare_data(name, Linkage::Export, false, false)
            .map_err(|e| ForgeError::codegen(format!("declare_data: {}", e)))?;
        module
            .define_data(id, &data_ctx)
            .map_err(|e| ForgeError::codegen(format!("define_data: {}", e)))?;
        Ok(id)
    }

    pub fn declare_runtime_in_func(&mut self, func: &mut cranelift_codegen::ir::Function) -> RuntimeRefs {
        let int_fmt = self.module.declare_data_in_func(self.runtime_symbols.int_fmt_id, func);
        let float_fmt = self.module.declare_data_in_func(self.runtime_symbols.float_fmt_id, func);
        let int_inline_fmt = self.module.declare_data_in_func(self.runtime_symbols.int_inline_fmt_id, func);
        let float_inline_fmt = self.module.declare_data_in_func(self.runtime_symbols.float_inline_fmt_id, func);
        let int_scanf = self.module.declare_data_in_func(self.runtime_symbols.int_scanf_id, func);
        let float_scanf = self.module.declare_data_in_func(self.runtime_symbols.float_scanf_id, func);
        let str_scanf = self.module.declare_data_in_func(self.runtime_symbols.str_scanf_id, func);
        let printf = self.module.declare_func_in_func(self.runtime_symbols.printf_id, func);
        let puts = self.module.declare_func_in_func(self.runtime_symbols.puts_id, func);
        let fputs = self.module.declare_func_in_func(self.runtime_symbols.fputs_id, func);
        let stdout = self.module.declare_data_in_func(self.runtime_symbols.stdout_id, func);
        let scanf = self.module.declare_func_in_func(self.runtime_symbols.scanf_id, func);
        let pow = self.module.declare_func_in_func(self.runtime_symbols.pow_id, func);
        let exit = self.module.declare_func_in_func(self.runtime_symbols.exit_id, func);

        RuntimeRefs {
            printf,
            puts,
            fputs,
            stdout,
            scanf,
            pow,
            exit,
            int_fmt,
            float_fmt,
            int_inline_fmt,
            float_inline_fmt,
            int_scanf,
            float_scanf,
            str_scanf,
        }
    }
}

/// Owns the active function's IR construction and function-local compilation state.
pub struct FunctionCompiler<'a, 'ctx> {
    pub ctx: &'a mut CodeGenContext,
    pub builder: FunctionBuilder<'ctx>,
    pub runtime: RuntimeRefs,
    pub string_map: HashMap<String, GlobalValue>,
    pub var_map: HashMap<String, Variable>,
    pub float_vars: HashSet<String>,
    pub string_vars: HashSet<String>,
    pub break_targets: Vec<cranelift_codegen::ir::Block>,
}

impl<'a, 'ctx> FunctionCompiler<'a, 'ctx> {
    pub fn new(
        ctx: &'a mut CodeGenContext,
        builder: FunctionBuilder<'ctx>,
        runtime: RuntimeRefs,
        string_map: HashMap<String, GlobalValue>,
    ) -> Self {
        Self {
            ctx,
            builder,
            runtime,
            string_map,
            var_map: HashMap::new(),
            float_vars: HashSet::new(),
            string_vars: HashSet::new(),
            break_targets: Vec::new(),
        }
    }

    fn is_float_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Number(n) => n.is_float,
            Expr::Identifier(name) => self.float_vars.contains(name),
            Expr::BinaryOp { lhs, rhs, .. } => self.is_float_expr(lhs) || self.is_float_expr(rhs),
            Expr::UnaryOp { operand, .. } => self.is_float_expr(operand),
            Expr::Input(InputNode { subtype: Some(Subtype::Float) }) => true,
            _ => false,
        }
    }

    fn compile_namespace_call(&mut self, namespace: &str, method: &str, _args: &[Expr]) -> ForgeResult<Value> {
        match (namespace, method) {
            ("Program", "Stop") => {
                let zero = self.builder.ins().iconst(types::I32, 0);
                self.builder.ins().call(self.runtime.exit, &[zero]);
                self.builder.ins().return_(&[zero]);
                let dead_block = self.builder.create_block();
                self.builder.switch_to_block(dead_block);
                self.builder.seal_block(dead_block);
                Ok(zero)
            }
            _ => Err(ForgeError::codegen(format!("Unsupported namespace call: {}.{}", namespace, method))),
        }
    }

    pub fn compile_statement(&mut self, stmt: &Statement) -> ForgeResult<()> {
        match stmt {
            Statement::VarDecl(v) => {
                let ty = if type_decl_is_float(&v.type_decl) {
                    types::F64
                } else if matches!(v.type_decl, TypeDecl::Weld) {
                    self.ctx.ptr_type
                } else {
                    types::I32
                };
                let var = self.builder.declare_var(ty);
                self.var_map.insert(v.name.clone(), var);
                if ty == types::F64 {
                    self.float_vars.insert(v.name.clone());
                }
                if ty == self.ctx.ptr_type {
                    self.string_vars.insert(v.name.clone());
                }

                if let Some(init) = &v.initializer {
                    let val = self.compile_expr(init)?;
                    self.builder.def_var(var, val);
                } else {
                    let zero = if ty == types::F64 {
                        self.builder.ins().f64const(0.0)
                    } else {
                        self.builder.ins().iconst(ty, 0).into()
                    };
                    self.builder.def_var(var, zero);
                }
            }
            Statement::Print(p) => {
                if let Expr::Str(parts) = &p.expr {
                    if parts.iter().any(|part| matches!(part, StringPart::Interp(_))) {
                        for part in parts {
                            match part {
                                StringPart::Literal(text) => {
                                    if let Some(gv) = self.string_map.get(text) {
                                        let value = self.builder.ins().symbol_value(self.ctx.ptr_type, *gv);
                                        let stdout_ptr = self.builder.ins().symbol_value(self.ctx.ptr_type, self.runtime.stdout);
                                        let stdout = self.builder.ins().load(self.ctx.ptr_type, MachMemFlags::new(), stdout_ptr, 0);
                                        self.builder.ins().call(self.runtime.fputs, &[value, stdout]);
                                    }
                                }
                                StringPart::Interp(name) => {
                                    let value = self.compile_expr(&Expr::Identifier(name.clone()))?;
                                    if self.string_map.contains_key(name) || self.string_vars.contains(name) {
                                        let stdout_ptr = self.builder.ins().symbol_value(self.ctx.ptr_type, self.runtime.stdout);
                                        let stdout = self.builder.ins().load(self.ctx.ptr_type, MachMemFlags::new(), stdout_ptr, 0);
                                        self.builder.ins().call(self.runtime.fputs, &[value, stdout]);
                                    } else {
                                        let fmt = if self.float_vars.contains(name) {
                                            self.runtime.float_inline_fmt
                                        } else {
                                            self.runtime.int_inline_fmt
                                        };
                                        let fmt_value = self.builder.ins().symbol_value(self.ctx.ptr_type, fmt);
                                        let numeric_value = if self.float_vars.contains(name) {
                                            value
                                        } else {
                                            self.builder.ins().fcvt_from_sint(types::F64, value)
                                        };
                                        self.builder.ins().call(self.runtime.printf, &[fmt_value, numeric_value]);
                                    }
                                }
                            }
                        }
                        let newline = self.string_map.get("\n").map(|gv| self.builder.ins().symbol_value(self.ctx.ptr_type, *gv));
                        if let Some(value) = newline {
                            let stdout_ptr = self.builder.ins().symbol_value(self.ctx.ptr_type, self.runtime.stdout);
                            let stdout = self.builder.ins().load(self.ctx.ptr_type, MachMemFlags::new(), stdout_ptr, 0);
                            self.builder.ins().call(self.runtime.fputs, &[value, stdout]);
                        }
                        return Ok(());
                    }
                }
                let val = self.compile_expr(&p.expr)?;
                let is_string = matches!(&p.expr, Expr::Str(_))
                    || matches!(&p.expr, Expr::Identifier(name) if self.string_vars.contains(name));
                let is_float = self.is_float_expr(&p.expr);

                if is_string {
                    self.builder.ins().call(self.runtime.puts, &[val]);
                } else {
                    let fmt_gv = if is_float {
                        self.runtime.float_fmt
                    } else {
                        self.runtime.int_fmt
                    };
                    let fmt_val = self.builder.ins().symbol_value(self.ctx.ptr_type, fmt_gv);
                    let numeric_val = if is_float {
                        val
                    } else {
                        self.builder.ins().fcvt_from_sint(types::F64, val)
                    };
                    self.builder.ins().call(self.runtime.printf, &[fmt_val, numeric_val]);
                }
            }
            Statement::Assignment(a) => {
                if a.target_path.len() == 1 {
                    let var_name = &a.target_path[0];
                    let var = self.var_map.get(var_name).copied();
                    if let Some(var) = var {
                        let val = self.compile_expr(&a.value)?;
                        self.builder.def_var(var, val);
                    } else {
                        return Err(ForgeError::codegen(format!("Undefined variable: {}", var_name)));
                    }
                } else {
                    return Err(ForgeError::codegen("Member assignment not supported yet"));
                }
            }
            Statement::If(if_node) => {
                let merge_block = self.builder.create_block();
                let is_top_level_if = self.break_targets.is_empty();
                if is_top_level_if {
                    self.break_targets.push(merge_block);
                }

                for (cond, body) in &if_node.branches {
                    let cond_val = self.compile_expr(cond)?;
                    let cond_bool = self.builder.ins().icmp_imm_u(IntCC::NotEqual, cond_val, 0);

                    let then_block = self.builder.create_block();
                    let next_cond_block = self.builder.create_block();

                    self.builder.ins().brif(cond_bool, then_block, &[], next_cond_block, &[]);

                    self.builder.switch_to_block(then_block);
                    self.builder.seal_block(then_block);

                    for s in body {
                        self.compile_statement(s)?;
                    }
                    self.builder.ins().jump(merge_block, &[]);

                    self.builder.switch_to_block(next_cond_block);
                    self.builder.seal_block(next_cond_block);
                }

                if let Some(else_body) = &if_node.else_body {
                    for s in else_body {
                        self.compile_statement(s)?;
                    }
                }
                self.builder.ins().jump(merge_block, &[]);

                if is_top_level_if {
                    self.break_targets.pop();
                }

                self.builder.switch_to_block(merge_block);
                self.builder.seal_block(merge_block);
            }
            Statement::While(while_node) => {
                let header_block = self.builder.create_block();
                let body_block = self.builder.create_block();
                let exit_block = self.builder.create_block();
                self.builder.ins().jump(header_block, &[]);

                self.builder.switch_to_block(header_block);
                let condition = self.compile_expr(&while_node.condition)?;
                let condition = self.builder.ins().icmp_imm_u(IntCC::NotEqual, condition, 0);
                self.builder.ins().brif(condition, body_block, &[], exit_block, &[]);

                self.break_targets.push(exit_block);

                self.builder.switch_to_block(body_block);
                for s in &while_node.body {
                    self.compile_statement(s)?;
                }
                self.builder.ins().jump(header_block, &[]);
                self.builder.seal_block(body_block);
                self.builder.seal_block(header_block);

                self.break_targets.pop();

                self.builder.switch_to_block(exit_block);
                self.builder.seal_block(exit_block);
            }
            Statement::Stop => {
                let target = self.break_targets.last().copied().ok_or_else(|| {
                    ForgeError::codegen("Stop used outside of break target")
                })?;
                self.builder.ins().jump(target, &[]);
                let dead_block = self.builder.create_block();
                self.builder.switch_to_block(dead_block);
                self.builder.seal_block(dead_block);
            }
            Statement::ExprStmt(e) => {
                self.compile_expr(e)?;
            }
            Statement::DataDecl(_) | Statement::ObjectDecl(_) | Statement::Use(_) => {
                // Ignore for now, validly parsed but no codegen action needed
            }
            _ => return Err(ForgeError::codegen(format!("Unsupported statement: {:?}", stmt))),
        }
        Ok(())
    }

    pub fn compile_expr(&mut self, expr: &Expr) -> ForgeResult<Value> {
        match expr {
            Expr::Number(n) => {
                if n.is_float {
                    Ok(self.builder.ins().f64const(n.float_val))
                } else {
                    Ok(self.builder.ins().iconst(types::I32, n.int_val))
                }
            }
            Expr::Identifier(name) => {
                if let Some(var) = self.var_map.get(name).copied() {
                    Ok(self.builder.use_var(var))
                } else if let Some(gv) = self.string_map.get(name).copied() {
                    Ok(self.builder.ins().symbol_value(self.ctx.ptr_type, gv))
                } else {
                    Err(ForgeError::codegen(format!("Undefined variable: {}", name)))
                }
            }
            Expr::NamespaceCall { namespace, method, args } => {
                self.compile_namespace_call(namespace, method, args)
            }
            Expr::BinaryOp { op, lhs, rhs } => {
                let l = self.compile_expr(lhs)?;
                let r = self.compile_expr(rhs)?;

                let is_float = self.is_float_expr(lhs) || self.is_float_expr(rhs);
                if is_float {
                    let l_f = if self.is_float_expr(lhs) {
                        l
                    } else {
                        self.builder.ins().fcvt_from_sint(types::F64, l)
                    };
                    let r_f = if self.is_float_expr(rhs) {
                        r
                    } else {
                        self.builder.ins().fcvt_from_sint(types::F64, r)
                    };
                    match op {
                        BinOp::Add => Ok(self.builder.ins().fadd(l_f, r_f)),
                        BinOp::Sub => Ok(self.builder.ins().fsub(l_f, r_f)),
                        BinOp::Mul => Ok(self.builder.ins().fmul(l_f, r_f)),
                        BinOp::Div => Ok(self.builder.ins().fdiv(l_f, r_f)),
                        BinOp::Pow => {
                            let pow_inst = self.builder.ins().call(self.runtime.pow, &[l_f, r_f]);
                            Ok(self.builder.inst_results(pow_inst)[0])
                        }
                        _ => Err(ForgeError::codegen(format!("Unsupported float op: {:?}", op))),
                    }
                } else {
                    match op {
                        BinOp::Add => Ok(self.builder.ins().iadd(l, r)),
                        BinOp::Sub => Ok(self.builder.ins().isub(l, r)),
                        BinOp::Mul => Ok(self.builder.ins().imul(l, r)),
                        BinOp::Div => Ok(self.builder.ins().sdiv(l, r)),
                        BinOp::Pow => {
                            let l_f = self.builder.ins().fcvt_from_sint(types::F64, l);
                            let r_f = self.builder.ins().fcvt_from_sint(types::F64, r);
                            let pow_inst = self.builder.ins().call(self.runtime.pow, &[l_f, r_f]);
                            let res_f = self.builder.inst_results(pow_inst)[0];
                            Ok(self.builder.ins().fcvt_to_sint(types::I32, res_f))
                        }
                        BinOp::Eq => Ok(self.builder.ins().icmp(IntCC::Equal, l, r)),
                        BinOp::Ne => Ok(self.builder.ins().icmp(IntCC::NotEqual, l, r)),
                        BinOp::Lt => Ok(self.builder.ins().icmp(IntCC::SignedLessThan, l, r)),
                        BinOp::Gt => Ok(self.builder.ins().icmp(IntCC::SignedGreaterThan, l, r)),
                        BinOp::Le => Ok(self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, l, r)),
                        BinOp::Ge => Ok(self.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, l, r)),
                        BinOp::And => Ok(self.builder.ins().band(l, r)),
                        BinOp::Or => Ok(self.builder.ins().bor(l, r)),
                        BinOp::Xor => Ok(self.builder.ins().bxor(l, r)),
                    }
                }
            }
            Expr::UnaryOp { op, operand } => {
                let val = self.compile_expr(operand)?;
                match op {
                    UnOp::Plus => Ok(val),
                    UnOp::Neg => {
                        if self.is_float_expr(operand) {
                            Ok(self.builder.ins().fneg(val))
                        } else {
                            Ok(self.builder.ins().ineg(val))
                        }
                    }
                }
            }
            Expr::Input(input_node) => {
                let (fmt_gv, ty) = match input_node.subtype {
                    Some(Subtype::Int) => (self.runtime.int_scanf, types::I32),
                    Some(Subtype::Float) => (self.runtime.float_scanf, types::F64),
                    _ => {
                        // String input
                        let slot_data = StackSlotData::new(StackSlotKind::ExplicitSlot, 256, 0);
                        let slot = self.builder.create_sized_stack_slot(slot_data);
                        let slot_ptr = self.builder.ins().stack_addr(self.ctx.ptr_type, slot, 0);
                        let fmt_val = self.builder.ins().symbol_value(self.ctx.ptr_type, self.runtime.str_scanf);
                        self.builder.ins().call(self.runtime.scanf, &[fmt_val, slot_ptr]);
                        return Ok(slot_ptr);
                    }
                };

                let slot_data = StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 0);
                let slot = self.builder.create_sized_stack_slot(slot_data);

                let slot_ptr = self.builder.ins().stack_addr(self.ctx.ptr_type, slot, 0);
                let fmt_val = self.builder.ins().symbol_value(self.ctx.ptr_type, fmt_gv);

                self.builder.ins().call(self.runtime.scanf, &[fmt_val, slot_ptr]);
                Ok(self.builder.ins().stack_load(self.ctx.ptr_type, ty, slot, 0))
            }
            Expr::Str(parts) => {
                let mut s = String::new();
                for part in parts {
                    if let StringPart::Literal(l) = part {
                        s.push_str(l);
                    }
                }
                if let Some(gv) = self.string_map.get(&s).copied() {
                    Ok(self.builder.ins().symbol_value(self.ctx.ptr_type, gv))
                } else {
                    Err(ForgeError::codegen(format!("Undefined string literal: {}", s)))
                }
            }
            _ => Err(ForgeError::codegen(format!("Unsupported expr: {:?}", expr))),
        }
    }
}

pub fn compile(program: &Program, obj_path: &std::path::Path, _link_math: bool) -> ForgeResult<()> {
    let mut ctx = CodeGenContext::new()?;

    // Define main
    let mut main_sig = ctx.module.make_signature();
    main_sig.returns.push(AbiParam::new(types::I32));
    let main_id = ctx
        .module
        .declare_function("main", Linkage::Export, &main_sig)
        .map_err(|e| ForgeError::codegen(format!("declare main: {}", e)))?;

    let mut clif_ctx = ctx.module.make_context();
    let mut fn_builder_ctx = FunctionBuilderContext::new();

    clif_ctx.func.signature = main_sig.clone();
    clif_ctx.func.name = UserFuncName::user(0, main_id.as_u32());

    // In modern Cranelift, we must declare data/funcs inside the function context
    let runtime = ctx.declare_runtime_in_func(&mut clif_ctx.func);

    // Find Main
    let main_func = program.statements.iter().find_map(|s| {
        if let Statement::FunctionDecl(f) = s {
            if f.name == "Main" { return Some(f); }
        }
        None
    }).ok_or_else(|| ForgeError::codegen("No Main function found"))?;
    let functions: HashMap<String, FunctionDecl> = program.statements.iter().filter_map(|stmt| {
        if let Statement::FunctionDecl(function) = stmt {
            Some((function.name.clone(), function.clone()))
        } else {
            None
        }
    }).collect();
    let expanded_body = expand_function_calls(&main_func.body, &functions)?;

    // Pre-pass to collect all string literals so we can declare them in the function context
    let mut string_map: HashMap<String, GlobalValue> = HashMap::new();
    for stmt in &expanded_body {
        collect_strings(stmt, &mut string_map, &mut ctx, &mut clif_ctx)?;
    }
    for stmt in &program.statements {
        if let Statement::ObjectDecl(object) = stmt {
            for (path, value) in &object.inits {
                if let Expr::Str(parts) = value {
                    let literal = parts.iter().filter_map(|part| match part {
                        StringPart::Literal(text) => Some(text.as_str()),
                        StringPart::Interp(_) => None,
                    }).collect::<String>();
                    let name = format!("{}.{}", object.name, path.iter().skip(1).cloned().collect::<Vec<_>>().join("."));
                    if !string_map.contains_key(&literal) {
                        let id = ctx.define_string(&format!("str_lit_{}", string_map.len()), format!("{}\0", literal).as_bytes())?;
                        let gv = ctx.module.declare_data_in_func(id, &mut clif_ctx.func);
                        string_map.insert(literal.clone(), gv);
                    }
                    if let Some(gv) = string_map.get(&literal) {
                        string_map.insert(name, *gv);
                    }
                }
            }
        }
    }
    if !string_map.contains_key("\n") {
        let id = ctx.define_string("str_newline", b"\n\0")?;
        let gv = ctx.module.declare_data_in_func(id, &mut clif_ctx.func);
        string_map.insert("\n".to_string(), gv);
    }

    {
        let mut builder = FunctionBuilder::new(&mut clif_ctx.func, &mut fn_builder_ctx);
        let block = builder.create_block();
        builder.switch_to_block(block);
        builder.seal_block(block);

        let mut compiler = FunctionCompiler::new(&mut ctx, builder, runtime, string_map);

        for stmt in &expanded_body {
            compiler.compile_statement(stmt)?;
        }

        let ret_val = compiler.builder.ins().iconst(types::I32, 0);
        compiler.builder.ins().return_(&[ret_val]);
    }

    ctx.module
        .define_function(main_id, &mut clif_ctx)
        .map_err(|e| ForgeError::codegen(format!("define main: {}", e)))?;

    ctx.module.clear_context(&mut clif_ctx);

    let product = ctx.module.finish();
    let bytes = product.emit().map_err(|e| ForgeError::codegen(format!("emit object: {}", e)))?;
    std::fs::write(obj_path, bytes).map_err(|e| ForgeError::codegen(format!("write object: {}", e)))?;

    Ok(())
}

fn collect_strings(
    stmt: &Statement,
    string_map: &mut HashMap<String, GlobalValue>,
    ctx: &mut CodeGenContext,
    clif_ctx: &mut cranelift_codegen::Context,
) -> ForgeResult<()> {
    match stmt {
        Statement::Print(p) => {
            if let Expr::Str(parts) = &p.expr {
                let mut s = String::new();
                for part in parts {
                    if let StringPart::Literal(l) = part { s.push_str(l); }
                }
                if !string_map.contains_key(&s) {
                    let name = format!("str_lit_{}", string_map.len());
                    let id = ctx.define_string(&name, format!("{}\0", s).as_bytes())?;
                    let gv = ctx.module.declare_data_in_func(id, &mut clif_ctx.func);
                    string_map.insert(s.clone(), gv);
                }
                for part in parts {
                    if let StringPart::Literal(text) = part {
                        if !string_map.contains_key(text) {
                            let name = format!("str_lit_{}", string_map.len());
                            let id = ctx.define_string(&name, format!("{}\0", text).as_bytes())?;
                            let gv = ctx.module.declare_data_in_func(id, &mut clif_ctx.func);
                            string_map.insert(text.clone(), gv);
                        }
                    }
                }
            }
        }
        Statement::If(if_node) => {
            for (_, body) in &if_node.branches {
                for s in body { collect_strings(s, string_map, ctx, clif_ctx)?; }
            }
            if let Some(else_body) = &if_node.else_body {
                for s in else_body { collect_strings(s, string_map, ctx, clif_ctx)?; }
            }
        }
        Statement::While(while_node) => {
            collect_strings_in_body(&while_node.body, string_map, ctx, clif_ctx)?;
        }
        _ => {}
    }
    Ok(())
}

fn collect_strings_in_body(
    body: &[Statement],
    string_map: &mut HashMap<String, GlobalValue>,
    ctx: &mut CodeGenContext,
    clif_ctx: &mut cranelift_codegen::Context,
) -> ForgeResult<()> {
    for stmt in body {
        collect_strings(stmt, string_map, ctx, clif_ctx)?;
    }
    Ok(())
}

fn expand_function_calls(
    body: &[Statement],
    functions: &HashMap<String, FunctionDecl>,
) -> ForgeResult<Vec<Statement>> {
    let mut expanded = Vec::new();
    for statement in body {
        match statement {
            Statement::ExprStmt(Expr::Call { callee, args }) if args.is_empty() => {
                let function = functions.get(callee).ok_or_else(|| {
                    ForgeError::codegen(format!("Undefined function: {}", callee))
                })?;
                if function.ret_kind != RetKind::Nunction || !function.params.is_empty() {
                    return Err(ForgeError::codegen(format!(
                        "Only zero-argument Nunction calls are supported: {}",
                        callee
                    )));
                }
                expanded.extend(expand_function_calls(&function.body, functions)?);
            }
            Statement::If(node) => {
                let mut node = node.clone();
                node.branches = node.branches.iter().map(|(condition, branch)| {
                    Ok((condition.clone(), expand_function_calls(branch, functions)?))
                }).collect::<ForgeResult<Vec<_>>>()?;
                node.else_body = node.else_body.as_ref().map(|branch| {
                    expand_function_calls(branch, functions)
                }).transpose()?;
                expanded.push(Statement::If(node));
            }
            Statement::While(node) => {
                let mut node = node.clone();
                node.body = expand_function_calls(&node.body, functions)?;
                expanded.push(Statement::While(node));
            }
            Statement::Stop => expanded.push(Statement::Stop),
            _ => expanded.push(statement.clone()),
        }
    }
    Ok(expanded)
}

fn type_decl_is_float(type_decl: &TypeDecl) -> bool {
    matches!(type_decl, TypeDecl::Number(Subtype::Float))
}