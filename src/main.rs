use std::process::ExitCode;

mod ast;
mod cli;
mod codegen;
mod errors;
mod parser;
mod semantic;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match cli::parse_args(&args) {
        Ok(opts) => cli::run(opts),
        Err(code) => code,
    }
}