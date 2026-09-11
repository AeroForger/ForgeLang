use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn furnace(config_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_furnace"));
    command.env("FURNACE_CONFIG_DIR", config_dir);
    command
}

fn run(command: &mut Command) -> Output {
    command.output().unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn write_source(directory: &Path) -> PathBuf {
    let source = directory.join("main.anvil");
    fs::write(
        &source,
        "Open Nunction Main() { Print(\"configured-backend\"); }",
    )
    .unwrap();
    source
}

fn write_project(directory: &Path) -> PathBuf {
    fs::create_dir_all(directory.join("src")).unwrap();
    fs::write(
        directory.join("src/Main.anvil"),
        "Open Nunction Main() { Print(\"project-run\"); }",
    )
    .unwrap();
    let project = directory.join("app.blower");
    fs::write(
        &project,
        "Project { Name=\"ConfiguredApp\"; } Files { location=\"src/*.anvil\"; }",
    )
    .unwrap();
    project
}

#[test]
fn backend_command_saves_native_and_cranelift() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config-home");

    let native = run(furnace(&config_dir).args(["backend", "native"]));
    assert!(native.status.success(), "{}", stderr(&native));
    assert_eq!(stdout(&native), "Default backend set to native\n");
    assert_eq!(
        fs::read_to_string(config_dir.join("config")).unwrap(),
        "backend = \"native\"\n"
    );

    let cranelift = run(furnace(&config_dir).args(["backend", "cranelift"]));
    assert!(cranelift.status.success(), "{}", stderr(&cranelift));
    assert_eq!(stdout(&cranelift), "Default backend set to cranelift\n");
    assert_eq!(
        fs::read_to_string(config_dir.join("config")).unwrap(),
        "backend = \"cranelift\"\n"
    );
}

#[test]
fn saved_backend_persists_for_build_and_compile_processes() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config-home");
    let source = write_source(temp.path());
    let project = write_project(&temp.path().join("project"));

    assert!(run(furnace(&config_dir).args(["backend", "native"]))
        .status
        .success());
    let native_compile = run(furnace(&config_dir).current_dir(temp.path()).args([
        "compile",
        source.to_str().unwrap(),
        "linux",
    ]));
    assert!(
        native_compile.status.success(),
        "{}",
        stderr(&native_compile)
    );
    assert!(stdout(&native_compile).contains("Backend: native"));
    let native_build = run(furnace(&config_dir).args(["build", project.to_str().unwrap()]));
    assert!(native_build.status.success(), "{}", stderr(&native_build));
    assert!(stdout(&native_build).contains("Backend: native"));

    assert!(run(furnace(&config_dir).args(["backend", "cranelift"]))
        .status
        .success());
    let cranelift_compile = run(furnace(&config_dir).current_dir(temp.path()).args([
        "compile",
        source.to_str().unwrap(),
        "linux",
    ]));
    assert!(
        cranelift_compile.status.success(),
        "{}",
        stderr(&cranelift_compile)
    );
    assert!(stdout(&cranelift_compile).contains("Backend: cranelift"));
    let cranelift_build = run(furnace(&config_dir).args(["build", project.to_str().unwrap()]));
    assert!(
        cranelift_build.status.success(),
        "{}",
        stderr(&cranelift_build)
    );
    assert!(stdout(&cranelift_build).contains("Backend: cranelift"));
}

#[test]
fn saved_backend_is_used_by_run_for_anvil_and_blower_inputs() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config-home");
    let source = write_source(temp.path());
    let project = write_project(&temp.path().join("project"));

    assert!(run(furnace(&config_dir).args(["backend", "native"]))
        .status
        .success());
    for input in [&source, &project] {
        let output = run(furnace(&config_dir)
            .env("PATH", "")
            .args(["run", input.to_str().unwrap()]));
        assert!(output.status.success(), "{}", stderr(&output));
    }

    assert!(run(furnace(&config_dir).args(["backend", "cranelift"]))
        .status
        .success());
    let output = run(furnace(&config_dir)
        .env("PATH", "")
        .args(["run", source.to_str().unwrap()]));
    assert!(!output.status.success());
    assert!(stderr(&output).contains("cannot invoke linker 'cc'"));
}

#[test]
fn explicit_override_precedes_saved_and_even_malformed_config() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config-home");
    let source = write_source(temp.path());
    assert!(run(furnace(&config_dir).args(["backend", "cranelift"]))
        .status
        .success());

    let native = run(furnace(&config_dir).current_dir(temp.path()).args([
        "compile",
        source.to_str().unwrap(),
        "linux",
        "--backend",
        "native",
    ]));
    assert!(native.status.success(), "{}", stderr(&native));
    assert!(stdout(&native).contains("Backend: native"));

    fs::write(config_dir.join("config"), "corrupted").unwrap();
    let explicit = run(furnace(&config_dir).current_dir(temp.path()).args([
        "compile",
        source.to_str().unwrap(),
        "linux",
        "--backend",
        "native",
    ]));
    assert!(explicit.status.success(), "{}", stderr(&explicit));
}

#[test]
fn missing_config_uses_cranelift_and_malformed_config_is_not_ignored() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config-home");
    let source = write_source(temp.path());

    let default = run(furnace(&config_dir).current_dir(temp.path()).args([
        "compile",
        source.to_str().unwrap(),
        "linux",
    ]));
    assert!(default.status.success(), "{}", stderr(&default));
    assert!(stdout(&default).contains("Backend: cranelift"));

    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config"), "backend = \"llvm\"\n").unwrap();
    let malformed = run(furnace(&config_dir).current_dir(temp.path()).args([
        "compile",
        source.to_str().unwrap(),
        "linux",
    ]));
    assert!(!malformed.status.success());
    assert!(stderr(&malformed).contains("unknown backend 'llvm'"));
}

#[test]
fn unknown_backend_and_configuration_write_failures_are_reported() {
    let temp = tempfile::tempdir().unwrap();
    let config_dir = temp.path().join("config-home");
    let unknown = run(furnace(&config_dir).args(["backend", "llvm"]));
    assert!(!unknown.status.success());
    assert!(stderr(&unknown).contains("unknown backend 'llvm'"));

    fs::write(&config_dir, "blocks directory creation").unwrap();
    let unwritable = run(furnace(&config_dir).args(["backend", "native"]));
    assert!(!unwritable.status.success());
    assert!(stderr(&unwritable).contains("cannot create Furnace configuration directory"));
}
