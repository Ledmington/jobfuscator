#![forbid(unsafe_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use tempfile::TempDir;

const GREEN: &str = "\x1b[0;32m";
const RED: &str = "\x1b[0;31m";
const RESET: &str = "\x1b[0m";

struct TestCase {
    name: &'static str,
    executable: bool,
}

const TEST_CASES: &[TestCase] = &[
    TestCase {
        name: "Empty",
        executable: false,
    },
    TestCase {
        name: "HelloWorld",
        executable: true,
    },
    TestCase {
        name: "Math",
        executable: false,
    },
    TestCase {
        name: "Stream",
        executable: false,
    },
    TestCase {
        name: "List",
        executable: false,
    },
    TestCase {
        name: "TimeUnit",
        executable: false,
    },
    TestCase {
        name: "Arrays",
        executable: false,
    },
    TestCase {
        name: "SecuritySettings$1",
        executable: false,
    },
    TestCase {
        name: "Employee",
        executable: true,
    },
    TestCase {
        name: "Calculator",
        executable: true,
    },
    TestCase {
        name: "Object",
        executable: false,
    },
];

struct TestEnv {
    test_dir: PathBuf,
    system_java: PathBuf,
    system_javap: PathBuf,
    our_javap: PathBuf,
    jobf: PathBuf,
}

impl TestEnv {
    fn new(build_mode: &str) -> Self {
        let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let target = test_dir.parent().unwrap().join("target").join(build_mode);

        Self {
            system_java: which("java"),
            system_javap: which("javap"),
            our_javap: target.join("javap"),
            jobf: target.join("jobf"),
            test_dir,
        }
    }

    fn data_dir(&self) -> PathBuf {
        self.test_dir.join("data")
    }

    fn class_file(&self, name: &str) -> PathBuf {
        self.data_dir().join(format!("{name}.class"))
    }

    fn tmp_class_file(&self, tmp: &TempDir, name: &str) -> PathBuf {
        tmp.path().join(format!("{name}.class"))
    }
}

fn which(binary: &str) -> PathBuf {
    let output = Command::new("which")
        .arg(binary)
        .output()
        .unwrap_or_else(|_| panic!("failed to run 'which {binary}'"));
    assert!(output.status.success(), "'which {binary}' failed");
    PathBuf::from(String::from_utf8(output.stdout).unwrap().trim())
}

/// Formats a Command's program and arguments as a shell-like string for display.
fn format_cmd(cmd: &Command) -> String {
    let prog = cmd.get_program().to_string_lossy().into_owned();
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| {
            let s = a.to_string_lossy();
            if s.contains(' ') {
                format!("'{s}'")
            } else {
                s.into_owned()
            }
        })
        .collect();
    if args.is_empty() {
        prog
    } else {
        format!("{prog} {}", args.join(" "))
    }
}

/// Runs a command and returns its output, along with the formatted command line.
fn run(cmd: &mut Command) -> (Output, String) {
    let cmdline = format_cmd(cmd);
    let output = cmd
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn {:?}: {}", cmd.get_program(), e));
    (output, cmdline)
}

fn jobf_shuffle(env: &TestEnv, input: &Path, output: &Path, flag: &str) -> Result<(), String> {
    let (out, cmdline) = run(Command::new(&env.jobf)
        .arg("--input")
        .arg(input)
        .arg("--output")
        .arg(output)
        .arg("--quiet=true")
        .arg("--seed=0x01020304")
        .arg(flag)
        .arg("--force"));

    if out.status.success() {
        Ok(())
    } else {
        Err(format_output_detail(&cmdline, &out))
    }
}

/// Runs `diff` on two byte slices by writing them to temp files, prints the
/// output, and returns true if they differ.
fn diff_outputs(expected: &[u8], actual: &[u8], label_expected: &str, label_actual: &str) -> bool {
    use std::io::Write;

    let mut expected_file = tempfile::NamedTempFile::new().unwrap();
    let mut actual_file = tempfile::NamedTempFile::new().unwrap();
    expected_file.write_all(expected).unwrap();
    actual_file.write_all(actual).unwrap();

    let output = Command::new("diff")
        .arg("--label")
        .arg(label_expected)
        .arg("--label")
        .arg(label_actual)
        .arg("-u")
        .arg(expected_file.path())
        .arg(actual_file.path())
        .output()
        .expect("failed to run 'diff'");

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }

    !output.status.success()
}

