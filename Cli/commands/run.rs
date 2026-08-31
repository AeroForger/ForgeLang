use std::path::Path;
use std::process::{Command, Stdio};

use crate::platform::Platform;

pub fn execute(input: &Path) -> Result<i32, String> {
    let platform = Platform::parse("linux")
        .map_err(|message| format!("error: {}", message))?;

    println!("Compiling {}...", input.display());
    let (obj_path, output_path) = super::compile::compile_to_object(input)?;

    println!("Linking...");
    super::compile::link_object(&obj_path, &output_path, platform)?;

    let status = Command::new(&output_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("error: failed to execute '{}': {}", output_path.display(), e))?;

    Ok(status.code().unwrap_or(1))
}
