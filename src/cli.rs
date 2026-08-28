use std::path::PathBuf;
use std::process::ExitCode;
use std::process::Command;

use crate::codegen;
use crate::parser;

pub struct CliOptions {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub emit_ir: bool,
    pub keep_ll: bool,
    pub link_math: bool,
    pub version: bool,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn parse_args(args: &[String]) -> Result<CliOptions, ExitCode> {
    if args.is_empty() {
        eprintln!("usage: furnace <input.anvil> [-o out] [--emit-ir] [--keep-ll] [-lm] [--version]");
        return Err(ExitCode::from(2));
    }

    let mut opts = CliOptions {
        input: PathBuf::new(),
        output: None,
        emit_ir: false,
        keep_ll: false,
        link_math: false,
        version: false,
    };

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--version"        => opts.version = true,
            "--emit-ir"        => opts.emit_ir = true,
            "--emit-llvm"      => opts.emit_ir = true, // back-compat alias
            "--keep-ll"        => opts.keep_ll = true,
            "-lm"              => opts.link_math = true,
            "-o" => {
                let next = iter.next().ok_or_else(|| {
                    eprintln!("error: -o requires an argument");
                    ExitCode::from(2)
                })?;
                opts.output = Some(PathBuf::from(next));
            }
            s if s.starts_with('-') => {
                eprintln!("error: unknown flag: {}", s);
                return Err(ExitCode::from(2));
            }
            s => {
                if opts.input.as_os_str().is_empty() {
                    opts.input = PathBuf::from(s);
                } else {
                    eprintln!("error: multiple input files not supported");
                    return Err(ExitCode::from(2));
                }
            }
        }
    }

    if opts.version { return Ok(opts); }
    if opts.input.as_os_str().is_empty() {
        eprintln!("error: no input file");
        return Err(ExitCode::from(2));
    }
    Ok(opts)
}

pub fn run(opts: CliOptions) -> ExitCode {
    if opts.version {
        println!("furnace {}", VERSION);
        return ExitCode::SUCCESS;
    }

    let source = match std::fs::read_to_string(&opts.input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", opts.input.display(), e);
            return ExitCode::from(1);
        }
    };

    let program = match parser::parse_program(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            return ExitCode::from(1);
        }
    };

    if let Err(e) = crate::semantic::analyze(&program) {
        eprintln!("{}", e);
        return ExitCode::from(1);
    }

    eprintln!("parsed {} statements", program.statements.len());

    let (obj_path, output_path) = match &opts.output {
        Some(p) => (p.with_extension("o"), Some(p.clone())),
        None => (opts.input.with_extension("o"), None),
    };

    if let Err(e) = codegen::compile(&program, &obj_path, opts.link_math) {
        // codegen stub returns this; treat as "not yet implemented" but not a crash
        eprintln!("{}", e);
        return ExitCode::from(1);
    }

    if let Some(output) = output_path {
        let mut linker = Command::new("cc");
        linker.arg(&obj_path).arg("-o").arg(&output);
        if opts.link_math {
            linker.arg("-lm");
        }
        match linker.status() {
            Ok(status) if status.success() => {
                let _ = std::fs::remove_file(&obj_path);
            }
            Ok(status) => {
                eprintln!("error: linker exited with {}", status);
                return ExitCode::from(1);
            }
            Err(e) => {
                eprintln!("error: cannot invoke linker: {}", e);
                return ExitCode::from(1);
            }
        }
    }

    ExitCode::SUCCESS
}