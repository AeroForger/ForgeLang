use std::path::{Path, PathBuf};
use std::process::ExitCode;

const CONSOLE_TEMPLATE: &str = "Open Nunction Main()\n{\n}\n";

pub fn execute(app_type: &str, name: &str) -> ExitCode {
    if app_type != "console" {
        eprintln!(
            "error: unsupported application type '{}'; expected console",
            app_type
        );
        return ExitCode::from(2);
    }

    if let Err(message) = validate_name(name) {
        eprintln!("error: {}", message);
        return ExitCode::from(2);
    }

    let project_dir = PathBuf::from(name);
    if project_dir.exists() {
        eprintln!(
            "error: directory '{}' already exists",
            project_dir.display()
        );
        return ExitCode::from(1);
    }

    if let Err(error) = create_project(&project_dir, name) {
        eprintln!("error: cannot create project '{}': {}", name, error);
        return ExitCode::from(1);
    }

    println!("Created {} project '{}'", app_type, name);
    ExitCode::SUCCESS
}

fn validate_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty() {
        return Err("project name cannot be empty");
    }
    if name == "." || name == ".." || name.contains('/') || name.contains('\\') {
        return Err("project name must be a single safe filesystem name");
    }
    if name.chars().any(|character| character.is_control()) {
        return Err("project name cannot contain control characters");
    }
    if !name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return Err("project name may contain only letters, numbers, '_' and '-'");
    }
    Ok(())
}

fn create_project(project_dir: &Path, name: &str) -> std::io::Result<()> {
    std::fs::create_dir(project_dir)?;
    let source_dir = project_dir.join("src");
    let project_file = project_dir.join(format!("{}.blower", name));
    let result = (|| {
        std::fs::create_dir(&source_dir)?;
        std::fs::write(source_dir.join("Main.anvil"), CONSOLE_TEMPLATE)?;
        let config = format!(
            "Project\n{{\n    Name = \"{}\";\n}}\n\nFiles\n{{\n    location = \"src/*.anvil\";\n}}\n",
            name
        );
        std::fs::write(&project_file, config)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&project_file);
        let _ = std::fs::remove_file(source_dir.join("Main.anvil"));
        let _ = std::fs::remove_dir(&source_dir);
        let _ = std::fs::remove_dir(project_dir);
    }
    result
}
