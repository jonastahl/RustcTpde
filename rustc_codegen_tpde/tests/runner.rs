#![feature(exit_status_error)]

use libtest_mimic::{Arguments, Trial, Failed};
use pretty_assertions::assert_eq;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};

fn main() {
    let args = Arguments::from_args();
    let mut tests = Vec::new();

    {
        let test_cases_dir = Path::new("tests/lib_cases");
        for entry in fs::read_dir(test_cases_dir)
            .expect("Failed to read test cases directory") {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                let test_name = entry.file_name().into_string().unwrap();

                tests.push(Trial::test(test_name, move || run_lib_case(&path)));
            }
        }
    }
    {
        let test_cases_dir = Path::new("tests/exec_cases");
        for entry in fs::read_dir(test_cases_dir)
            .expect("Failed to read test cases directory") {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                let test_name = entry.file_name().into_string().unwrap();

                tests.push(Trial::test(test_name, move || run_exec_case(&path)));
            }
        }
    }


    libtest_mimic::run(&args, tests).exit();
}

fn run_lib_case(path: &Path) -> Result<(), libtest_mimic::Failed> {
    let overflow = !path.file_name().map_or_else(|| false, |s| s.to_str().unwrap().starts_with("nof_"));

    // 1. Resolve paths
    let source_path = path.join("source.rs");
    let expected_ir_path = path.join("ir");
    let expected_asm_path = path.join("asm");
    let actual_ir_path = path.join("actual_ir");
    let actual_obj_path = path.join("actual_obj");
    let actual_asm_path = path.join("actual_asm");

    let test_path = path.join("test.rs");
    let test_runner = path.join("test_runner");

    // 2. Read files (if expected output files don't exist yet, default to empty string)
    fs::exists(&source_path)
        .map_err(|_| Failed::from("Could not find source"))?;

    // 3. Compile the files
    rustc()
        .arg("--crate-type")
        .arg("lib")
        .arg("-Z")
        .arg("codegen-backend=../target/debug/librustc_codegen_tpde_dylib.so")
        .arg("-C")
        .arg(format!("overflow-checks={}", if overflow { "yes" } else { "no" }))
        .arg(&source_path)
        .arg("-o")
        .arg(&actual_ir_path)
        .arg("--emit")
        .arg("llvm-ir")
        .output() // Executes the command and captures stdout/stderr
        .map_err(|e| format!("Failed to execute rustc command: {}", e))?;
    rustc()
        .arg("--crate-type")
        .arg("lib")
        .arg("-Z")
        .arg("codegen-backend=../target/debug/librustc_codegen_tpde_dylib.so")
        .arg("-C")
        .arg(format!("overflow-checks={}", if overflow { "yes" } else { "no" }))
        .arg(&source_path)
        .arg("-o")
        .arg(&actual_obj_path)
        .output() // Executes the command and captures stdout/stderr
        .map_err(|e| format!("Failed to execute rustc command: {}", e))?;

    let asm_file = File::create(&actual_asm_path)
        .expect("Failed to create assembly output file");
    // Raw bytes and addresses are left out, as their layout (line wrapping, symbol
    // annotations) differs between binutils versions and shifts with every size change.
    Command::new("objdump")
        .arg("-d")
        .arg("--no-show-raw-insn")
        .arg("--no-addresses")
        .arg(&actual_obj_path)
        .stdout(Stdio::from(asm_file))
        .status()
        .expect("Failed to execute objdump");

    for entry in fs::read_dir(&path).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(path).expect("Failed to delete file");
        }
    }

    let expected_ir = fs::read_to_string(&expected_ir_path)
        .map_err(|_| eprintln!("Could not find expected ir")).unwrap_or_default();
    let expected_result = fs::read_to_string(&expected_asm_path)
        .map_err(|_| eprintln!("Could not find expected asm")).unwrap_or_default();
    let actual_ir = fs::read_to_string(&actual_ir_path)
        .map_err(|_| eprintln!("Could not find generated ir")).unwrap_or_default();
    let actual_result = fs::read_to_string(&actual_asm_path)
        .map_err(|_| eprintln!("Could not find generated asm")).unwrap_or_default();

    let expected_result = normalize_asm(&expected_result);
    let actual_result = normalize_asm(&actual_result);

    fs::rename(&actual_ir_path, &expected_ir_path)?;
    fs::remove_file(&actual_asm_path)?;
    fs::write(&expected_asm_path, &actual_result)?;

    // 4. Compare the outputs
    if actual_ir.trim() != expected_ir.trim() {
        return Err(Failed::from(format!(
            "IR mismatch in {:?}\n{}", path, reduced_diff(expected_ir.trim(), actual_ir.trim())
        )));
    } else if actual_result != expected_result {
        return Err(Failed::from(format!(
            "Result mismatch in {:?}\n{}", path, reduced_diff(&expected_result, &actual_result)
        )));
    }

    // 5. Compile the test files
    if test_path.try_exists().unwrap_or(false) {
        rustc()
            .arg(&test_path)
            .arg("-C")
            .arg(format!("link-arg={}", actual_obj_path.to_str().unwrap()))
            .arg("-o")
            .arg(&test_runner)
            .status()
            .expect("Failed to link file")
            .exit_ok()
            .expect("Compilation failed");
        Command::new(&test_runner)
            .status()
            .expect("Files seems to be not correct")
            .exit_ok()
            .expect("Test runner failed");
        fs::remove_file(&test_runner)
            .map_err(|_| eprintln!("Could not find test runner")).unwrap_or_default();
    } else {
        panic!("Test runner not found")
    }

    fs::remove_file(&actual_obj_path)
        .map_err(|_| eprintln!("Deleting obj failed")).unwrap_or_default();

    Ok(())
}

