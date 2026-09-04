use rayon::prelude::*;
use std::collections::HashMap;

use crate::ast::{AssignmentTarget, Expr, FunctionDecl, ForNode, Param, Program, Statement, Subtype, TypeDecl};
use crate::errors::{ForgeError, ForgeResult};

#[derive(Debug, Clone, Copy)]
struct ValidationContext<'a> {
	current_function: Option<&'a str>,
	in_loop_or_if: bool,
}

struct Scope {
	var_types: HashMap<String, TypeDecl>,
}

impl Scope {
	fn new() -> Self {
		Self { var_types: HashMap::new() }
	}

	fn from_params(params: &[Param]) -> Self {
		let var_types = params.iter()
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
				scope.var_types.insert(decl.name.clone(), decl.type_decl.clone());
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
				scope.var_types.insert(node.init.name.clone(), node.init.type_decl.clone());
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

pub fn analyze(program: &Program) -> ForgeResult<()> {
	// Validate loose top-level statements
	let top_level_ctx = ValidationContext {
		current_function: None,
		in_loop_or_if: false,
	};
	let empty_scope = Scope::new();
	for statement in &program.statements {
		if !matches!(statement, Statement::FunctionDecl(_)) {
			validate_statement(statement, &empty_scope, program, top_level_ctx)?;
		}
	}

	let functions: Vec<&FunctionDecl> = program.statements.par_iter().filter_map(|statement| {
		if let Statement::FunctionDecl(function) = statement { Some(function) } else { None }
	}).collect();

	functions.par_iter().try_for_each(|function| validate_function(function, program))
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
				return Err(ForgeError::parse("Stop can only be used inside a loop or If statement"));
			}
		}
		Statement::Print(print) => validate_expr(&print.expr, scope, program, context)?,
		Statement::Return(value) => {
			if let Some(value) = value {
				validate_expr(value, scope, program, context)?;
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
	let loop_ctx = ValidationContext { in_loop_or_if: true, ..context };
	if let Some(init_value) = &node.init.initializer {
		validate_expr(init_value, scope, program, context)?;
	}
	validate_expr(&node.condition, scope, program, context)?;
	validate_statements(&node.body, scope, program, loop_ctx)
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
						expr_type_name(init), decl.name
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
				return Err(ForgeError::parse("Empty list declared with 'new' cannot have an initializer"));
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
		Subtype::Int => matches!(expr, Expr::Number(n) if !n.is_float) || matches!(expr, Expr::BinaryOp { .. } | Expr::UnaryOp { .. } | Expr::Identifier(_)),
		Subtype::Float => matches!(expr, Expr::Number(n) if n.is_float) || matches!(expr, Expr::BinaryOp { .. } | Expr::UnaryOp { .. } | Expr::Identifier(_)),
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
					expr_type_name(&assignment.value), name
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
			let function = program.statements.iter().find_map(|statement| match statement {
				Statement::FunctionDecl(function) if function.name == *callee => Some(function),
				_ => None,
			}).ok_or_else(|| ForgeError::parse(format!("Undefined function: {}", callee)))?;
			if function.params.len() != args.len() {
				return Err(ForgeError::parse(format!("Function {} expects {} arguments, got {}", callee, function.params.len(), args.len())));
			}
			for arg in args {
				validate_expr(arg, scope, program, context)?;
			}
		}
		Expr::NamespaceCall { namespace, method, args } => {
			if namespace == "Program" {
				if context.current_function.is_none() {
					return Err(ForgeError::parse("Program.Stop() can only be used inside a function scope"));
				}
				if method == "Stop" {
					if !args.is_empty() {
						return Err(ForgeError::parse("Program.Stop() expects 0 arguments"));
					}
				} else {
					return Err(ForgeError::parse(format!("Unknown method {} in Program namespace", method)));
				}
			} else {
				return Err(ForgeError::parse(format!("Unknown namespace: {}", namespace)));
			}
			for arg in args {
				validate_expr(arg, scope, program, context)?;
			}
		}
		Expr::MethodCall { object, method, args } => {
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
		Expr::ArrayLiteral(elements) | Expr::TupleLiteral(elements) | Expr::ListLiteral(elements) => {
			for elem in elements {
				validate_expr(elem, scope, program, context)?;
			}
		}
		Expr::BinaryOp { lhs, rhs, .. } => {
			validate_expr(lhs, scope, program, context)?;
			validate_expr(rhs, scope, program, context)?;
		}
		Expr::UnaryOp { operand, .. } => validate_expr(operand, scope, program, context)?,
		Expr::MemberAccess { object, .. } => validate_expr(object, scope, program, context)?,
		Expr::Identifier(name) if name.contains('.') && !scope.contains_key(name) => {
			return Err(ForgeError::parse(format!("shared member access is not allowed: {}", name)));
		}
		_ => {}
	}
	Ok(())
}