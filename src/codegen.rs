use cranelift_codegen::ir::{AbiParam, FuncRef, GlobalValue, InstBuilder, MachMemFlags, StackSlotData, StackSlotKind, UserFuncName, Value};
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::settings;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{DataDescription, DataId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::ast::*;
use crate::errors::{ForgeError, ForgeResult};
use std::collections::{HashMap, HashSet};

pub fn compile(program: &Program, obj_path: &std::path::Path, _link_math: bool) -> ForgeResult<()> {
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

    // Define global format strings
    let int_fmt_id = define_string(&mut module, "int_fmt", b"%.0f\n\0")?;
    let float_fmt_id = define_string(&mut module, "float_fmt", b"%f\n\0")?;
    let int_inline_fmt_id = define_string(&mut module, "int_inline_fmt", b"%.0f\0")?;
    let float_inline_fmt_id = define_string(&mut module, "float_inline_fmt", b"%f\0")?;
    let int_scanf_id = define_string(&mut module, "int_scanf", b"%d\0")?;
    let float_scanf_id = define_string(&mut module, "float_scanf", b"%lf\0")?;
    let str_scanf_id = define_string(&mut module, "str_scanf", b"%255s\0")?;

    // Define main
    let mut main_sig = module.make_signature();
    main_sig.returns.push(AbiParam::new(types::I32));
    let main_id = module
        .declare_function("main", Linkage::Export, &main_sig)
        .map_err(|e| ForgeError::codegen(format!("declare main: {}", e)))?;

    let mut ctx = module.make_context();
    let mut fn_builder_ctx = FunctionBuilderContext::new();

    ctx.func.signature = main_sig.clone();
    ctx.func.name = UserFuncName::user(0, main_id.as_u32());

    // In modern Cranelift, we must declare data/funcs inside the function context
    let int_fmt_gv = module.declare_data_in_func(int_fmt_id, &mut ctx.func);
    let float_fmt_gv = module.declare_data_in_func(float_fmt_id, &mut ctx.func);
    let int_inline_fmt_gv = module.declare_data_in_func(int_inline_fmt_id, &mut ctx.func);
    let float_inline_fmt_gv = module.declare_data_in_func(float_inline_fmt_id, &mut ctx.func);
    let int_scanf_gv = module.declare_data_in_func(int_scanf_id, &mut ctx.func);
    let float_scanf_gv = module.declare_data_in_func(float_scanf_id, &mut ctx.func);
    let str_scanf_gv = module.declare_data_in_func(str_scanf_id, &mut ctx.func);
    let printf_ref = module.declare_func_in_func(printf_id, &mut ctx.func);
    let puts_ref = module.declare_func_in_func(puts_id, &mut ctx.func);
    let fputs_ref = module.declare_func_in_func(fputs_id, &mut ctx.func);
    let stdout_gv = module.declare_data_in_func(stdout_id, &mut ctx.func);
    let scanf_ref = module.declare_func_in_func(scanf_id, &mut ctx.func);
    let pow_ref = module.declare_func_in_func(pow_id, &mut ctx.func);

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
        collect_strings(stmt, &mut string_map, &mut module, &mut ctx)?;
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
                        let id = define_string(&mut module, &format!("str_lit_{}", string_map.len()), format!("{}\0", literal).as_bytes())?;
                        let gv = module.declare_data_in_func(id, &mut ctx.func);
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
        let id = define_string(&mut module, "str_newline", b"\n\0")?;
        let gv = module.declare_data_in_func(id, &mut ctx.func);
        string_map.insert("\n".to_string(), gv);
    }

    {
        let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut fn_builder_ctx);
        let block = bcx.create_block();
        bcx.switch_to_block(block);
        bcx.seal_block(block);

        let mut var_map: HashMap<String, Variable> = HashMap::new();
        let mut float_vars: HashSet<String> = HashSet::new();
        let mut string_vars: HashSet<String> = HashSet::new();

        for stmt in &expanded_body {
            build_statement(
                &mut bcx,
                stmt,
                &mut var_map,
                &mut float_vars,
                &mut string_vars,
                &string_map,
                printf_ref,
                puts_ref,
                fputs_ref,
                stdout_gv,
                scanf_ref,
                pow_ref,
                int_fmt_gv,
                float_fmt_gv,
                int_inline_fmt_gv,
                float_inline_fmt_gv,
                int_scanf_gv,
                float_scanf_gv,
                str_scanf_gv,
                ptr_type,
            )?;
        }

        let ret_val = bcx.ins().iconst(types::I32, 0);
        bcx.ins().return_(&[ret_val]);
    }

    module
        .define_function(main_id, &mut ctx)
        .map_err(|e| ForgeError::codegen(format!("define main: {}", e)))?;

    module.clear_context(&mut ctx);

    let product = module.finish();
    let bytes = product.emit().map_err(|e| ForgeError::codegen(format!("emit object: {}", e)))?;
    std::fs::write(obj_path, bytes).map_err(|e| ForgeError::codegen(format!("write object: {}", e)))?;

    Ok(())
}

