use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use crate::args::BackendKind;
use crate::platform::Platform;

pub fn execute(input: &Path, platform: Platform, backend: BackendKind) -> ExitCode {
    let source = match std::fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {}", input.display(), e);
            return ExitCode::from(1);
        }
    };

    println!("Compiling {}...", input.display());

    let program = match furnace::parser::parse_program(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::from(1);
        }
    };

    if let Err(e) = furnace::semantic::analyze(&program) {
        eprintln!("{}", e);
        return ExitCode::from(1);
    }

    let stem = match input.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => "main",
    };

    let output_exe = if input
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        == Some(stem)
    {
        input.parent().unwrap().join(stem)
    } else {
        PathBuf::from(format!("./{}", stem))
    };

    match backend {
        BackendKind::Native => {
            println!("Linking...");
            let elf_bytes = match furnace::backend::compile_to_elf(&program) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("{}", e);
                    return ExitCode::from(1);
                }
            };

            if let Err(e) = std::fs::write(&output_exe, elf_bytes) {
                eprintln!(
                    "error: cannot write executable '{}': {}",
                    output_exe.display(),
                    e
                );
                return ExitCode::from(1);
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = std::fs::metadata(&output_exe) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    let _ = std::fs::set_permissions(&output_exe, perms);
                }
            }

            println!("Build successful!");
            println!("Backend: Native ELF64");
            println!("Output: {}", output_exe.display());
            ExitCode::SUCCESS
        }
        BackendKind::Cranelift => {
            let obj_path = PathBuf::from(format!("{}.o", stem));
            if let Err(e) = furnace::codegen::compile(&program, &obj_path, true) {
                eprintln!("{}", e);
                return ExitCode::from(1);
            }

            println!("Linking...");

            let runtime_path = PathBuf::from(format!("{}.runtime.c", stem));
            if let Err(e) = std::fs::write(&runtime_path, furnace::codegen::RUNTIME_SUPPORT_C) {
                let _ = std::fs::remove_file(&obj_path);
                eprintln!("error: cannot write runtime support: {}", e);
                return ExitCode::from(1);
            }

            let mut linker = Command::new(platform.linker_name());
            linker
                .arg(&obj_path)
                .arg(&runtime_path)
                .arg("-o")
                .arg(&output_exe);
            for flag in platform.default_linker_flags() {
                linker.arg(flag);
            }

            match linker.status() {
                Ok(status) if status.success() => {
                    let _ = std::fs::remove_file(&obj_path);
                    let _ = std::fs::remove_file(&runtime_path);
                    println!("Build successful!");
                    println!("Output: {}", output_exe.display());
                    ExitCode::SUCCESS
                }
                Ok(status) => {
                    let _ = std::fs::remove_file(&obj_path);
                    let _ = std::fs::remove_file(&runtime_path);
                    eprintln!("error: linker exited with status {}", status);
                    ExitCode::from(1)
                }
                Err(e) => {
                    let _ = std::fs::remove_file(&obj_path);
                    let _ = std::fs::remove_file(&runtime_path);
                    eprintln!(
                        "error: cannot invoke linker '{}': {}",
                        platform.linker_name(),
                        e
                    );
                    ExitCode::from(1)
                }
            }
        }
    }
}
