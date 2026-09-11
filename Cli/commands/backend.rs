use std::process::ExitCode;

use crate::args::BackendKind;

pub fn execute(backend: BackendKind) -> ExitCode {
    if let Err(message) = crate::config::save_backend(backend) {
        eprintln!("error: {}", message);
        return ExitCode::from(1);
    }
    println!("Default backend set to {}", backend.name());
    ExitCode::SUCCESS
}
