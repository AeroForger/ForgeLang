use rayon::prelude::*;

use crate::ast::{Expr, FunctionDecl, Program, Statement};
use crate::errors::{ForgeError, ForgeResult};

pub fn analyze(program: &Program) -> ForgeResult<()> {
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
	validate_statements(&function.body, &names, program)
}

fn validate_statements(
	statements: &[Statement],
	parameters: &std::collections::HashSet<&str>,
	program: &Program,
) -> ForgeResult<()> {
	for statement in statements {
		match statement {
			Statement::VarDecl(decl) => {
				if let Some(initializer) = &decl.initializer {
					validate_expr(initializer, parameters, program)?;
				}
			}
			Statement::Assignment(assignment) => {
				if assignment.target_path.len() > 1 {
					return Err(ForgeError::parse("concurrent functions cannot mutate shared members"));
				}
				validate_expr(&assignment.value, parameters, program)?;
			}
			Statement::If(node) => {
				for (condition, body) in &node.branches {
					validate_expr(condition, parameters, program)?;
					validate_statements(body, parameters, program)?;
				}
				if let Some(body) = &node.else_body {
					validate_statements(body, parameters, program)?;
				}
			}
			Statement::While(node) => {
				validate_expr(&node.condition, parameters, program)?;
				validate_statements(&node.body, parameters, program)?;
			}
			Statement::Print(print) => validate_expr(&print.expr, parameters, program)?,
			Statement::Return(value) => {
				if let Some(value) = value {
					validate_expr(value, parameters, program)?;
				}
			}
			Statement::ExprStmt(expr) => validate_expr(expr, parameters, program)?,
			_ => {}
		}
	}
	Ok(())
}

fn validate_expr(
	expr: &Expr,
	parameters: &std::collections::HashSet<&str>,
	program: &Program,
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
				validate_expr(arg, parameters, program)?;
			}
		}
		Expr::BinaryOp { lhs, rhs, .. } => {
			validate_expr(lhs, parameters, program)?;
			validate_expr(rhs, parameters, program)?;
		}
		Expr::UnaryOp { operand, .. } => validate_expr(operand, parameters, program)?,
		Expr::MemberAccess { object, .. } => validate_expr(object, parameters, program)?,
		Expr::Identifier(name) if name.contains('.') && !parameters.contains(name.as_str()) => {
			return Err(ForgeError::parse(format!("shared member access is not allowed: {}", name)));
		}
		_ => {}
	}
	Ok(())
}