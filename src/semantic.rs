use rayon::prelude::*;

use crate::ast::{Expr, FunctionDecl, Program, Statement};
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
			if let Some(initializer) = &decl.initializer {
				validate_expr(initializer, parameters, program, context)?;
			}
		}
		Statement::Assignment(assignment) => {
			if assignment.target_path.len() > 1 {
				return Err(ForgeError::parse("concurrent functions cannot mutate shared members"));
			}
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