use std::fs;
use std::path::Path;
use std::process::Command;

fn write_project(root: &std::path::Path, name: &str) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/Helpers.anvil"),
        "Open Int Answer() { Return 42; }\n",
    )
    .unwrap();
    fs::write(
        root.join("src/Main.anvil"),
        "Using Helpers: Answer; Open Nunction Main() { Print(Answer()); }\n",
    )
    .unwrap();
    fs::write(
        root.join("project.blower"),
        format!(
            "Project {{ Name = \"{}\"; }} Files {{ location = \"src/*.anvil\"; }}",
            name
        ),
    )
    .unwrap();
}

fn write_config(root: &Path, contents: &str) {
    fs::write(root.join("project.blower"), contents).unwrap();
}

#[test]
fn build_compiles_multiple_sources_to_named_build_output_with_both_backends() {
    let temp = tempfile::tempdir().unwrap();
    write_project(temp.path(), "MultiFileApp");
    let binary = env!("CARGO_BIN_EXE_furnace");

    for backend in ["native", "cranelift"] {
        let result = Command::new(binary)
            .current_dir(temp.path().parent().unwrap())
            .args([
                "build",
                temp.path().join("project.blower").to_str().unwrap(),
                "--backend",
                backend,
            ])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{} build failed: {}",
            backend,
            String::from_utf8_lossy(&result.stderr)
        );
        let output = temp.path().join("build/MultiFileApp");
        assert!(output.is_file());
        let run = Command::new(&output).output().unwrap();
        assert!(run.status.success());
        assert_eq!(String::from_utf8_lossy(&run.stdout).trim(), "42");
    }
}

#[test]
fn build_rejects_non_blower_target() {
    let result = Command::new(env!("CARGO_BIN_EXE_furnace"))
        .args(["build", "main.anvil"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr)
        .contains("furnace build requires a .blower project file"));
}

#[test]
fn build_requires_a_project_argument() {
    let result = Command::new(env!("CARGO_BIN_EXE_furnace"))
        .arg("build")
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("'build' requires a .blower project file")
    );
}

#[test]
fn build_rejects_unsupported_positional_target_arguments() {
    let temp = tempfile::tempdir().unwrap();
    write_project(temp.path(), "ArgumentsApp");
    let result = Command::new(env!("CARGO_BIN_EXE_furnace"))
        .args([
            "build",
            temp.path().join("project.blower").to_str().unwrap(),
            "linux",
        ])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr)
        .contains("unsupported furnace build argument 'linux'"));
}

#[test]
fn build_rejects_unknown_and_missing_backend_values() {
    let temp = tempfile::tempdir().unwrap();
    write_project(temp.path(), "BackendApp");
    let project = temp.path().join("project.blower");

    let unknown = Command::new(env!("CARGO_BIN_EXE_furnace"))
        .args(["build", project.to_str().unwrap(), "--backend", "llvm"])
        .output()
        .unwrap();
    assert!(!unknown.status.success());
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown backend 'llvm'"));

    let missing = Command::new(env!("CARGO_BIN_EXE_furnace"))
        .args(["build", project.to_str().unwrap(), "--backend"])
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr)
        .contains("unsupported furnace build argument '--backend'"));
}