/// Formats the command line and any stdout/stderr from an Output for display after a failure.
fn format_output_detail(cmdline: &str, output: &Output) -> String {
    let mut detail = format!("    cmd: {cmdline}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.trim().is_empty() {
        for line in stdout.trim_end().lines() {
            detail.push_str(&format!("\n    stdout: {line}"));
        }
    }
    if !stderr.trim().is_empty() {
        for line in stderr.trim_end().lines() {
            detail.push_str(&format!("\n    stderr: {line}"));
        }
    }
    detail
}

fn ok(name: &str) {
    println!("  {name} ... {GREEN}OK{RESET}");
}

fn fail(failures: &mut Vec<String>, name: &str, reason: &str) {
    let short = reason.lines().next().unwrap_or(reason);
    failures.push(format!("{name}: {short}"));
    println!("  {name} ... {RED}FAILED{RESET}");
    println!("{reason}");
}

fn run_javap_tests(env: &TestEnv) -> Vec<String> {
    println!("\nJavap output tests");

    let mut failures = vec![];

    for case in TEST_CASES {
        let class_file = env.class_file(case.name);

        let (expected_out, expected_cmd) = run(Command::new(&env.system_javap)
            .args(["-l", "-v", "-p"])
            .arg(&class_file));

        let (actual_out, actual_cmd) = run(Command::new(&env.our_javap).arg(&class_file));

        if !expected_out.status.success() {
            fail(
                &mut failures,
                case.name,
                &format!(
                    "system javap failed\n{}",
                    format_output_detail(&expected_cmd, &expected_out)
                ),
            );
        } else if !actual_out.status.success() {
            fail(
                &mut failures,
                case.name,
                &format!(
                    "our javap failed\n{}",
                    format_output_detail(&actual_cmd, &actual_out)
                ),
            );
        } else if diff_outputs(
            &expected_out.stdout,
            &actual_out.stdout,
            "system javap",
            "our javap",
        ) {
            fail(
                &mut failures,
                case.name,
                &format!("output differs from system javap\n    cmd: {actual_cmd}"),
            );
        } else {
            ok(case.name);
        }
    }

    failures
}

fn run_roundtrip_tests(env: &TestEnv) -> Vec<String> {
    println!("\nEncoding roundtrip tests");

    let mut failures = vec![];

    for case in TEST_CASES {
        let class_file = env.class_file(case.name);
        let tmp = TempDir::new().unwrap();
        let output_file = env.tmp_class_file(&tmp, case.name);

        let (out, cmdline) = run(Command::new(&env.jobf)
            .arg("--input")
            .arg(&class_file)
            .arg("--output")
            .arg(&output_file)
            .arg("--quiet=true")
            .arg("--force"));

        if !out.status.success() {
            fail(
                &mut failures,
                case.name,
                &format!(
                    "jobf exited with {}\n{}",
                    out.status,
                    format_output_detail(&cmdline, &out)
                ),
            );
            continue;
        }

        if fs::read(&class_file).unwrap() != fs::read(&output_file).unwrap() {
            fail(
                &mut failures,
                case.name,
                &format!("roundtrip produced different bytes\n    cmd: {cmdline}"),
            );
        } else {
            ok(case.name);
        }
    }

    failures
}

fn run_shuffle_javap_tests(env: &TestEnv, label: &str, shuffle_flag: &str) -> Vec<String> {
    println!("\nEncoding after {label} roundtrip tests");

    let mut failures = vec![];

    for case in TEST_CASES {
        let class_file = env.class_file(case.name);
        let tmp = TempDir::new().unwrap();
        let shuffled = env.tmp_class_file(&tmp, case.name);

        if let Err(detail) = jobf_shuffle(env, &class_file, &shuffled, shuffle_flag) {
            fail(
                &mut failures,
                case.name,
                &format!("jobf failed during {label}\n{detail}"),
            );
            continue;
        }

        let (expected_out, expected_cmd) = run(Command::new(&env.system_javap)
            .args(["-l", "-v", "-p"])
            .arg(&shuffled));

        let (actual_out, actual_cmd) = run(Command::new(&env.our_javap).arg(&shuffled));

        if !expected_out.status.success() {
            fail(
                &mut failures,
                case.name,
                &format!(
                    "system javap failed after {label}\n{}",
                    format_output_detail(&expected_cmd, &expected_out)
                ),
            );
        } else if !actual_out.status.success() {
            fail(
                &mut failures,
                case.name,
                &format!(
                    "our javap failed after {label}\n{}",
                    format_output_detail(&actual_cmd, &actual_out)
                ),
            );
        } else if diff_outputs(
            &expected_out.stdout,
            &actual_out.stdout,
            "system javap",
            "our javap",
        ) {
            fail(
                &mut failures,
                case.name,
                &format!("javap output differs after {label}\n    cmd: {actual_cmd}"),
            );
        } else {
            ok(case.name);
        }
    }

    failures
}

