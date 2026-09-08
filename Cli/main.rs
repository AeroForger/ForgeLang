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
        args::Command::Compile {
            input,
            platform,
            backend,
        } => commands::compile::execute(&input, platform, backend),
        args::Command::Run { input, backend } => commands::run::execute(&input, backend),
        args::Command::New { app_type, name } => commands::new::execute(&app_type, &name),
        args::Command::Backend { backend } => commands::backend::execute(backend),
        args::Command::Version => commands::version::execute(),
        args::Command::Help => commands::help::execute(),
    }
}
