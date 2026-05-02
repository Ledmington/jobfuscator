#![forbid(unsafe_code)]

use std::{
    fs,
    path::PathBuf,
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

fn run(cmd: &mut Command) -> Output {
    cmd.output()
        .unwrap_or_else(|e| panic!("failed to spawn {:?}: {}", cmd.get_program(), e))
}

fn jobf_shuffle(env: &TestEnv, input: &PathBuf, output: &PathBuf) -> bool {
    run(Command::new(&env.jobf)
        .arg("--input")
        .arg(input)
        .arg("--output")
        .arg(output)
        .arg("--quiet=true")
        .arg("--seed=0x01020304")
        .arg("--shuffle-fields=true")
        .arg("--force"))
    .status
    .success()
}

fn diff_text(expected: &[u8], actual: &[u8]) -> String {
    let expected = String::from_utf8_lossy(expected);
    let actual = String::from_utf8_lossy(actual);
    let mut out = String::new();
    for line in expected.lines() {
        if !actual.contains(line) {
            out.push_str(&format!("{RED}  - {line}{RESET}\n"));
        }
    }
    for line in actual.lines() {
        if !expected.contains(line) {
            out.push_str(&format!("{GREEN}  + {line}{RESET}\n"));
        }
    }
    out
}

fn ok(name: &str) {
    println!("  {name} ... {GREEN}OK{RESET}");
}

fn fail(failures: &mut Vec<String>, name: &str, reason: &str) {
    failures.push(format!("{name}: {reason}"));
    println!("  {name} ... {RED}FAILED{RESET}");
}

fn run_javap_tests(env: &TestEnv) -> Vec<String> {
    println!("\njavap output tests");

    let mut failures = vec![];

    for case in TEST_CASES {
        let class_file = env.class_file(case.name);

        let expected = run(Command::new(&env.system_javap)
            .args(["-l", "-v", "-p"])
            .arg(&class_file))
        .stdout;

        let actual = run(Command::new(&env.our_javap).arg(&class_file)).stdout;

        if expected != actual {
            fail(
                &mut failures,
                case.name,
                &format!(
                    "output differs from system javap\n{}",
                    diff_text(&expected, &actual),
                ),
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

        let status = run(Command::new(&env.jobf)
            .arg("--input")
            .arg(&class_file)
            .arg("--output")
            .arg(&output_file)
            .arg("--quiet=true")
            .arg("--force"))
        .status;

        if !status.success() {
            fail(
                &mut failures,
                case.name,
                &format!("jobf exited with {status}"),
            );
            continue;
        }

        if fs::read(&class_file).unwrap() != fs::read(&output_file).unwrap() {
            fail(
                &mut failures,
                case.name,
                "roundtrip produced different bytes",
            );
        } else {
            ok(case.name);
        }
    }

    failures
}

fn run_field_shuffle_tests(env: &TestEnv) -> Vec<String> {
    println!("\nEncoding after field-shuffle roundtrip tests");

    let mut failures = vec![];

    for case in TEST_CASES {
        let class_file = env.class_file(case.name);
        let tmp = TempDir::new().unwrap();
        let shuffled = env.tmp_class_file(&tmp, case.name);

        if !jobf_shuffle(env, &class_file, &shuffled) {
            fail(&mut failures, case.name, "jobf failed during field shuffle");
            continue;
        }

        let expected = run(Command::new(&env.system_javap)
            .args(["-l", "-v", "-p"])
            .arg(&shuffled))
        .stdout;

        let actual = run(Command::new(&env.our_javap).arg(&shuffled)).stdout;

        if expected != actual {
            fail(
                &mut failures,
                case.name,
                &format!(
                    "javap output differs after field shuffle\n{}",
                    diff_text(&expected, &actual),
                ),
            );
        } else {
            ok(case.name);
        }
    }

    failures
}

fn run_execution_tests(env: &TestEnv) -> Vec<String> {
    println!("\nExecution after field-shuffle tests");

    let mut failures = vec![];

    for case in TEST_CASES.iter().filter(|c| c.executable) {
        let class_file = env.class_file(case.name);
        let tmp = TempDir::new().unwrap();
        let shuffled = env.tmp_class_file(&tmp, case.name);

        let original = run(Command::new(&env.system_java)
            .arg("-cp")
            .arg(env.data_dir())
            .arg(case.name));

        if !original.status.success() {
            fail(
                &mut failures,
                case.name,
                &format!(
                    "original class is not executable (exit {:?})\n{}",
                    original.status.code(),
                    String::from_utf8_lossy(&original.stderr),
                ),
            );
            continue;
        }

        if !jobf_shuffle(env, &class_file, &shuffled) {
            fail(&mut failures, case.name, "jobf failed during field shuffle");
            continue;
        }

        let modified = run(Command::new(&env.system_java)
            .arg("-cp")
            .arg(tmp.path())
            .arg(case.name));

        if !modified.status.success() {
            fail(
                &mut failures,
                case.name,
                &format!(
                    "modified class is not executable (exit {:?})\n{}",
                    modified.status.code(),
                    String::from_utf8_lossy(&modified.stderr),
                ),
            );
            continue;
        }

        if original.stdout != modified.stdout {
            fail(
                &mut failures,
                case.name,
                &format!(
                    "output differs after field shuffle\n{}",
                    diff_text(&original.stdout, &modified.stdout),
                ),
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

    println!("Build mode:   {build_mode}");
    println!("system java:  {}", env.system_java.display());
    println!("system javap: {}", env.system_javap.display());
    println!("our javap:    {}", env.our_javap.display());
    println!("jobf:         {}", env.jobf.display());

    let mut failures: Vec<String> = vec![];
    failures.extend(run_javap_tests(&env));
    failures.extend(run_roundtrip_tests(&env));
    failures.extend(run_field_shuffle_tests(&env));
    failures.extend(run_execution_tests(&env));

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