fn run_shuffle_execution_tests(env: &TestEnv, label: &str, shuffle_flag: &str) -> Vec<String> {
    println!("\nExecution after {label} tests");

    let mut failures = vec![];

    for case in TEST_CASES.iter().filter(|c| c.executable) {
        let class_file = env.class_file(case.name);
        let tmp = TempDir::new().unwrap();
        let shuffled = env.tmp_class_file(&tmp, case.name);

        let (original, original_cmd) = run(Command::new(&env.system_java)
            .arg("-cp")
            .arg(env.data_dir())
            .arg(case.name));

        if !original.status.success() {
            fail(
                &mut failures,
                case.name,
                &format!(
                    "original class is not executable\n{}",
                    format_output_detail(&original_cmd, &original)
                ),
            );
            continue;
        }

        if let Err(detail) = jobf_shuffle(env, &class_file, &shuffled, shuffle_flag) {
            fail(
                &mut failures,
                case.name,
                &format!("jobf failed during {label}\n{detail}"),
            );
            continue;
        }

        let (modified, modified_cmd) = run(Command::new(&env.system_java)
            .arg("-cp")
            .arg(tmp.path())
            .arg(case.name));

        if !modified.status.success() {
            fail(
                &mut failures,
                case.name,
                &format!(
                    "modified class is not executable\n{}",
                    format_output_detail(&modified_cmd, &modified)
                ),
            );
            continue;
        }

        if diff_outputs(&original.stdout, &modified.stdout, "original", "modified") {
            fail(
                &mut failures,
                case.name,
                &format!("output differs after {label}\n    cmd: {modified_cmd}"),
            );
        } else {
            ok(case.name);
        }
    }

    failures
}

fn main() {
    let build_mode = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "debug".to_string());

    if build_mode != "debug" && build_mode != "release" {
        eprintln!("Usage: e2e [debug|release]");
        std::process::exit(1);
    }

    let env = TestEnv::new(&build_mode);

    println!(" Build mode:     {build_mode}");
    println!(" System's java:  {}", env.system_java.display());
    println!(" System's javap: {}", env.system_javap.display());
    println!(" Tested javap:   {}", env.our_javap.display());
    println!(" Tested jobf:    {}", env.jobf.display());

    let mut failures: Vec<String> = vec![];
    failures.extend(run_javap_tests(&env));
    failures.extend(run_roundtrip_tests(&env));
    failures.extend(run_shuffle_javap_tests(
        &env,
        "field-shuffle",
        "--shuffle-fields=true",
    ));
    failures.extend(run_shuffle_execution_tests(
        &env,
        "field-shuffle",
        "--shuffle-fields=true",
    ));
    failures.extend(run_shuffle_javap_tests(
        &env,
        "method-shuffle",
        "--shuffle-methods=true",
    ));
    failures.extend(run_shuffle_execution_tests(
        &env,
        "method-shuffle",
        "--shuffle-methods=true",
    ));
    failures.extend(run_shuffle_javap_tests(
        &env,
        "constant-pool-shuffle",
        "--shuffle-constant-pool=true",
    ));
    failures.extend(run_shuffle_execution_tests(
        &env,
        "constant_pool-shuffle",
        "--shuffle-constant-pool=true",
    ));

    if failures.is_empty() {
        println!("\n{GREEN}All tests passed.{RESET}");
    } else {
        println!("\n{}{} failure(s):{}", RED, failures.len(), RESET);
        for f in &failures {
            println!("  - {f}");
        }
        std::process::exit(1);
    }
}