fn collect_strings(
    stmt: &Statement,
    string_map: &mut HashMap<String, GlobalValue>,
    module: &mut ObjectModule,
    ctx: &mut cranelift_codegen::Context,
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
                    let id = define_string(module, &name, format!("{}\0", s).as_bytes())?;
                    let gv = module.declare_data_in_func(id, &mut ctx.func);
                    string_map.insert(s.clone(), gv);
                }
                for part in parts {
                    if let StringPart::Literal(text) = part {
                        if !string_map.contains_key(text) {
                            let name = format!("str_lit_{}", string_map.len());
                            let id = define_string(module, &name, format!("{}\0", text).as_bytes())?;
                            let gv = module.declare_data_in_func(id, &mut ctx.func);
                            string_map.insert(text.clone(), gv);
                        }
                    }
                }
            }
        }
        Statement::If(if_node) => {
            for (_, body) in &if_node.branches {
                for s in body { collect_strings(s, string_map, module, ctx)?; }
            }
            if let Some(else_body) = &if_node.else_body {
                for s in else_body { collect_strings(s, string_map, module, ctx)?; }
            }
        }
        Statement::While(while_node) => {
            collect_strings_in_body(&while_node.body, string_map, module, ctx)?;
        }
        _ => {}
    }
    Ok(())
}

