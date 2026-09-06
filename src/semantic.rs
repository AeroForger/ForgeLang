use rayon::prelude::*;
use std::collections::HashMap;

use crate::ast::{
    AssignmentTarget, BinOp, Expr, ForNode, FunctionDecl, Param, Program, RetKind, Statement,
    Subtype, TypeDecl,
};
use crate::errors::{ForgeError, ForgeResult};

#[derive(Debug, Clone, Copy)]
struct ValidationContext<'a> {
    current_function: Option<&'a str>,
    in_loop_or_if: bool,
    in_loop: bool,
}

#[derive(Clone)]
struct Scope {
    var_types: HashMap<String, TypeDecl>,
}

impl Scope {
    fn new() -> Self {
        Self {
            var_types: HashMap::new(),
        }
    }

    fn from_params(params: &[Param]) -> Self {
        let var_types = params
            .iter()
            .map(|p| (p.name.clone(), p.type_decl.clone()))
            .collect();
        Self { var_types }
    }

    fn get(&self, name: &str) -> Option<&TypeDecl> {
        self.var_types.get(name)
    }

    fn contains_key(&self, name: &str) -> bool {
        self.var_types.contains_key(name)
    }
}

fn collect_var_types(statements: &[Statement], scope: &mut Scope) {
    for stmt in statements {
        match stmt {
            Statement::VarDecl(decl) => {
                scope
                    .var_types
                    .insert(decl.name.clone(), decl.type_decl.clone());
            }
            Statement::If(node) => {
                for (_, body) in &node.branches {
                    collect_var_types(body, scope);
                }
                if let Some(else_body) = &node.else_body {
                    collect_var_types(else_body, scope);
                }
            }
            Statement::While(node) => {
                collect_var_types(&node.body, scope);
            }
            Statement::For(node) => {
                collect_var_types(&node.body, scope);
            }
            Statement::FunctionDecl(func) => {
                collect_var_types(&func.body, scope);
            }
            _ => {}
        }
    }
}

fn expr_is_bool(expr: &Expr, scope: &Scope) -> bool {
    match expr {
        Expr::Bool(_) => true,
        Expr::Identifier(name) => matches!(scope.get(name), Some(TypeDecl::Bool)),
        _ => false,
    }
}

fn expr_type_name(expr: &Expr) -> &'static str {
    match expr {
        Expr::Bool(_) => "Bool",
        Expr::Number(n) if n.is_float => "Float",
        Expr::Number(_) => "Int",
        Expr::Str(_) => "Weld",
        Expr::ArrayLiteral(_) => "Array",
        Expr::TupleLiteral(_) => "Tuple",
        Expr::ListLiteral(_) => "List",
        Expr::Input(_) => "Input",
        _ => "value",
    }
}

