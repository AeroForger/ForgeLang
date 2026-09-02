use std::process::ExitCode;

mod args;
mod commands;
mod platform;

fn main() -> ExitCode {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    let command = match args::parse_args(&raw_args) {
        Ok(cmd) => cmd,
        Err(exit_code) => return exit_code,
    };

    match command {
        args::Command::Compile { input, platform } => {
            commands::compile::execute(&input, platform)
        }
        args::Command::Run { input } => {
            commands::run::execute(&input)
        }
        args::Command::Version => {
            commands::version::execute()
        }
        args::Command::Help => {
            commands::help::execute()
        }
    }
}
