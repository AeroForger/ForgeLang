use std::path::Path;
use std::process::{Command, ExitCode};

use crate::platform::Platform;

pub fn execute(input: &Path) -> ExitCode {
    let source = match std::fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read '{}': {}", input.display(), e);
            return ExitCode::from(1);
        }
    };

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

    let pid = std::process::id();
    let temp_dir = std::env::temp_dir();
    let obj_path = temp_dir.join(format!("furnace_run_{}_{}.o", pid, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let exe_path = temp_dir.join(format!("furnace_run_{}_{}.exe", pid, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));

    if let Err(e) = furnace::codegen::compile(&program, &obj_path, true) {
        eprintln!("{}", e);
        return ExitCode::from(1);
    }

    let platform = Platform::Linux;
    let mut linker = Command::new(platform.linker_name());
    linker.arg(&obj_path).arg("-o").arg(&exe_path);
    for flag in platform.default_linker_flags() {
        linker.arg(flag);
    }

    let link_status = match linker.status() {
        Ok(status) => status,
        Err(e) => {
            let _ = std::fs::remove_file(&obj_path);
            eprintln!("error: cannot invoke linker '{}': {}", platform.linker_name(), e);
            return ExitCode::from(1);
        }
    };

    let _ = std::fs::remove_file(&obj_path);

    if !link_status.success() {
        eprintln!("error: linker exited with status {}", link_status);
        return ExitCode::from(1);
    }

    let run_status = match Command::new(&exe_path)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
    {
        Ok(status) => status,
        Err(e) => {
            let _ = std::fs::remove_file(&exe_path);
            eprintln!("error: failed to execute compiled program: {}", e);
            return ExitCode::from(1);
        }
    };

    let _ = std::fs::remove_file(&exe_path);

    if let Some(code) = run_status.code() {
        ExitCode::from(code as u8)
    } else {
        ExitCode::from(1)
    }
}