#[test]
fn build_resolves_locations_and_output_relative_to_blower_not_cwd() {
    let temp = tempfile::tempdir().unwrap();
    let project_dir = temp.path().join("workspace/app");
    let invocation_dir = temp.path().join("elsewhere");
    fs::create_dir_all(project_dir.join("code")).unwrap();
    fs::create_dir(&invocation_dir).unwrap();
    fs::write(
        project_dir.join("code/Main.anvil"),
        "Open Int Main() { Return 23; }",
    )
    .unwrap();
    fs::write(
        project_dir.join("app.blower"),
        "Project { Name=\"RelativeApp\"; } Files { location=\"code/*.anvil\"; }",
    )
    .unwrap();

    let result = Command::new(env!("CARGO_BIN_EXE_furnace"))
        .current_dir(&invocation_dir)
        .args(["build", "../workspace/app/app.blower", "-b", "native"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(project_dir.join("build/RelativeApp").is_file());
    assert!(!invocation_dir.join("build/RelativeApp").exists());
}

#[test]
fn build_uses_multiple_location_entries() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("src")).unwrap();
    fs::create_dir(temp.path().join("support")).unwrap();
    fs::write(
        temp.path().join("support/Answer.anvil"),
        "Open Int Answer() { Return 31; }",
    )
    .unwrap();
    fs::write(
        temp.path().join("src/Main.anvil"),
        "Using Answer: Answer; Open Int Main() { Return Answer(); }",
    )
    .unwrap();
    write_config(
        temp.path(),
        "Project { Name=\"LocationsApp\"; } Files { location=\"support/*.anvil\"; location=\"src/*.anvil\"; }",
    );

    let result = Command::new(env!("CARGO_BIN_EXE_furnace"))
        .args([
            "build",
            temp.path().join("project.blower").to_str().unwrap(),
            "--backend",
            "native",
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let run = Command::new(temp.path().join("build/LocationsApp"))
        .status()
        .unwrap();
    assert_eq!(run.code(), Some(31));
}

#[test]
fn build_reports_malformed_config_and_unmatched_sources() {
    let temp = tempfile::tempdir().unwrap();
    write_config(
        temp.path(),
        "Project { Name \"Broken\"; } Files { location=\"src/*.anvil\"; }",
    );
    let project = temp.path().join("project.blower");
    let malformed = Command::new(env!("CARGO_BIN_EXE_furnace"))
        .env("FURNACE_CONFIG_DIR", temp.path().join("config-home"))
        .args(["build", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!malformed.status.success());
    assert!(String::from_utf8_lossy(&malformed.stderr).contains("invalid .blower syntax at line 1"));

    fs::create_dir(temp.path().join("src")).unwrap();
    write_config(
        temp.path(),
        "Project { Name=\"Empty\"; } Files { location=\"src/*.anvil\"; }",
    );
    let unmatched = Command::new(env!("CARGO_BIN_EXE_furnace"))
        .env("FURNACE_CONFIG_DIR", temp.path().join("config-home"))
        .args(["build", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!unmatched.status.success());
    assert!(String::from_utf8_lossy(&unmatched.stderr)
        .contains("no Sydrogen source files matched 'src/*.anvil'"));
}

#[test]
fn standalone_compile_retains_its_existing_output_behavior() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(
        temp.path().join("standalone.anvil"),
        "Open Int Main() { Return 9; }",
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_furnace"))
        .current_dir(temp.path())
        .args([
            "compile",
            "standalone.anvil",
            "linux",
            "--backend",
            "native",
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(temp.path().join("standalone").is_file());
    assert!(!temp.path().join("build/standalone").exists());
}

#[test]
fn new_creates_an_immediately_buildable_blower_project() {
    let temp = tempfile::tempdir().unwrap();
    let binary = env!("CARGO_BIN_EXE_furnace");
    let created = Command::new(binary)
        .current_dir(temp.path())
        .args(["new", "console", "-n", "FreshApp"])
        .output()
        .unwrap();
    assert!(created.status.success());
    assert!(temp.path().join("FreshApp/FreshApp.blower").is_file());
    assert!(temp.path().join("FreshApp/src/Main.anvil").is_file());
    let config = fs::read_to_string(temp.path().join("FreshApp/FreshApp.blower")).unwrap();
    assert!(config.contains("Name = \"FreshApp\";"));
    assert!(config.contains("location = \"src/*.anvil\";"));

    let built = Command::new(binary)
        .current_dir(temp.path().join("FreshApp"))
        .args(["build", "FreshApp.blower", "--backend", "native"])
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert!(temp.path().join("FreshApp/build/FreshApp").is_file());
}

#[test]
fn new_rejects_names_that_cannot_be_project_output_names() {
    let temp = tempfile::tempdir().unwrap();
    for name in ["has space", "has.dot", "unicode-λ"] {
        let result = Command::new(env!("CARGO_BIN_EXE_furnace"))
            .current_dir(temp.path())
            .args(["new", "console", "-n", name])
            .output()
            .unwrap();
        assert!(!result.status.success(), "new unexpectedly accepted {name}");
        assert!(!temp.path().join(name).exists());
    }
}
