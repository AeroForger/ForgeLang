use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use furnace::{codegen, parser::parse_program, semantic};

use crate::platform::Platform;

pub fn execute(input: &Path, platform: Platform) -> Result<(), String> {
    println!("Compiling {}...", input.display());

    let (obj_path, output_path) = compile_to_object(input)?;

    println!("Linking...");
    link_object(&obj_path, &output_path, platform)?;

    println!("Build successful!");
    println!("Output: {}", output_path.display());
    Ok(())
}

pub(crate) fn compile_to_object(input: &Path) -> Result<(PathBuf, PathBuf), String> {
    if !input.exists() {
        return Err(format!("error: input file '{}' does not exist", input.display()));
    }

    let source = fs::read_to_string(input)
        .map_err(|e| format!("error: failed to read '{}': {}", input.display(), e))?;

    let program = parse_program(&source)
        .map_err(|e| format!("error: failed to parse '{}': {}", input.display(), e))?;

    semantic::analyze(&program)
        .map_err(|e| format!("error: semantic analysis failed for '{}': {}", input.display(), e))?;

    let output_dir = input
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("error: invalid input file name '{}'", input.display()))?;

    let obj_path = output_dir.join(format!("{}.o", stem));
    let output_path = output_dir.join(stem);

    codegen::compile(&program, &obj_path, true)
        .map_err(|e| format!("error: code generation failed for '{}': {}", input.display(), e))?;

    Ok((obj_path, output_path))
}

pub(crate) fn link_object(obj_path: &Path, output_path: &Path, platform: Platform) -> Result<(), String> {
    let mut cmd = Command::new(platform.linker_name());
    cmd.arg(obj_path)
        .arg("-o")
        .arg(output_path)
        .args(platform.default_linker_flags());

    let status = cmd.status().map_err(|e| {
        format!(
            "error: failed to invoke linker '{}' for '{}': {}",
            platform.linker_name(),
            output_path.display(),
            e
        )
    })?;

    if !status.success() {
        return Err(format!(
            "error: linking failed for '{}' ({} exited with status {:?})",
            output_path.display(),
            platform.linker_name(),
            status.code().unwrap_or_default()
        ));
    }

    Ok(())
}
