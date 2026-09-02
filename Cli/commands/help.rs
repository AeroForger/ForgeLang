use std::process::ExitCode;

pub fn execute() -> ExitCode {
    println!("Furnace {}", furnace::VERSION);
    println!();
    println!("Usage:");
    println!("    Furnace compile <file>.anvil <platform>");
    println!("    Furnace run <file>.anvil");
    println!("    Furnace -version");
    println!("    Furnace -help");
    ExitCode::SUCCESS
}
