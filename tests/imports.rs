use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn furnace() -> Command {
    Command::new(env!("CARGO_BIN_EXE_furnace"))
}

fn run(command: &mut Command) -> Output {
    command.output().unwrap()
}

fn write_file(path: impl AsRef<Path>, contents: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn project(root: &Path, patterns: &[&str]) -> PathBuf {
    let locations = patterns
        .iter()
        .map(|pattern| format!("    location = \"{}\";\n", pattern))
        .collect::<String>();
    let path = root.join("Imports.blower");
    write_file(
        &path,
        &format!("Project {{ Name=\"Imports\"; }}\nFiles {{\n{locations}}}\n"),
    );
    path
}

fn build(project: &Path, backend: &str) -> Output {
    run(furnace().args(["build", project.to_str().unwrap(), "--backend", backend]))
}

fn error(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn use_and_using_compile_and_run_with_both_backends() {
    let temp = tempfile::tempdir().unwrap();
    write_file(
        temp.path().join("src/Math.anvil"),
        "Open Int Add(Int A, Int B) { Return A + B; }\nOpen Int Seven() { Return 7; }",
    );
    write_file(
        temp.path().join("src/Main.anvil"),
        "Use Math;\nUsing Math: Seven;\nOpen Nunction Main() { Print(Math.Add(2, 3)); Print(Seven()); }",
    );
    let project = project(temp.path(), &["src/*.anvil"]);

    for backend in ["native", "cranelift"] {
        let output = build(&project, backend);
        assert!(output.status.success(), "{backend}: {}", error(&output));
        let executed = run(&mut Command::new(temp.path().join("build/Imports")));
        assert!(executed.status.success());
        assert_eq!(String::from_utf8_lossy(&executed.stdout), "5\n7\n");
    }
}

#[test]
fn imported_calls_keep_normal_semantic_checks_and_source_names() {
    let temp = tempfile::tempdir().unwrap();
    write_file(
        temp.path().join("src/Math.anvil"),
        "Open Int Add(Int A, Int B) { Return A + B; }",
    );
    write_file(
        temp.path().join("src/Main.anvil"),
        "Using Math: Add; Open Nunction Main() { Add(1); }",
    );
    let output = build(&project(temp.path(), &["src/*.anvil"]), "native");
    assert!(!output.status.success());
    let message = error(&output);
    assert!(
        message.contains("Function Add expects 2 arguments, got 1"),
        "{message}"
    );
    assert!(!message.contains("__forge_"), "{message}");
}

#[test]
fn use_preserves_namespace_and_does_not_import_direct_symbol() {
    let temp = tempfile::tempdir().unwrap();
    write_file(
        temp.path().join("src/File.anvil"),
        "Open Nunction Function1() {}",
    );
    write_file(
        temp.path().join("src/Main.anvil"),
        "Use File; Open Nunction Main() { Function1(); }",
    );
    let output = build(&project(temp.path(), &["src/*.anvil"]), "native");
    assert!(!output.status.success());
    assert!(error(&output).contains("Undefined function: Function1"));
}

#[test]
fn reports_missing_modules_symbols_and_private_symbols() {
    let missing_module = tempfile::tempdir().unwrap();
    write_file(
        missing_module.path().join("src/Main.anvil"),
        "Use Missing; Open Nunction Main() {}",
    );
    let output = build(&project(missing_module.path(), &["src/*.anvil"]), "native");
    assert!(!output.status.success());
    let message = error(&output);
    assert!(message.contains("Main.anvil:1:1"), "{message}");
    assert!(message.contains("module 'Missing' not found"), "{message}");

    let missing_symbol = tempfile::tempdir().unwrap();
    write_file(
        missing_symbol.path().join("src/File.anvil"),
        "Open Nunction Present() {}",
    );
    write_file(
        missing_symbol.path().join("src/Main.anvil"),
        "Using File: Absent; Open Nunction Main() {}",
    );
    let output = build(&project(missing_symbol.path(), &["src/*.anvil"]), "native");
    assert!(!output.status.success());
    assert!(error(&output).contains("symbol 'Absent' not found in module 'File'"));

    let private = tempfile::tempdir().unwrap();
    write_file(
        private.path().join("src/File.anvil"),
        "Closed Nunction Hidden() {}",
    );
    write_file(
        private.path().join("src/Main.anvil"),
        "Use File; Open Nunction Main() { File.Hidden(); }",
    );
    let output = build(&project(private.path(), &["src/*.anvil"]), "native");
    assert!(!output.status.success());
    assert!(error(&output).contains("symbol 'Hidden' in module 'File' is private"));

    write_file(
        private.path().join("src/Main.anvil"),
        "Using File: Hidden; Open Nunction Main() { Hidden(); }",
    );
    let output = build(&project(private.path(), &["src/*.anvil"]), "native");
    assert!(!output.status.success());
    assert!(error(&output).contains("symbol 'Hidden' in module 'File' is private"));
}

#[test]
fn duplicate_imports_are_idempotent() {
    let temp = tempfile::tempdir().unwrap();
    write_file(
        temp.path().join("src/File.anvil"),
        "Open Int Value() { Return 11; }",
    );
    write_file(
        temp.path().join("src/Main.anvil"),
        "Use File; Use File; Using File: Value; Using File: Value; Open Int Main() { Return Value(); }",
    );
    let output = build(&project(temp.path(), &["src/*.anvil"]), "native");
    assert!(output.status.success(), "{}", error(&output));
    let status = run(&mut Command::new(temp.path().join("build/Imports"))).status;
    assert_eq!(status.code(), Some(11));
}

#[test]
fn selective_import_conflicts_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    write_file(temp.path().join("src/A.anvil"), "Open Nunction Run() {}");
    write_file(temp.path().join("src/B.anvil"), "Open Nunction Run() {}");
    write_file(
        temp.path().join("src/Main.anvil"),
        "Using A: Run; Using B: Run; Open Nunction Main() {}",
    );
    let output = build(&project(temp.path(), &["src/*.anvil"]), "native");
    assert!(!output.status.success());
    assert!(error(&output).contains("imported symbol 'Run' conflicts"));

    write_file(
        temp.path().join("src/Main.anvil"),
        "Using A: Run; Nunction Run() {} Open Nunction Main() {}",
    );
    let output = build(&project(temp.path(), &["src/*.anvil"]), "native");
    assert!(!output.status.success());
    assert!(error(&output).contains("conflicts with a declaration in module 'Main'"));
}

#[test]
fn duplicate_module_filenames_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    write_file(temp.path().join("src/File.anvil"), "Open Nunction One() {}");
    write_file(
        temp.path().join("tests/File.anvil"),
        "Open Nunction Two() {}",
    );
    write_file(
        temp.path().join("src/Main.anvil"),
        "Open Nunction Main() {}",
    );
    let output = build(
        &project(temp.path(), &["src/*.anvil", "tests/*.anvil"]),
        "native",
    );
    assert!(!output.status.success());
    assert!(error(&output).contains("duplicate module name 'File'"));
}

