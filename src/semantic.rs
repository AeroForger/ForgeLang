use rayon::prelude::*;

use crate::ast::{AssignmentTarget, Expr, FunctionDecl, ForNode, Program, Statement, Subtype, TypeDecl};
use crate::errors::{ForgeError, ForgeResult};

#[derive(Debug, Clone, Copy)]
struct ValidationContext<'a> {
	current_function: Option<&'a str>,
	in_loop_or_if: bool,
}

pub fn analyze(program: &Program) -> ForgeResult<()> {
	// Validate loose top-level statements
	let top_level_ctx = ValidationContext {
		current_function: None,
		in_loop_or_if: false,
	};
	let empty_params = std::collections::HashSet::new();
	for statement in &program.statements {
		if !matches!(statement, Statement::FunctionDecl(_)) {
			validate_statement(statement, &empty_params, program, top_level_ctx)?;
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

	let names: std::collections::HashSet<&str> = function.params.iter().map(|param| param.name.as_str()).collect();
	let context = ValidationContext {
		current_function: Some(function.name.as_str()),
		in_loop_or_if: false,
	};
	validate_statements(&function.body, &names, program, context)
}

fn validate_statements(
	statements: &[Statement],
	parameters: &std::collections::HashSet<&str>,
	program: &Program,
	context: ValidationContext,
) -> ForgeResult<()> {
	for statement in statements {
		validate_statement(statement, parameters, program, context)?;
	}
	Ok(())
}

fn validate_statement(
	statement: &Statement,
	parameters: &std::collections::HashSet<&str>,
	program: &Program,
	context: ValidationContext,
) -> ForgeResult<()> {
	match statement {
		Statement::VarDecl(decl) => {
			validate_var_decl(decl, parameters, program, context)?;
		}
		Statement::Assignment(assignment) => {
			validate_assignment_target(&assignment.target, parameters, program, context)?;
			validate_expr(&assignment.value, parameters, program, context)?;
		}
		Statement::If(node) => {
			let branch_ctx = ValidationContext {
				in_loop_or_if: true,
				..context
			};
			for (condition, body) in &node.branches {
				validate_expr(condition, parameters, program, context)?;
				validate_statements(body, parameters, program, branch_ctx)?;
			}
			if let Some(body) = &node.else_body {
				validate_statements(body, parameters, program, branch_ctx)?;
			}
		}
		Statement::While(node) => {
			let loop_ctx = ValidationContext {
				in_loop_or_if: true,
				..context
			};
			validate_expr(&node.condition, parameters, program, context)?;
			validate_statements(&node.body, parameters, program, loop_ctx)?;
		}
        Statement::For(node) => {
            validate_for_stmt(node, parameters, program, context)?;
        }
        Statement::Stop => {
			if context.current_function == Some("Main") {
				return Err(ForgeError::parse("Stop cannot be used in Main"));
			}
			if !context.in_loop_or_if {
				return Err(ForgeError::parse("Stop can only be used inside a loop or If statement"));
			}
		}
		Statement::Print(print) => validate_expr(&print.expr, parameters, program, context)?,
		Statement::Return(value) => {
			if let Some(value) = value {
				validate_expr(value, parameters, program, context)?;
			}
		}
		Statement::ExprStmt(expr) => validate_expr(expr, parameters, program, context)?,
		_ => {}
	}
	Ok(())
}

fn validate_for_stmt(
	node: &ForNode,
	parameters: &std::collections::HashSet<&str>,
	program: &Program,
	context: ValidationContext,
) -> ForgeResult<()> {
	let loop_ctx = ValidationContext { in_loop_or_if: true, ..context };
	if let Some(init_value) = &node.init.initializer {
		validate_expr(init_value, parameters, program, context)?;
	}
	validate_expr(&node.condition, parameters, program, context)?;
	validate_statements(&node.body, parameters, program, loop_ctx)
}

fn validate_var_decl(
	decl: &crate::ast::VarDecl,
	parameters: &std::collections::HashSet<&str>,
	program: &Program,
	context: ValidationContext,
) -> ForgeResult<()> {
	match &decl.type_decl {
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
		validate_expr(initializer, parameters, program, context)?;
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
	parameters: &std::collections::HashSet<&str>,
	program: &Program,
	context: ValidationContext,
) -> ForgeResult<()> {
	match target {
		AssignmentTarget::Var(_) => Ok(()),
		AssignmentTarget::Member { object, .. } => {
			validate_assignment_target(object, parameters, program, context)
		}
		AssignmentTarget::Index { object, index } => {
			validate_assignment_target(object, parameters, program, context)?;
			validate_expr(index, parameters, program, context)
		}
	}
}

fn validate_expr(
	expr: &Expr,
	parameters: &std::collections::HashSet<&str>,
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
				validate_expr(arg, parameters, program, context)?;
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
				validate_expr(arg, parameters, program, context)?;
			}
		}
		Expr::MethodCall { object, method, args } => {
			validate_expr(object, parameters, program, context)?;
			for arg in args {
				validate_expr(arg, parameters, program, context)?;
			}
			match method.as_str() {
				"Add" | "Remove" | "RemoveAt" | "Length" | "Len" => Ok(()),
				other => Err(ForgeError::parse(format!("Unknown method: {}", other))),
			}?;
		}
		Expr::IndexAccess { object, index } => {
			validate_expr(object, parameters, program, context)?;
			validate_expr(index, parameters, program, context)?;
		}
		Expr::ArrayLiteral(elements) | Expr::TupleLiteral(elements) | Expr::ListLiteral(elements) => {
			for elem in elements {
				validate_expr(elem, parameters, program, context)?;
			}
		}
		Expr::BinaryOp { lhs, rhs, .. } => {
			validate_expr(lhs, parameters, program, context)?;
			validate_expr(rhs, parameters, program, context)?;
		}
		Expr::UnaryOp { operand, .. } => validate_expr(operand, parameters, program, context)?,
		Expr::MemberAccess { object, .. } => validate_expr(object, parameters, program, context)?,
		Expr::Identifier(name) if name.contains('.') && !parameters.contains(name.as_str()) => {
			return Err(ForgeError::parse(format!("shared member access is not allowed: {}", name)));
		}
		_ => {}
	}
	Ok(())
}