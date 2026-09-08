use std::path::PathBuf;
use std::process::ExitCode;

use crate::platform::Platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Cranelift,
    Native,
}

impl BackendKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "cranelift" => Some(Self::Cranelift),
            "native" => Some(Self::Native),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Cranelift => "cranelift",
            Self::Native => "native",
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Compile {
        input: PathBuf,
        platform: Platform,
        backend: BackendKind,
    },
    Run {
        input: PathBuf,
        backend: BackendKind,
    },
    New {
        app_type: String,
        name: String,
    },
    Backend {
        backend: BackendKind,
    },
    Version,
    Help,
}

pub fn parse_args(args: &[String]) -> Result<Command, ExitCode> {
    if args.is_empty() {
        eprintln!("error: no command specified");
        eprintln!();
        eprintln!("Usage:");
        eprintln!("    Furnace compile <file>.anvil <platform> [--backend cranelift|native]");
        eprintln!("    Furnace run <file>.anvil [--backend cranelift|native]");
        eprintln!("    Furnace backend <native|cranelift>");
        eprintln!("    Furnace new <APP_TYPE> -n <NAME>");
        eprintln!("Available backends: native, cranelift");
        eprintln!("Helpers:");
        eprintln!("    Furnace -version");
        eprintln!("    Furnace -help");
        return Err(ExitCode::from(2));
    }

    let first = args[0].as_str();

    match first {
        "-version" | "--version" | "-v" | "version" => Ok(Command::Version),
        "-help" | "--help" | "-h" | "help" => Ok(Command::Help),
        "compile" | "Compile" => {
            if args.len() < 3 {
                eprintln!("error: 'compile' requires an input .anvil file and a target platform");
                eprintln!(
                    "usage: Furnace compile <file>.anvil <platform> [--backend cranelift|native]"
                );
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

            let backend = parse_backend_flag(&args[3..])?;

            Ok(Command::Compile {
                input: input_path,
                platform,
                backend,
            })
        }
        "run" | "Run" => {
            if args.len() < 2 {
                eprintln!("error: 'run' requires an input .anvil file");
                eprintln!("usage: Furnace run <file>.anvil [--backend cranelift|native]");
                return Err(ExitCode::from(2));
            }

            let input_path = PathBuf::from(&args[1]);
            validate_anvil_extension(&input_path)?;

            let backend = parse_backend_flag(&args[2..])?;

            Ok(Command::Run {
                input: input_path,
                backend,
            })
        }
        "backend" | "Backend" => {
            if args.len() != 2 {
                eprintln!("error: 'backend' requires a backend type");
                eprintln!("usage: Furnace backend <native|cranelift>");
                eprintln!("available backends: native, cranelift");
                return Err(ExitCode::from(2));
            }

            let backend = parse_backend_kind(&args[1])?;
            Ok(Command::Backend { backend })
        }
        "new" | "New" => {
            if args.len() < 2 {
                eprintln!("error: 'new' requires an application type and a project name");
                eprintln!("usage: Furnace new <APP_TYPE> -n <NAME>");
                return Err(ExitCode::from(2));
            }
            if args.len() != 4 || args[2] != "-n" {
                eprintln!("error: 'new' requires -n <NAME>");
                eprintln!("usage: Furnace new <APP_TYPE> -n <NAME>");
                return Err(ExitCode::from(2));
            }
            if args[3].is_empty() {
                eprintln!("error: project name cannot be empty");
                return Err(ExitCode::from(2));
            }

            Ok(Command::New {
                app_type: args[1].clone(),
                name: args[3].clone(),
            })
        }
        unknown => {
            eprintln!("error: unknown command '{}'", unknown);
            eprintln!();
            eprintln!("Usage:");
            eprintln!("    Furnace compile <file>.anvil <platform> [--backend cranelift|native]");
            eprintln!("    Furnace run <file>.anvil [--backend cranelift|native]");
            eprintln!("    Furnace backend <native|cranelift>");
            eprintln!("    Furnace new <APP_TYPE> -n <NAME>");
            eprintln!("Available backends: native, cranelift");
            eprintln!("    Furnace -version");
            eprintln!("    Furnace -help");
            Err(ExitCode::from(2))
        }
    }
}

fn parse_backend_flag(remaining_args: &[String]) -> Result<BackendKind, ExitCode> {
    let mut backend = if std::env::var("FURNACE_BACKEND").as_deref() == Ok("native") {
        BackendKind::Native
    } else {
        BackendKind::Cranelift
    };
    let mut i = 0;
    while i < remaining_args.len() {
        if remaining_args[i] == "--backend" || remaining_args[i] == "-b" {
            if i + 1 >= remaining_args.len() {
                eprintln!("error: missing argument for --backend");
                return Err(ExitCode::from(2));
            }
            backend = parse_backend_kind(&remaining_args[i + 1])?;
            i += 2;
        } else {
            i += 1;
        }
    }
    Ok(backend)
}

fn parse_backend_kind(value: &str) -> Result<BackendKind, ExitCode> {
    BackendKind::parse(value).ok_or_else(|| {
        eprintln!(
            "error: unknown backend '{}'. Available backends: native, cranelift",
            value
        );
        ExitCode::from(2)
    })
}

fn validate_anvil_extension(path: &PathBuf) -> Result<(), ExitCode> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("anvil") => Ok(()),
        _ => {
            eprintln!(
                "error: input file '{}' must have a .anvil extension",
                path.display()
            );
            Err(ExitCode::from(2))
        }
    }
}