fn run_exec_case(path: &Path) -> Result<(), libtest_mimic::Failed> {
    let overflow = !path.file_name().map_or_else(|| false, |s| s.to_str().unwrap().starts_with("nof_"));

    let source_path = path.join("source.rs");
    let actual_bin_path = path.join("test_bin");
    let expected_output_path = path.join("output.txt");
    let input_path = path.join("input.txt"); // 1. Add input path

    // 1. Ensure source file exists
    if !source_path.exists() {
        return Err(Failed::from("Could not find source.rs"));
    }

    // 2. Compile the binary using the custom codegen backend
    rustc()
        .arg("--crate-type")
        .arg("bin")
        .arg("-Z")
        .arg("codegen-backend=../target/debug/librustc_codegen_tpde_dylib.so")
        .arg("-C")
        .arg(format!("overflow-checks={}", if overflow { "yes" } else { "no" }))
        .arg(&source_path)
        .arg("-o")
        .arg(&actual_bin_path)
        .status()
        .expect("Failed to execute rustc command")
        .exit_ok()
        .map_err(|_| Failed::from("Compilation of executable failed"))?;

    // 3. Execute the compiled binary and capture stdout/stderr
    let mut cmd = Command::new(&actual_bin_path);

    // Check if input.txt exists, and if so, pipe it to stdin
    if input_path.exists() {
        let input_file = File::open(&input_path)
            .map_err(|e| Failed::from(format!("Failed to open input.txt: {}", e)))?;
        cmd.stdin(Stdio::from(input_file));
    }

    let execution_output = cmd
        .output()
        .expect("Failed to run the compiled binary");

    // Fail if the executable crashed or panicked, printing stderr for debugging
    execution_output.status.exit_ok().map_err(|_| {
        let stderr = String::from_utf8_lossy(&execution_output.stderr);
        Failed::from(format!("Executable returned a non-zero exit code. Stderr:\n{}", stderr))
    })?;

    // 4. Compare output if output.txt is provided
    if expected_output_path.exists() {
        let expected_output = fs::read_to_string(&expected_output_path)
            .map_err(|e| Failed::from(format!("Failed to read output.txt: {}", e)))?;

        let actual_output = String::from_utf8_lossy(&execution_output.stdout);

        // Trimming both ends before comparison avoids trivial test failures
        // due to trailing newlines or Windows/Unix line ending mismatches (\r\n vs \n).
        if actual_output.trim() != expected_output.trim() {
            assert_eq!(
                expected_output.trim(),
                actual_output.trim(),
                "Output mismatch in {:?}", path
            );
        }
    }

    // 5. Cleanup
    fs::remove_file(&actual_bin_path)
        .map_err(|_| eprintln!("Deleting executable failed")).unwrap_or_default();

    Ok(())
}

fn rustc() -> Command {
    let sysroot = Command::new("rustc")
        .arg("+nightly-2026-08-19")
        .arg("--print")
        .arg("sysroot")
        .output()
        .expect("Failed to query sysroot");
    let sysroot = String::from_utf8(sysroot.stdout).expect("Invalid sysroot");

    let mut cmd = Command::new("rustc");
    cmd.arg("+nightly-2026-08-19")
        .arg(format!("--remap-path-prefix={}=/sysroot", sysroot.trim()));
    cmd
}

/// Newer binutils print implicit segment registers of string instructions
/// (e.g. `insl (%dx),%es:(%rdi)`), older ones don't.
fn normalize_asm(asm: &str) -> String {
    asm.replace("%es:(%rdi)", "(%rdi)").replace("%ds:(%rsi)", "(%rsi)")
}

/// Unified diff that only shows the changed hunks with a few lines of context.
fn reduced_diff(expected: &str, actual: &str) -> String {
    similar::TextDiff::from_lines(expected, actual)
        .unified_diff()
        .context_radius(3)
        .header("expected", "actual")
        .to_string()
}