#[test]
fn nested_imports_resolve_without_leaking_their_scope() {
    let temp = tempfile::tempdir().unwrap();
    write_file(
        temp.path().join("src/Utils.anvil"),
        "Open Int Helper() { Return 19; }",
    );
    write_file(
        temp.path().join("src/Middle.anvil"),
        "Use Utils; Open Int Value() { Return Utils.Helper(); }",
    );
    write_file(
        temp.path().join("src/Main.anvil"),
        "Use Middle; Open Int Main() { Return Middle.Value(); }",
    );
    let project = project(temp.path(), &["src/*.anvil"]);
    let output = build(&project, "cranelift");
    assert!(output.status.success(), "{}", error(&output));
    let status = run(&mut Command::new(temp.path().join("build/Imports"))).status;
    assert_eq!(status.code(), Some(19));

    write_file(
        temp.path().join("src/Main.anvil"),
        "Use Middle; Open Int Main() { Return Utils.Helper(); }",
    );
    let output = build(&project, "native");
    assert!(!output.status.success());
}

#[test]
fn direct_and_long_circular_imports_are_rejected() {
    let direct = tempfile::tempdir().unwrap();
    write_file(direct.path().join("src/A.anvil"), "Use B;");
    write_file(direct.path().join("src/B.anvil"), "Use A;");
    write_file(
        direct.path().join("src/Main.anvil"),
        "Open Nunction Main() {}",
    );
    let output = build(&project(direct.path(), &["src/*.anvil"]), "native");
    assert!(!output.status.success());
    assert!(error(&output).contains("circular import detected: A -> B -> A"));

    let long = tempfile::tempdir().unwrap();
    write_file(long.path().join("src/A.anvil"), "Use B;");
    write_file(long.path().join("src/B.anvil"), "Use C;");
    write_file(long.path().join("src/C.anvil"), "Use A;");
    write_file(
        long.path().join("src/Main.anvil"),
        "Open Nunction Main() {}",
    );
    let output = build(&project(long.path(), &["src/*.anvil"]), "native");
    assert!(!output.status.success());
    assert!(error(&output).contains("circular import detected: A -> B -> C -> A"));
}