fn resolved_expr_type_name(expr: &Expr, scope: &Scope) -> &'static str {
    match expr {
        Expr::Identifier(name) => match scope.get(name) {
            Some(TypeDecl::Bool) => "Bool",
            Some(TypeDecl::Number(Subtype::Int)) => "Int",
            Some(TypeDecl::Number(Subtype::Float)) => "Float",
            Some(TypeDecl::Weld) => "Weld",
            _ => expr_type_name(expr),
        },
        _ => expr_type_name(expr),
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ExprType {
    Number(Subtype),
    Weld,
    Bool,
    Array,
    Tuple,
    List,
    Unknown,
}

fn type_decl_to_expr_type(type_decl: &TypeDecl) -> ExprType {
    match type_decl {
        TypeDecl::Number(subtype) => ExprType::Number(subtype.clone()),
        TypeDecl::Weld => ExprType::Weld,
        TypeDecl::Bool => ExprType::Bool,
        TypeDecl::Ore(_) => ExprType::Array,
        TypeDecl::OreTuple(_) => ExprType::Tuple,
        TypeDecl::Materials(_, _) => ExprType::List,
    }
}

fn ret_kind_to_expr_type(ret_kind: &RetKind) -> ExprType {
    match ret_kind {
        RetKind::Int => ExprType::Number(Subtype::Int),
        RetKind::Float => ExprType::Number(Subtype::Float),
        RetKind::Weld => ExprType::Weld,
        RetKind::Bool => ExprType::Bool,
        RetKind::Ore(_) => ExprType::Array,
        RetKind::OreTuple(_) => ExprType::Tuple,
        RetKind::Materials(_, _) => ExprType::List,
        RetKind::Generic | RetKind::Function | RetKind::Dynamic => ExprType::Unknown,
        RetKind::Nunction | RetKind::Void => ExprType::Unknown,
    }
}

fn expr_type(expr: &Expr, scope: &Scope, program: &Program) -> ExprType {
    match expr {
        Expr::Number(number) => ExprType::Number(if number.is_float {
            Subtype::Float
        } else {
            Subtype::Int
        }),
        Expr::Str(_) => ExprType::Weld,
        Expr::Bool(_) => ExprType::Bool,
        Expr::Identifier(name) => scope
            .get(name)
            .map(type_decl_to_expr_type)
            .unwrap_or(ExprType::Unknown),
        Expr::Input(input) => input
            .subtype
            .clone()
            .map(ExprType::Number)
            .unwrap_or(ExprType::Weld),
        Expr::Call { callee, .. } => program
            .statements
            .iter()
            .find_map(|statement| match statement {
                Statement::FunctionDecl(function) if function.name == *callee => {
                    Some(ret_kind_to_expr_type(&function.ret_kind))
                }
                _ => None,
            })
            .unwrap_or(ExprType::Unknown),
        Expr::BinaryOp { op, lhs, rhs } => match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Pow => {
                match (expr_type(lhs, scope, program), expr_type(rhs, scope, program)) {
                    (ExprType::Number(Subtype::Float), _)
                    | (_, ExprType::Number(Subtype::Float)) => ExprType::Number(Subtype::Float),
                    (ExprType::Number(Subtype::Int), ExprType::Number(Subtype::Int)) => {
                        ExprType::Number(Subtype::Int)
                    }
                    _ => ExprType::Unknown,
                }
            }
            BinOp::Rem => ExprType::Number(Subtype::Int),
            BinOp::Eq
            | BinOp::Ne
            | BinOp::Lt
            | BinOp::Gt
            | BinOp::Le
            | BinOp::Ge
            | BinOp::And
            | BinOp::Or
            | BinOp::Xor => ExprType::Bool,
        },
        Expr::UnaryOp { operand, .. } => expr_type(operand, scope, program),
        Expr::ArrayLiteral(_) => ExprType::Array,
        Expr::TupleLiteral(_) => ExprType::Tuple,
        Expr::ListLiteral(_) => ExprType::List,
        Expr::NamespaceCall { .. }
        | Expr::MethodCall { .. }
        | Expr::MemberAccess { .. }
        | Expr::IndexAccess { .. } => ExprType::Unknown,
    }
}

fn types_compatible(actual: &ExprType, expected: &TypeDecl) -> bool {
    match (actual, expected) {
        (ExprType::Unknown, _) => true,
        (ExprType::Number(actual), TypeDecl::Number(expected)) => {
            actual == expected || *expected == Subtype::Generic
        }
        (ExprType::Weld, TypeDecl::Weld)
        | (ExprType::Bool, TypeDecl::Bool)
        | (ExprType::Array, TypeDecl::Ore(_))
        | (ExprType::Tuple, TypeDecl::OreTuple(_))
        | (ExprType::List, TypeDecl::Materials(_, _)) => true,
        _ => false,
    }
}

