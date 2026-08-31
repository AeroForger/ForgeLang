use std::path::PathBuf;
use std::process::ExitCode;

use crate::platform::Platform;

#[derive(Debug)]
pub enum Command {
    Compile {
        input: PathBuf,
        platform: Platform,
    },
    Run {
        input: PathBuf,
    },
    Version,
    Help,
}

pub fn parse_args(args: &[String]) -> Result<Command, ExitCode> {
    if args.is_empty() {
        eprintln!("error: no command specified");
        eprintln!();
        eprintln!("Usage:");
        eprintln!("    Furnace compile <file>.anvil <platform>");
        eprintln!("    Furnace run <file>.anvil");
        eprintln!("    Furnace -version");
        eprintln!("    Furnace -help");
        return Err(ExitCode::from(2));
    }

    let first = args[0].as_str();

    match first {
        "-version" | "--version" | "-v" | "version" => Ok(Command::Version),
        "-help" | "--help" | "-h" | "help" => Ok(Command::Help),
        "compile" | "Compile" => {
            if args.len() < 2 {
                eprintln!("error: 'compile' requires an input .anvil file and a target platform");
                eprintln!("usage: Furnace compile <file>.anvil <platform>");
                return Err(ExitCode::from(2));
            }
            if args.len() < 3 {
                eprintln!("error: missing target platform");
                eprintln!("usage: Furnace compile <file>.anvil <platform>");
                return Err(ExitCode::from(2));
            }

            let input_path = PathBuf::from(&args[1]);
            validate_anvil_extension(&input_path)?;

            let platform = match Platform::parse(&args[2]) {
                Ok(p) => p,
                Err(err_msg) => {
                    eprintln!("error: {}", err_msg);
                    return Err(ExitCode::from(2));
                }
            };

            Ok(Command::Compile {
                input: input_path,
                platform,
            })
        }
        "run" | "Run" => {
            if args.len() < 2 {
                eprintln!("error: 'run' requires an input .anvil file");
                eprintln!("usage: Furnace run <file>.anvil");
                return Err(ExitCode::from(2));
            }

            let input_path = PathBuf::from(&args[1]);
            validate_anvil_extension(&input_path)?;

            Ok(Command::Run {
                input: input_path,
            })
        }
        unknown => {
            eprintln!("error: unknown command '{}'", unknown);
            eprintln!();
            eprintln!("Usage:");
            eprintln!("    Furnace compile <file>.anvil <platform>");
            eprintln!("    Furnace run <file>.anvil");
            eprintln!("    Furnace -version");
            eprintln!("    Furnace -help");
            Err(ExitCode::from(2))
        }
    }
}

fn validate_anvil_extension(path: &PathBuf) -> Result<(), ExitCode> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("anvil") => Ok(()),
        _ => {
            eprintln!("error: input file '{}' must have a .anvil extension", path.display());
            Err(ExitCode::from(2))
        }
    }
}
