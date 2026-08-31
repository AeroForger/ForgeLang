use furnace::VERSION;

pub fn print_help() {
    println!("Furnace {}", VERSION);
    println!();
    println!("Usage:");
    println!("    Furnace compile <file>.anvil <platform>");
    println!("    Furnace run <file>.anvil");
    println!("    Furnace -version");
    println!("    Furnace -help");
}
