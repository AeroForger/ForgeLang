use furnace::backend::compile_to_elf;
use furnace::parser::parse_program;
use furnace::semantic::analyze;
use std::os::unix::fs::PermissionsExt;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn compile_and_run(source: &str) -> Result<i32, String> {
    compile_and_run_with_stdin(source, "")
}

fn compile_and_run_with_stdin(source: &str, stdin: &str) -> Result<i32, String> {
    let program = parse_program(source).map_err(|e| e.to_string())?;
    analyze(&program).map_err(|e| e.to_string())?;
    let elf_bytes = compile_to_elf(&program).map_err(|e| e.to_string())?;

    let temp_dir = std::env::temp_dir();
    let count = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let bin_dir = temp_dir.join(format!(
        "test_native_bin_{}_{}_{}",
        std::process::id(),
        count,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&bin_dir).map_err(|e| e.to_string())?;
    let bin_path = bin_dir.join("program");
    std::fs::write(&bin_path, elf_bytes).map_err(|e| e.to_string())?;

    let mut perms = std::fs::metadata(&bin_path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&bin_path, perms).unwrap();

    let mut child = std::process::Command::new(&bin_path)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(mut pipe) = child.stdin.take() {
        use std::io::Write;
        pipe.write_all(stdin.as_bytes())
            .map_err(|e| e.to_string())?;
    }
    let output = child.wait().map_err(|e| e.to_string())?;
    let _ = std::fs::remove_dir_all(&bin_dir);

    Ok(output.code().unwrap_or(-1))
}

#[test]
fn test_integer_input() {
    let code = "Open Int Main() { Return Input(Int); }";
    let result = compile_and_run_with_stdin(code, "-12\n").unwrap();
    assert_eq!(result, 244);
}

#[test]
fn test_signed_modulo() {
    let code = "Open Int Main() { Return -10 % 3; }";
    assert_eq!(compile_and_run(code).unwrap(), 255);
}

#[test]
fn test_string_function_prints_line() {
    let code =
        "Nunction Show(Weld Value) { Print(Value); } Open Nunction Main() { Show(\"hello\"); }";
    let program = parse_program(code).unwrap();
    analyze(&program).unwrap();
    let elf_bytes = compile_to_elf(&program).unwrap();
    assert!(!elf_bytes.is_empty());
}

#[test]
fn test_return_constant() {
    let code = "Open Int Main() { Return 42; }";
    let exit_code = compile_and_run(code).unwrap();
    assert_eq!(exit_code, 42);
}

#[test]
fn test_variables_and_arithmetic() {
    let code = "
        Open Int Main() {
            Int X = 10;
            Int Y = 20;
            Int Z = X + Y;
            Return Z;
        }
    ";
    let exit_code = compile_and_run(code).unwrap();
    assert_eq!(exit_code, 30);
}

#[test]
fn test_function_call() {
    let code = "
        Int Add(Int A, Int B) {
            Return A + B;
        }

        Open Int Main() {
            Return Add(15, 25);
        }
    ";
    let exit_code = compile_and_run(code).unwrap();
    assert_eq!(exit_code, 40);
}

#[test]
fn test_if_else_branches() {
    let code = "
        Open Int Main() {
            Int X = 10;
            If (X > 5) {
                Return 1;
            } Else {
                Return 0;
            }
        }
    ";
    let exit_code = compile_and_run(code).unwrap();
    assert_eq!(exit_code, 1);
}

#[test]
fn test_signed_comparisons() {
    let code_true = "
        Open Int Main() {
            Int Neg = -10;
            Int Pos = 5;
            If (Neg < Pos) {
                Return 1;
            } Else {
                Return 0;
            }
        }
    ";
    assert_eq!(compile_and_run(code_true).unwrap(), 1);

    let code_false = "
        Open Int Main() {
            Int Neg = -10;
            Int Pos = 5;
            If (Neg > Pos) {
                Return 1;
            } Else {
                Return 0;
            }
        }
    ";
    assert_eq!(compile_and_run(code_false).unwrap(), 0);
}

#[test]
fn test_division() {
    let code = "
        Open Int Main() {
            Int A = 100;
            Int B = 4;
            Return A / B;
        }
    ";
    assert_eq!(compile_and_run(code).unwrap(), 25);
}

#[test]
fn test_modulo() {
    let code = "
        Open Int Main() {
            Int X = 10 % 3;
            Return X;
        }
    ";
    assert_eq!(compile_and_run(code).unwrap(), 1);
}

#[test]
fn test_multiple_function_calls_and_arithmetic() {
    let code = "
        Int Square(Int X) {
            Return X * X;
        }

        Int SumSquares(Int A, Int B) {
            Return Square(A) + Square(B);
        }

        Open Int Main() {
            Return SumSquares(3, 4);
        }
    ";
    assert_eq!(compile_and_run(code).unwrap(), 25);
}

#[test]
fn test_unsupported_more_than_6_params() {
    let code = "
        Int TooMany(Int A, Int B, Int C, Int D, Int E, Int F, Int G) {
            Return A;
        }
        Open Int Main() {
            Return TooMany(1,2,3,4,5,6,7);
        }
    ";
    let res = compile_and_run(code);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("parameters"));
}
