pub mod elf;
pub mod x86_64;

use crate::ast::Program;
use crate::ast::{Expr, RetKind, Statement, Subtype, TypeDecl};
use crate::errors::{ForgeError, ForgeResult};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static FALLBACK_ID: AtomicU64 = AtomicU64::new(0);

pub fn compile_to_elf(program: &Program) -> ForgeResult<Vec<u8>> {
    if requires_typed_backend(program) {
        return compile_typed_elf(program);
    }
    let ir = crate::lowering::lower(program)?;
    let backend = x86_64::NativeX86Backend::new();
    let (user_code, func_offsets) = backend.compile_program(&ir)?;
    elf::generate_elf64_executable(&user_code, &func_offsets)
}

fn compile_typed_elf(program: &Program) -> ForgeResult<Vec<u8>> {
    let id = FALLBACK_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "furnace_native_typed_{}_{}",
        std::process::id(),
        id
    ));
    std::fs::create_dir_all(&directory).map_err(|error| {
        ForgeError::codegen(format!("create native build directory: {}", error))
    })?;
    let object = directory.join("program.o");
    let executable = directory.join("program");
    let runtime_source = directory.join("runtime.c");

    let result = (|| {
        crate::codegen::compile(program, &object, true)?;
        std::fs::write(&runtime_source, crate::codegen::RUNTIME_SUPPORT_C)
            .map_err(|error| ForgeError::codegen(format!("write runtime support: {}", error)))?;
        let output = Command::new("cc")
            .arg(&object)
            .arg(&runtime_source)
            .arg("-o")
            .arg(&executable)
            .arg("-lm")
            .output()
            .map_err(|error| ForgeError::codegen(format!("invoke native linker: {}", error)))?;
        if !output.status.success() {
            return Err(ForgeError::codegen(format!(
                "native linker failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        std::fs::read(&executable)
            .map_err(|error| ForgeError::codegen(format!("read native executable: {}", error)))
    })();

    let _ = std::fs::remove_file(&object);
    let _ = std::fs::remove_file(&runtime_source);
    let _ = std::fs::remove_file(&executable);
    let _ = std::fs::remove_dir(&directory);
    result
}

fn requires_typed_backend(program: &Program) -> bool {
    program
        .statements
        .iter()
        .any(statement_requires_typed_backend)
}

fn type_requires_typed_backend(ty: &TypeDecl) -> bool {
    matches!(
        ty,
        TypeDecl::Number(Subtype::Float)
            | TypeDecl::Ore(_)
            | TypeDecl::OreTuple(_)
            | TypeDecl::Materials(_, _)
    )
}

fn return_requires_typed_backend(ret: &RetKind) -> bool {
    matches!(
        ret,
        RetKind::Float | RetKind::Ore(_) | RetKind::OreTuple(_) | RetKind::Materials(_, _)
    )
}

fn statement_requires_typed_backend(statement: &Statement) -> bool {
    match statement {
        Statement::DataDecl(_) | Statement::ObjectDecl(_) => true,
        Statement::VarDecl(decl) => {
            type_requires_typed_backend(&decl.type_decl)
                || decl
                    .initializer
                    .as_ref()
                    .is_some_and(expr_requires_typed_backend)
        }
        Statement::FunctionDecl(function) => {
            return_requires_typed_backend(&function.ret_kind)
                || function
                    .params
                    .iter()
                    .any(|param| type_requires_typed_backend(&param.type_decl))
                || function.body.iter().any(statement_requires_typed_backend)
        }
        Statement::Print(print) => expr_requires_typed_backend(&print.expr),
        Statement::Input(_) => true,
        Statement::If(node) => {
            node.branches.iter().any(|(condition, body)| {
                expr_requires_typed_backend(condition)
                    || body.iter().any(statement_requires_typed_backend)
            }) || node
                .else_body
                .as_ref()
                .is_some_and(|body| body.iter().any(statement_requires_typed_backend))
        }
        Statement::While(node) => {
            expr_requires_typed_backend(&node.condition)
                || node.body.iter().any(statement_requires_typed_backend)
        }
        Statement::For(node) => {
            type_requires_typed_backend(&node.init.type_decl)
                || node
                    .init
                    .initializer
                    .as_ref()
                    .is_some_and(expr_requires_typed_backend)
                || expr_requires_typed_backend(&node.condition)
                || node.body.iter().any(statement_requires_typed_backend)
        }
        Statement::Return(value) => value.as_ref().is_some_and(expr_requires_typed_backend),
        Statement::Assignment(assignment) => expr_requires_typed_backend(&assignment.value),
        Statement::ExprStmt(expr) => expr_requires_typed_backend(expr),
        Statement::Stop | Statement::Skip | Statement::Use(_) => false,
    }
}

fn expr_requires_typed_backend(expr: &Expr) -> bool {
    match expr {
        Expr::Number(number) => number.is_float,
        Expr::Input(_) => true,
        Expr::ArrayLiteral(_) | Expr::TupleLiteral(_) | Expr::ListLiteral(_) => true,
        Expr::MemberAccess { object, .. } => expr_requires_typed_backend(object),
        Expr::IndexAccess { object, index } => {
            expr_requires_typed_backend(object) || expr_requires_typed_backend(index)
        }
        Expr::BinaryOp { lhs, rhs, .. } => {
            expr_requires_typed_backend(lhs) || expr_requires_typed_backend(rhs)
        }
        Expr::UnaryOp { operand, .. } => expr_requires_typed_backend(operand),
        Expr::Call { args, .. } | Expr::NamespaceCall { args, .. } => {
            args.iter().any(expr_requires_typed_backend)
        }
        Expr::MethodCall { object, args, .. } => {
            expr_requires_typed_backend(object) || args.iter().any(expr_requires_typed_backend)
        }
        Expr::Str(_) | Expr::Bool(_) | Expr::Identifier(_) => false,
    }
}