fn collect_strings_in_body(
    body: &[Statement],
    string_map: &mut HashMap<String, GlobalValue>,
    module: &mut ObjectModule,
    ctx: &mut cranelift_codegen::Context,
) -> ForgeResult<()> {
    for stmt in body {
        collect_strings(stmt, string_map, module, ctx)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_statement(
    bcx: &mut FunctionBuilder,
    stmt: &Statement,
    var_map: &mut HashMap<String, Variable>,
    float_vars: &mut HashSet<String>,
    string_vars: &mut HashSet<String>,
    string_map: &HashMap<String, GlobalValue>,
    printf_ref: FuncRef,
    puts_ref: FuncRef,
    fputs_ref: FuncRef,
    stdout_gv: GlobalValue,
    scanf_ref: FuncRef,
    pow_ref: FuncRef,
    int_fmt_gv: GlobalValue,
    float_fmt_gv: GlobalValue,
    int_inline_fmt_gv: GlobalValue,
    float_inline_fmt_gv: GlobalValue,
    int_scanf_gv: GlobalValue,
    float_scanf_gv: GlobalValue,
    str_scanf_gv: GlobalValue,
    ptr_type: types::Type,
) -> ForgeResult<()> {
    match stmt {
        Statement::VarDecl(v) => {
            let ty = if type_decl_is_float(&v.type_decl) { types::F64 } else if matches!(v.type_decl, TypeDecl::Weld) { ptr_type } else { types::I32 };
            let var = bcx.declare_var(ty);
            var_map.insert(v.name.clone(), var);
            if ty == types::F64 {
                float_vars.insert(v.name.clone());
            }
            if ty == ptr_type {
                string_vars.insert(v.name.clone());
            }
            
            if let Some(init) = &v.initializer {
                let val = build_expr(bcx, init, var_map, float_vars, string_map, printf_ref, scanf_ref, pow_ref, int_scanf_gv, float_scanf_gv, str_scanf_gv, ptr_type)?;
                bcx.def_var(var, val);
            } else {
                let zero = if ty == types::F64 {
                    bcx.ins().f64const(0.0)
                } else {
                    bcx.ins().iconst(ty, 0).into()
                };
                bcx.def_var(var, zero);
            }
        }
        Statement::Print(p) => {
            if let Expr::Str(parts) = &p.expr {
                if parts.iter().any(|part| matches!(part, StringPart::Interp(_))) {
                    for part in parts {
                        match part {
                            StringPart::Literal(text) => {
                                if let Some(gv) = string_map.get(text) {
                                    let value = bcx.ins().symbol_value(ptr_type, *gv);
                                    let stdout_ptr = bcx.ins().symbol_value(ptr_type, stdout_gv);
                                    let stdout = bcx.ins().load(ptr_type, MachMemFlags::new(), stdout_ptr, 0);
                                    bcx.ins().call(fputs_ref, &[value, stdout]);
                                }
                            }
                            StringPart::Interp(name) => {
                                let value = build_expr(bcx, &Expr::Identifier(name.clone()), var_map, float_vars, string_map, printf_ref, scanf_ref, pow_ref, int_scanf_gv, float_scanf_gv, str_scanf_gv, ptr_type)?;
                                if string_map.contains_key(name) || string_vars.contains(name) {
                                    let stdout_ptr = bcx.ins().symbol_value(ptr_type, stdout_gv);
                                    let stdout = bcx.ins().load(ptr_type, MachMemFlags::new(), stdout_ptr, 0);
                                    bcx.ins().call(fputs_ref, &[value, stdout]);
                                } else {
                                    let fmt = if float_vars.contains(name) { float_inline_fmt_gv } else { int_inline_fmt_gv };
                                    let fmt_value = bcx.ins().symbol_value(ptr_type, fmt);
                                    let numeric_value = if float_vars.contains(name) { value } else { bcx.ins().fcvt_from_sint(types::F64, value) };
                                    bcx.ins().call(printf_ref, &[fmt_value, numeric_value]);
                                }
                            }
                        }
                    }
                    let newline = string_map.get("\n").map(|gv| bcx.ins().symbol_value(ptr_type, *gv));
                    if let Some(value) = newline {
                        let stdout_ptr = bcx.ins().symbol_value(ptr_type, stdout_gv);
                        let stdout = bcx.ins().load(ptr_type, MachMemFlags::new(), stdout_ptr, 0);
                        bcx.ins().call(fputs_ref, &[value, stdout]);
                    }
                    return Ok(());
                }
            }
            let val = build_expr(bcx, &p.expr, var_map, float_vars, string_map, printf_ref, scanf_ref, pow_ref, int_scanf_gv, float_scanf_gv, str_scanf_gv, ptr_type)?;
            let is_string = matches!(&p.expr, Expr::Str(_)) || matches!(&p.expr, Expr::Identifier(name) if string_vars.contains(name));
            let is_float = expr_is_float_inner(&p.expr, float_vars);
            
            if is_string {
                bcx.ins().call(puts_ref, &[val]);
            } else {
                let fmt_gv = if is_float { float_fmt_gv } else { int_fmt_gv };
                let fmt_val = bcx.ins().symbol_value(ptr_type, fmt_gv);
                let numeric_val = if is_float { val } else { bcx.ins().fcvt_from_sint(types::F64, val) };
                bcx.ins().call(printf_ref, &[fmt_val, numeric_val]);
            }
        }
        Statement::Assignment(a) => {
            if a.target_path.len() == 1 {
                let var_name = &a.target_path[0];
                if let Some(var) = var_map.get(var_name) {
                    let val = build_expr(bcx, &a.value, var_map, float_vars, string_map, printf_ref, scanf_ref, pow_ref, int_scanf_gv, float_scanf_gv, str_scanf_gv, ptr_type)?;
                    bcx.def_var(*var, val);
                } else {
                    return Err(ForgeError::codegen(format!("Undefined variable: {}", var_name)));
                }
            } else {
                return Err(ForgeError::codegen("Member assignment not supported yet"));
            }
        }
        Statement::If(if_node) => {
            let merge_block = bcx.create_block();
            
            for (cond, body) in &if_node.branches {
                let cond_val = build_expr(bcx, cond, var_map, float_vars, string_map, printf_ref, scanf_ref, pow_ref, int_scanf_gv, float_scanf_gv, str_scanf_gv, ptr_type)?;
                let cond_bool = bcx.ins().icmp_imm_u(IntCC::NotEqual, cond_val, 0);
                
                let then_block = bcx.create_block();
                let next_cond_block = bcx.create_block();
                
                bcx.ins().brif(cond_bool, then_block, &[], next_cond_block, &[]);
                
                bcx.switch_to_block(then_block);
                bcx.seal_block(then_block);
                
                for s in body {
                    build_statement(bcx, s, var_map, float_vars, string_vars, string_map, printf_ref, puts_ref, fputs_ref, stdout_gv, scanf_ref, pow_ref, int_fmt_gv, float_fmt_gv, int_inline_fmt_gv, float_inline_fmt_gv, int_scanf_gv, float_scanf_gv, str_scanf_gv, ptr_type)?;
                }
                bcx.ins().jump(merge_block, &[]);
                
                bcx.switch_to_block(next_cond_block);
                bcx.seal_block(next_cond_block);
            }
            
            if let Some(else_body) = &if_node.else_body {
                for s in else_body {
                    build_statement(bcx, s, var_map, float_vars, string_vars, string_map, printf_ref, puts_ref, fputs_ref, stdout_gv, scanf_ref, pow_ref, int_fmt_gv, float_fmt_gv, int_inline_fmt_gv, float_inline_fmt_gv, int_scanf_gv, float_scanf_gv, str_scanf_gv, ptr_type)?;
                }
            }
            bcx.ins().jump(merge_block, &[]);
            
            bcx.switch_to_block(merge_block);
            bcx.seal_block(merge_block);
        }
        Statement::While(while_node) => {
            let header_block = bcx.create_block();
            let body_block = bcx.create_block();
            let exit_block = bcx.create_block();
            bcx.ins().jump(header_block, &[]);

            bcx.switch_to_block(header_block);
            let condition = build_expr(bcx, &while_node.condition, var_map, float_vars, string_map, printf_ref, scanf_ref, pow_ref, int_scanf_gv, float_scanf_gv, str_scanf_gv, ptr_type)?;
            let condition = bcx.ins().icmp_imm_u(IntCC::NotEqual, condition, 0);
            bcx.ins().brif(condition, body_block, &[], exit_block, &[]);

            bcx.switch_to_block(body_block);
            for s in &while_node.body {
                build_statement(bcx, s, var_map, float_vars, string_vars, string_map, printf_ref, puts_ref, fputs_ref, stdout_gv, scanf_ref, pow_ref, int_fmt_gv, float_fmt_gv, int_inline_fmt_gv, float_inline_fmt_gv, int_scanf_gv, float_scanf_gv, str_scanf_gv, ptr_type)?;
            }
            bcx.ins().jump(header_block, &[]);
            bcx.seal_block(body_block);
            bcx.seal_block(header_block);

            bcx.switch_to_block(exit_block);
            bcx.seal_block(exit_block);
        }
        Statement::DataDecl(_) | Statement::ObjectDecl(_) | Statement::Use(_) | Statement::ExprStmt(_) => {
            // Ignore for now, validly parsed but no codegen action needed
        }
        _ => return Err(ForgeError::codegen(format!("Unsupported statement: {:?}", stmt))),
    }
    Ok(())
}

fn define_string(module: &mut ObjectModule, name: &str, bytes: &[u8]) -> ForgeResult<DataId> {
    let mut data_ctx = DataDescription::new();
    data_ctx.define(bytes.to_vec().into_boxed_slice());
    let id = module.declare_data(name, Linkage::Export, false, false)
        .map_err(|e| ForgeError::codegen(format!("declare_data: {}", e)))?;
    module.define_data(id, &data_ctx)
        .map_err(|e| ForgeError::codegen(format!("define_data: {}", e)))?;
    Ok(id)
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
            _ => expanded.push(statement.clone()),
        }
    }
    Ok(expanded)
}

