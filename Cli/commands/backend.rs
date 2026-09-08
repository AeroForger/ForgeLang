use std::process::ExitCode;

use crate::args::BackendKind;

pub fn execute(backend: BackendKind) -> ExitCode {
    println!("Backend: {}", backend.name());

    match backend {
        BackendKind::Native => {
            println!("Output: direct x86-64 ELF64 executable");
        }
        BackendKind::Cranelift => {
            println!("Output: native object file linked with cc");
        }
    }

    ExitCode::SUCCESS
}
