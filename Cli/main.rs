mod args;
mod commands;
mod platform;

use std::process::ExitCode;

use crate::args::{parse_args, Command};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match parse_args(&args) {
        Ok(Command::Compile { input, platform }) => {
            if let Err(err) = commands::compile::execute(&input, platform) {
                eprintln!("{}", err);
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Ok(Command::Run { input }) => {
            match commands::run::execute(&input) {
                Ok(code) => ExitCode::from(code as u8),
                Err(err) => {
                    eprintln!("{}", err);
                    ExitCode::FAILURE
                }
            }
        }
        Ok(Command::Version) => {
            commands::version::print_version();
            ExitCode::SUCCESS
        }
        Ok(Command::Help) => {
            commands::help::print_help();
            ExitCode::SUCCESS
        }
        Err(code) => code,
    }
}