fn type_decl_is_float(type_decl: &TypeDecl) -> bool {
    matches!(type_decl, TypeDecl::Number(Subtype::Float))
}

fn expr_is_float_inner(expr: &Expr, float_vars: &HashSet<String>) -> bool {
    match expr {
        Expr::Number(n) => n.is_float,
        Expr::Identifier(name) => float_vars.contains(name),
        Expr::BinaryOp { lhs, rhs, .. } => expr_is_float_inner(lhs, float_vars) || expr_is_float_inner(rhs, float_vars),
        Expr::UnaryOp { operand, .. } => expr_is_float_inner(operand, float_vars),
        Expr::Input(InputNode { subtype: Some(Subtype::Float) }) => true,
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_expr(
    bcx: &mut FunctionBuilder,
    expr: &Expr,
    var_map: &HashMap<String, Variable>,
    float_vars: &HashSet<String>,
    string_map: &HashMap<String, GlobalValue>,
    printf_ref: FuncRef,
    scanf_ref: FuncRef,
    pow_ref: FuncRef,
    int_scanf_gv: GlobalValue,
    float_scanf_gv: GlobalValue,
    str_scanf_gv: GlobalValue,
    ptr_type: types::Type,
) -> Result<Value, String> {
    match expr {
        Expr::Number(n) => {
            if n.is_float {
                Ok(bcx.ins().f64const(n.float_val))
            } else {
                Ok(bcx.ins().iconst(types::I32, n.int_val))
            }
        }
        Expr::Identifier(name) => {
            if let Some(var) = var_map.get(name) {
                Ok(bcx.use_var(*var))
            } else if let Some(gv) = string_map.get(name) {
                Ok(bcx.ins().symbol_value(ptr_type, *gv))
            } else {
                Err(format!("Undefined variable: {}", name))
            }
        }
        Expr::BinaryOp { op, lhs, rhs } => {
            let l = build_expr(bcx, lhs, var_map, float_vars, string_map, printf_ref, scanf_ref, pow_ref, int_scanf_gv, float_scanf_gv, str_scanf_gv, ptr_type)?;
            let r = build_expr(bcx, rhs, var_map, float_vars, string_map, printf_ref, scanf_ref, pow_ref, int_scanf_gv, float_scanf_gv, str_scanf_gv, ptr_type)?;
            
            let is_float = expr_is_float_inner(lhs, float_vars) || expr_is_float_inner(rhs, float_vars);
            if is_float {
                let l_f = if expr_is_float_inner(lhs, float_vars) { l } else { bcx.ins().fcvt_from_sint(types::F64, l) };
                let r_f = if expr_is_float_inner(rhs, float_vars) { r } else { bcx.ins().fcvt_from_sint(types::F64, r) };
                match op {
                    BinOp::Add => Ok(bcx.ins().fadd(l_f, r_f)),
                    BinOp::Sub => Ok(bcx.ins().fsub(l_f, r_f)),
                    BinOp::Mul => Ok(bcx.ins().fmul(l_f, r_f)),
                    BinOp::Div => Ok(bcx.ins().fdiv(l_f, r_f)),
                    BinOp::Pow => {
                        let pow_inst = bcx.ins().call(pow_ref, &[l_f, r_f]);
                        Ok(bcx.inst_results(pow_inst)[0])
                    }
                    _ => Err(format!("Unsupported float op: {:?}", op)),
                }
            } else {
                match op {
                    BinOp::Add => Ok(bcx.ins().iadd(l, r)),
                    BinOp::Sub => Ok(bcx.ins().isub(l, r)),
                    BinOp::Mul => Ok(bcx.ins().imul(l, r)),
                    BinOp::Div => Ok(bcx.ins().sdiv(l, r)),
                    BinOp::Pow => {
                        let l_f = bcx.ins().fcvt_from_sint(types::F64, l);
                        let r_f = bcx.ins().fcvt_from_sint(types::F64, r);
                        let pow_inst = bcx.ins().call(pow_ref, &[l_f, r_f]);
                        let res_f = bcx.inst_results(pow_inst)[0];
                        Ok(bcx.ins().fcvt_to_sint(types::I32, res_f))
                    }
                    BinOp::Eq => Ok(bcx.ins().icmp(IntCC::Equal, l, r)),
                    BinOp::Ne => Ok(bcx.ins().icmp(IntCC::NotEqual, l, r)),
                    BinOp::Lt => Ok(bcx.ins().icmp(IntCC::SignedLessThan, l, r)),
                    BinOp::Gt => Ok(bcx.ins().icmp(IntCC::SignedGreaterThan, l, r)),
                    BinOp::Le => Ok(bcx.ins().icmp(IntCC::SignedLessThanOrEqual, l, r)),
                    BinOp::Ge => Ok(bcx.ins().icmp(IntCC::SignedGreaterThanOrEqual, l, r)),
                    BinOp::And => Ok(bcx.ins().band(l, r)),
                    BinOp::Or => Ok(bcx.ins().bor(l, r)),
                    BinOp::Xor => Ok(bcx.ins().bxor(l, r)),
                    _ => Err(format!("Unsupported int op: {:?}", op)),
                }
            }
        }
        Expr::UnaryOp { op, operand } => {
            let val = build_expr(bcx, operand, var_map, float_vars, string_map, printf_ref, scanf_ref, pow_ref, int_scanf_gv, float_scanf_gv, str_scanf_gv, ptr_type)?;
            match op {
                UnOp::Plus => Ok(val),
                UnOp::Neg => {
                    if expr_is_float_inner(operand, float_vars) {
                        Ok(bcx.ins().fneg(val))
                    } else {
                        Ok(bcx.ins().ineg(val))
                    }
                }
            }
        }
        Expr::Input(input_node) => {
            let (fmt_gv, ty) = match input_node.subtype {
                Some(Subtype::Int) => (int_scanf_gv, types::I32),
                Some(Subtype::Float) => (float_scanf_gv, types::F64),
                _ => {
                    // String input
                    let slot_data = StackSlotData::new(StackSlotKind::ExplicitSlot, 256, 0);
                    let slot = bcx.create_sized_stack_slot(slot_data);
                    let slot_ptr = bcx.ins().stack_addr(ptr_type, slot, 0);
                    let fmt_val = bcx.ins().symbol_value(ptr_type, str_scanf_gv);
                    bcx.ins().call(scanf_ref, &[fmt_val, slot_ptr]);
                    return Ok(slot_ptr);
                }
            };
            
            let slot_data = StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 0);
            let slot = bcx.create_sized_stack_slot(slot_data);
            
            let slot_ptr = bcx.ins().stack_addr(ptr_type, slot, 0);
            let fmt_val = bcx.ins().symbol_value(ptr_type, fmt_gv);
            
            bcx.ins().call(scanf_ref, &[fmt_val, slot_ptr]);
            Ok(bcx.ins().stack_load(ptr_type, ty, slot, 0))
        }
        Expr::Str(parts) => {
            let mut s = String::new();
            for part in parts {
                if let StringPart::Literal(l) = part {
                    s.push_str(l);
                }
            }
            if let Some(gv) = string_map.get(&s) {
                Ok(bcx.ins().symbol_value(ptr_type, *gv))
            } else {
                Err(format!("Undefined string literal: {}", s))
            }
        }
        _ => Err(format!("Unsupported expr: {:?}", expr)),
    }
}