#[test]
fn project_imports_cannot_see_excluded_files() {
    let temp = tempfile::tempdir().unwrap();
    write_file(
        temp.path().join("src/Main.anvil"),
        "Use Secret; Open Nunction Main() {}",
    );
    write_file(
        temp.path().join("hidden/Secret.anvil"),
        "Open Nunction Hidden() {}",
    );
    let output = build(&project(temp.path(), &["src/*.anvil"]), "native");
    assert!(!output.status.success());
    assert!(error(&output).contains("module 'Secret' not found"));
}

#[test]
fn standalone_imports_resolve_relative_to_the_source_file() {
    let temp = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    write_file(
        temp.path().join("File.anvil"),
        "Open Int Add(Int A, Int B) { Return A + B; }",
    );
    write_file(
        temp.path().join("Main.anvil"),
        "Using File: Add; Open Int Main() { Return Add(20, 4); }",
    );

    for backend in ["native", "cranelift"] {
        let output = run(furnace().current_dir(elsewhere.path()).args([
            "compile",
            temp.path().join("Main.anvil").to_str().unwrap(),
            "linux",
            "--backend",
            backend,
        ]));
        assert!(output.status.success(), "{backend}: {}", error(&output));
        let status = run(&mut Command::new(elsewhere.path().join("Main"))).status;
        assert_eq!(status.code(), Some(24));
    }

    write_file(
        temp.path().join("Main.anvil"),
        "Use File; Open Int Main() { Return File.Add(20, 4); }",
    );
    let output = run(furnace().current_dir(elsewhere.path()).args([
        "compile",
        temp.path().join("Main.anvil").to_str().unwrap(),
        "linux",
        "--backend",
        "native",
    ]));
    assert!(output.status.success(), "{}", error(&output));
    let status = run(&mut Command::new(elsewhere.path().join("Main"))).status;
    assert_eq!(status.code(), Some(24));
}

#[test]
fn standalone_entry_filenames_do_not_have_to_be_import_identifiers() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("standalone-program.anvil");
    write_file(&source, "Open Nunction Main() {}");
    let output = run(furnace().current_dir(temp.path()).args([
        "compile",
        source.to_str().unwrap(),
        "linux",
        "--backend",
        "native",
    ]));
    assert!(output.status.success(), "{}", error(&output));
    assert!(temp.path().join("standalone-program").is_file());
}

#[test]
fn qualified_module_paths_resolve_structurally() {
    let temp = tempfile::tempdir().unwrap();
    write_file(
        temp.path().join("src/System/Math.anvil"),
        "Open Int Add(Int A, Int B) { Return A + B; }",
    );
    write_file(
        temp.path().join("src/Main.anvil"),
        "Use System.Math; Open Int Main() { Return System.Math.Add(3, 6); }",
    );
    let output = build(
        &project(temp.path(), &["src/*.anvil", "src/System/*.anvil"]),
        "native",
    );
    assert!(output.status.success(), "{}", error(&output));
    let status = run(&mut Command::new(temp.path().join("build/Imports"))).status;
    assert_eq!(status.code(), Some(9));

    write_file(
        temp.path().join("src/Main.anvil"),
        "Use System.Math; Open Int Main() { Return Math.Add(3, 6); }",
    );
    let output = build(
        &project(temp.path(), &["src/*.anvil", "src/System/*.anvil"]),
        "native",
    );
    assert!(!output.status.success());
}

#[test]
fn using_can_select_existing_non_function_symbols() {
    let temp = tempfile::tempdir().unwrap();
    write_file(
        temp.path().join("src/Types.anvil"),
        "Open Data Point { Int X; }",
    );
    write_file(
        temp.path().join("src/Main.anvil"),
        "Using Types: Point; Open Nunction Main() { Point P { X = 3; } Print(P.X); }",
    );
    let project = project(temp.path(), &["src/*.anvil"]);

    for backend in ["native", "cranelift"] {
        let output = build(&project, backend);
        assert!(output.status.success(), "{backend}: {}", error(&output));
        let executed = run(&mut Command::new(temp.path().join("build/Imports")));
        assert!(executed.status.success());
        assert_eq!(String::from_utf8_lossy(&executed.stdout), "3\n");
    }
}

#[test]
fn furnace_run_executes_a_project_with_imports() {
    let temp = tempfile::tempdir().unwrap();
    write_file(
        temp.path().join("src/File.anvil"),
        "Open Nunction Hello() { Print(\"hello-import\"); }",
    );
    write_file(
        temp.path().join("src/Main.anvil"),
        "Use File; Open Nunction Main() { File.Hello(); }",
    );
    let project = project(temp.path(), &["src/*.anvil"]);
    let output = run(furnace().args(["run", project.to_str().unwrap(), "--backend", "cranelift"]));
    assert!(output.status.success(), "{}", error(&output));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello-import\n");
}
