use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use crate::platform::Platform;

pub fn execute(input: &Path, platform: Platform) -> ExitCode {
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

    let obj_path = PathBuf::from(format!("{}.o", stem));
    let output_exe = PathBuf::from(format!("./{}", stem));

    if let Err(e) = furnace::codegen::compile(&program, &obj_path, true) {
        eprintln!("{}", e);
        return ExitCode::from(1);
    }

    println!("Linking...");

    let mut linker = Command::new(platform.linker_name());
    linker.arg(&obj_path).arg("-o").arg(&output_exe);
    for flag in platform.default_linker_flags() {
        linker.arg(flag);
    }

    match linker.status() {
        Ok(status) if status.success() => {
            let _ = std::fs::remove_file(&obj_path);
            println!("Build successful!");
            println!("Output: {}", output_exe.display());
            ExitCode::SUCCESS
        }
        Ok(status) => {
            let _ = std::fs::remove_file(&obj_path);
            eprintln!("error: linker exited with status {}", status);
            ExitCode::from(1)
        }
        Err(e) => {
            let _ = std::fs::remove_file(&obj_path);
            eprintln!("error: cannot invoke linker '{}': {}", platform.linker_name(), e);
            ExitCode::from(1)
        }
    }
}