fn type_name(type_decl: &TypeDecl) -> &'static str {
    match type_decl {
        TypeDecl::Number(Subtype::Int) => "Int",
        TypeDecl::Number(Subtype::Float) => "Float",
        TypeDecl::Number(Subtype::Generic) => "Generic",
        TypeDecl::Number(Subtype::Weld) => "Weld",
        TypeDecl::Weld => "Weld",
        TypeDecl::Bool => "Bool",
        TypeDecl::Ore(_) => "Array",
        TypeDecl::OreTuple(_) => "Tuple",
        TypeDecl::Materials(_, _) => "List",
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum NumericType {
    Int,
    Float,
}

fn numeric_type(expr: &Expr, scope: &Scope) -> Option<NumericType> {
    match expr {
        Expr::Number(n) => Some(if n.is_float {
            NumericType::Float
        } else {
            NumericType::Int
        }),
        Expr::Identifier(name) => match scope.get(name) {
            Some(TypeDecl::Number(Subtype::Int)) => Some(NumericType::Int),
            Some(TypeDecl::Number(Subtype::Float)) => Some(NumericType::Float),
            _ => None,
        },
        Expr::Input(input) => match input.subtype {
            Some(Subtype::Int) => Some(NumericType::Int),
            Some(Subtype::Float) => Some(NumericType::Float),
            _ => None,
        },
        Expr::BinaryOp { op, lhs, rhs }
            if matches!(
                op,
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Pow
            ) =>
        {
            let lhs_type = numeric_type(lhs, scope)?;
            let rhs_type = numeric_type(rhs, scope)?;
            Some(
                if lhs_type == NumericType::Float || rhs_type == NumericType::Float {
                    NumericType::Float
                } else {
                    NumericType::Int
                },
            )
        }
        Expr::BinaryOp {
            op: BinOp::Rem,
            lhs,
            rhs,
        } => {
            if numeric_type(lhs, scope) == Some(NumericType::Int)
                && numeric_type(rhs, scope) == Some(NumericType::Int)
            {
                Some(NumericType::Int)
            } else {
                None
            }
        }
        Expr::UnaryOp { operand, .. } => numeric_type(operand, scope),
        _ => None,
    }
}

pub fn analyze(program: &Program) -> ForgeResult<()> {
    // Validate loose top-level statements
    let top_level_ctx = ValidationContext {
        current_function: None,
        in_loop_or_if: false,
        in_loop: false,
    };
    let empty_scope = Scope::new();
    for statement in &program.statements {
        if !matches!(statement, Statement::FunctionDecl(_)) {
            validate_statement(statement, &empty_scope, program, top_level_ctx)?;
        }
    }

    let functions: Vec<&FunctionDecl> = program
        .statements
        .par_iter()
        .filter_map(|statement| {
            if let Statement::FunctionDecl(function) = statement {
                Some(function)
            } else {
                None
            }
        })
        .collect();

    functions
        .par_iter()
        .try_for_each(|function| validate_function(function, program))
}

fn validate_function(function: &FunctionDecl, program: &Program) -> ForgeResult<()> {
    if function.name == "Main" && !function.params.is_empty() {
        return Err(ForgeError::parse("Main cannot have parameters"));
    }

    let mut scope = Scope::from_params(&function.params);
    collect_var_types(&function.body, &mut scope);

    let context = ValidationContext {
        current_function: Some(function.name.as_str()),
        in_loop_or_if: false,
        in_loop: false,
    };
    validate_statements(&function.body, &scope, program, context)
}

fn validate_statements(
    statements: &[Statement],
    scope: &Scope,
    program: &Program,
    context: ValidationContext,
) -> ForgeResult<()> {
    for statement in statements {
        validate_statement(statement, scope, program, context)?;
    }
    Ok(())
}

fn validate_statement(
    statement: &Statement,
    scope: &Scope,
    program: &Program,
    context: ValidationContext,
) -> ForgeResult<()> {
    match statement {
        Statement::VarDecl(decl) => {
            validate_var_decl(decl, scope, program, context)?;
        }
        Statement::Assignment(assignment) => {
            validate_assignment_target(&assignment.target, scope, program, context)?;
            validate_expr(&assignment.value, scope, program, context)?;
            validate_assignment_type(&assignment, scope)?;
        }
        Statement::If(node) => {
            let branch_ctx = ValidationContext {
                in_loop_or_if: true,
                ..context
            };
            for (condition, body) in &node.branches {
                validate_expr(condition, scope, program, context)?;
                validate_statements(body, scope, program, branch_ctx)?;
            }
            if let Some(body) = &node.else_body {
                validate_statements(body, scope, program, branch_ctx)?;
            }
        }
        Statement::While(node) => {
            let loop_ctx = ValidationContext {
                in_loop_or_if: true,
                in_loop: true,
                ..context
            };
            validate_expr(&node.condition, scope, program, context)?;
            validate_statements(&node.body, scope, program, loop_ctx)?;
        }
        Statement::For(node) => {
            validate_for_stmt(node, scope, program, context)?;
        }
        Statement::Stop => {
            if context.current_function == Some("Main") {
                return Err(ForgeError::parse("Stop cannot be used in Main"));
            }
            if !context.in_loop_or_if {
                return Err(ForgeError::parse(
                    "Stop can only be used inside a loop or If statement",
                ));
            }
        }
        Statement::Skip => {
            if !context.in_loop {
                return Err(ForgeError::parse("Skip can only be used inside a loop"));
            }
        }
        Statement::Use(import) => {
            let path = import.path.join(".");
            let target = import
                .item
                .as_deref()
                .map_or(path.clone(), |item| format!("{}: {}", path, item));
            return Err(ForgeError::parse(format!(
                "imports are not implemented: {}",
                target
            )));
        }
        Statement::Print(print) => validate_expr(&print.expr, scope, program, context)?,
        Statement::Return(value) => {
            if let Some(value) = value {
                validate_expr(value, scope, program, context)?;
            }
            let function_name = context.current_function.ok_or_else(|| {
                ForgeError::parse("Return can only be used inside a function")
            })?;
            let function = program
                .statements
                .iter()
                .find_map(|statement| match statement {
                    Statement::FunctionDecl(function) if function.name == function_name => {
                        Some(function)
                    }
                    _ => None,
                })
                .ok_or_else(|| ForgeError::parse(format!("Undefined function: {}", function_name)))?;
            match (&function.ret_kind, value) {
                (ret_kind, None) if !ret_kind.is_void() => {
                    return Err(ForgeError::parse(format!(
                        "Function {} must return a value",
                        function_name
                    )));
                }
                (ret_kind, Some(_)) if ret_kind.is_void() => {
                    return Err(ForgeError::parse(format!(
                        "Void function {} cannot return a value",
                        function_name
                    )));
                }
                (ret_kind, Some(value)) => {
                    let expected = match ret_kind {
                        RetKind::Int => Some(TypeDecl::Number(Subtype::Int)),
                        RetKind::Float => Some(TypeDecl::Number(Subtype::Float)),
                        RetKind::Weld => Some(TypeDecl::Weld),
                        RetKind::Bool => Some(TypeDecl::Bool),
                        RetKind::Ore(size) => Some(TypeDecl::Ore(*size)),
                        RetKind::OreTuple(fields) => Some(TypeDecl::OreTuple(fields.clone())),
                        RetKind::Materials(element, has_new) => {
                            Some(TypeDecl::Materials(element.clone(), *has_new))
                        }
                        RetKind::Generic | RetKind::Function | RetKind::Dynamic => None,
                        RetKind::Nunction | RetKind::Void => unreachable!(),
                    };
                    if let Some(expected) = expected {
                        let actual = expr_type(value, scope, program);
                        if !types_compatible(&actual, &expected) {
                            return Err(ForgeError::parse(format!(
                                "Return type mismatch in {}: expected {}, got {}",
                                function_name,
                                type_name(&expected),
                                resolved_expr_type_name(value, scope)
                            )));
                        }
                    }
                }
                _ => {}
            }
        }
        Statement::ExprStmt(expr) => validate_expr(expr, scope, program, context)?,
        _ => {}
    }
    Ok(())
}

fn validate_for_stmt(
    node: &ForNode,
    scope: &Scope,
    program: &Program,
    context: ValidationContext,
) -> ForgeResult<()> {
    let loop_ctx = ValidationContext {
        in_loop_or_if: true,
        in_loop: true,
        ..context
    };
    let init_scope = scope.clone();
    if let Some(init_value) = &node.init.initializer {
        validate_expr(init_value, &init_scope, program, context)?;
    }

    let mut loop_scope = init_scope;
    loop_scope
        .var_types
        .insert(node.init.name.clone(), node.init.type_decl.clone());
    validate_expr(&node.condition, &loop_scope, program, context)?;
    validate_statements(&node.body, &loop_scope, program, loop_ctx)?;

    match loop_scope.get(&node.increment_var) {
        None => Err(ForgeError::parse(format!(
            "Undefined variable: {}",
            node.increment_var
        ))),
        Some(TypeDecl::Number(Subtype::Int | Subtype::Float)) => Ok(()),
        Some(_) => Err(ForgeError::parse(format!(
            "For loop increment variable '{}' must be numeric",
            node.increment_var
        ))),
    }
}

fn validate_var_decl(
    decl: &crate::ast::VarDecl,
    scope: &Scope,
    program: &Program,
    context: ValidationContext,
) -> ForgeResult<()> {
    match &decl.type_decl {
        TypeDecl::Bool => {
            if let Some(init) = &decl.initializer {
                if !expr_is_bool(init, scope) {
                    return Err(ForgeError::parse(format!(
                        "Type error: cannot assign {} to Bool variable '{}'",
                        expr_type_name(init),
                        decl.name
                    )));
                }
            }
        }
        TypeDecl::Ore(Some(expected_size)) => {
            if let Some(init) = &decl.initializer {
                if let Expr::ArrayLiteral(elements) = init {
                    if elements.len() as i64 != *expected_size {
                        return Err(ForgeError::parse(format!(
                            "Array size mismatch: expected {} elements, got {}",
                            expected_size,
                            elements.len()
                        )));
                    }
                }
            }
        }
        TypeDecl::OreTuple(fields) => {
            if let Some(init) = &decl.initializer {
                if let Expr::TupleLiteral(elements) = init {
                    if elements.len() != fields.len() {
                        return Err(ForgeError::parse(format!(
                            "Tuple field count mismatch: expected {} fields, got {}",
                            fields.len(),
                            elements.len()
                        )));
                    }
                    for (elem, (field_type, field_name)) in elements.iter().zip(fields.iter()) {
                        if !expr_matches_subtype(elem, field_type) {
                            return Err(ForgeError::parse(format!(
                                "Type mismatch for tuple field '{}': expected {:?}",
                                field_name, field_type
                            )));
                        }
                    }
                }
            }
        }
        TypeDecl::Materials(elem_type, has_new) => {
            if *has_new && decl.initializer.is_some() {
                return Err(ForgeError::parse(
                    "Empty list declared with 'new' cannot have an initializer",
                ));
            }
            if !has_new {
                if let Some(Expr::ListLiteral(elements)) = &decl.initializer {
                    for elem in elements {
                        if !expr_matches_subtype(elem, elem_type) {
                            return Err(ForgeError::parse(format!(
                                "Type mismatch in list initializer: expected element of type {:?}",
                                elem_type
                            )));
                        }
                    }
                }
            }
        }
        _ => {}
    }

    if let Some(initializer) = &decl.initializer {
        validate_expr(initializer, scope, program, context)?;
    }
    Ok(())
}

fn expr_matches_subtype(expr: &Expr, expected: &Subtype) -> bool {
    match expected {
        Subtype::Int => {
            matches!(expr, Expr::Number(n) if !n.is_float)
                || matches!(
                    expr,
                    Expr::BinaryOp { .. } | Expr::UnaryOp { .. } | Expr::Identifier(_)
                )
        }
        Subtype::Float => {
            matches!(expr, Expr::Number(n) if n.is_float)
                || matches!(
                    expr,
                    Expr::BinaryOp { .. } | Expr::UnaryOp { .. } | Expr::Identifier(_)
                )
        }
        Subtype::Weld => matches!(expr, Expr::Str(_)) || matches!(expr, Expr::Identifier(_)),
        Subtype::Generic => true,
    }
}

fn validate_assignment_target(
    target: &AssignmentTarget,
    scope: &Scope,
    program: &Program,
    context: ValidationContext,
) -> ForgeResult<()> {
    match target {
        AssignmentTarget::Var(_) => Ok(()),
        AssignmentTarget::Member { object, .. } => {
            validate_assignment_target(object, scope, program, context)
        }
        AssignmentTarget::Index { object, index } => {
            validate_assignment_target(object, scope, program, context)?;
            validate_expr(index, scope, program, context)
        }
    }
}

fn validate_assignment_type(
    assignment: &crate::ast::AssignmentNode,
    scope: &Scope,
) -> ForgeResult<()> {
    if let AssignmentTarget::Var(name) = &assignment.target {
        if let Some(TypeDecl::Bool) = scope.get(name) {
            if !expr_is_bool(&assignment.value, scope) {
                return Err(ForgeError::parse(format!(
                    "Type error: cannot assign {} to Bool variable '{}'",
                    expr_type_name(&assignment.value),
                    name
                )));
            }
        }
    }
    Ok(())
}

fn validate_expr(
    expr: &Expr,
    scope: &Scope,
    program: &Program,
    context: ValidationContext,
) -> ForgeResult<()> {
    match expr {
        Expr::Call { callee, args } => {
            let function = program
                .statements
                .iter()
                .find_map(|statement| match statement {
                    Statement::FunctionDecl(function) if function.name == *callee => Some(function),
                    _ => None,
                })
                .ok_or_else(|| ForgeError::parse(format!("Undefined function: {}", callee)))?;
            if function.params.len() != args.len() {
                return Err(ForgeError::parse(format!(
                    "Function {} expects {} arguments, got {}",
                    callee,
                    function.params.len(),
                    args.len()
                )));
            }
            for (index, (arg, param)) in args.iter().zip(&function.params).enumerate() {
                validate_expr(arg, scope, program, context)?;
                let actual = expr_type(arg, scope, program);
                if !types_compatible(&actual, &param.type_decl) {
                    return Err(ForgeError::parse(format!(
                        "Argument {} of {}: expected {}, got {}",
                        index + 1,
                        callee,
                        type_name(&param.type_decl),
                        resolved_expr_type_name(arg, scope)
                    )));
                }
            }
        }
        Expr::NamespaceCall {
            namespace,
            method,
            args,
        } => {
            if namespace == "Program" {
                if context.current_function.is_none() {
                    return Err(ForgeError::parse(
                        "Program.Stop() can only be used inside a function scope",
                    ));
                }
                if method == "Stop" {
                    if !args.is_empty() {
                        return Err(ForgeError::parse("Program.Stop() expects 0 arguments"));
                    }
                } else {
                    return Err(ForgeError::parse(format!(
                        "Unknown method {} in Program namespace",
                        method
                    )));
                }
            } else {
                return Err(ForgeError::parse(format!(
                    "Unknown namespace: {}",
                    namespace
                )));
            }
            for arg in args {
                validate_expr(arg, scope, program, context)?;
            }
        }
        Expr::MethodCall {
            object,
            method,
            args,
        } => {
            validate_expr(object, scope, program, context)?;
            for arg in args {
                validate_expr(arg, scope, program, context)?;
            }
            match method.as_str() {
                "Add" | "Remove" | "RemoveAt" | "Length" | "Len" => Ok(()),
                other => Err(ForgeError::parse(format!("Unknown method: {}", other))),
            }?;
        }
        Expr::IndexAccess { object, index } => {
            validate_expr(object, scope, program, context)?;
            validate_expr(index, scope, program, context)?;
        }
        Expr::ArrayLiteral(elements)
        | Expr::TupleLiteral(elements)
        | Expr::ListLiteral(elements) => {
            for elem in elements {
                validate_expr(elem, scope, program, context)?;
            }
        }
        Expr::BinaryOp { lhs, rhs, op } => {
            validate_expr(lhs, scope, program, context)?;
            validate_expr(rhs, scope, program, context)?;

            if matches!(
                op,
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem | BinOp::Pow
            ) && numeric_type(expr, scope).is_none()
            {
                let invalid_operand = if numeric_type(lhs, scope).is_none() {
                    lhs
                } else {
                    rhs
                };
                if matches!(op, BinOp::Rem) {
                    if resolved_expr_type_name(invalid_operand, scope) == "Float" {
                        return Err(ForgeError::parse(
                            "Modulo (%) is only supported for integer types",
                        ));
                    }
                    return Err(ForgeError::parse(format!(
                        "Modulo operator (%) requires integer operands, got {}",
                        resolved_expr_type_name(invalid_operand, scope)
                    )));
                }
                return Err(ForgeError::parse(format!(
                    "Invalid operand type for arithmetic operation: expected Int or Float, got {}",
                    resolved_expr_type_name(invalid_operand, scope)
                )));
            }
        }
        Expr::UnaryOp { operand, .. } => {
            validate_expr(operand, scope, program, context)?;
            if numeric_type(operand, scope).is_none() {
                return Err(ForgeError::parse(format!(
                    "Invalid operand type for unary operation: expected Int or Float, got {}",
                    expr_type_name(operand)
                )));
            }
        }
        Expr::MemberAccess { object, .. } => validate_expr(object, scope, program, context)?,
        Expr::Identifier(name) if name.contains('.') && !scope.contains_key(name) => {
            return Err(ForgeError::parse(format!(
                "shared member access is not allowed: {}",
                name
            )));
        }
        _ => {}
    }
    Ok(())
}
