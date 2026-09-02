use std::process::ExitCode;

pub fn execute() -> ExitCode {
    println!("Furnace {}", furnace::VERSION);
    ExitCode::SUCCESS
}
