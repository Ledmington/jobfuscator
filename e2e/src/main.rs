#![forbid(unsafe_code)]

use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};
use tempfile::TempDir;

struct TestCase {
    name: &'static str,
    executable: bool,
}

const TEST_CASES: &[TestCase] = &[
    TestCase {
        name: "HelloWorld",
        executable: false,
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
        let root = test_dir.parent().unwrap().to_path_buf();
        let target = root.join("target").join(build_mode);

        let system_java = which("java");
        let system_javap = which("javap");
        let our_javap = target.join("javap");
        let jobf = target.join("jobf");

        Self {
            test_dir,
            system_java,
            system_javap,
            our_javap,
            jobf,
        }
    }

    fn data_dir(&self) -> PathBuf {
        self.test_dir.join("data")
    }

    fn class_file(&self, name: &str) -> PathBuf {
        self.data_dir().join(format!("{}.class", name))
    }
}

fn which(binary: &str) -> PathBuf {
    let output = Command::new("which")
        .arg(binary)
        .output()
        .unwrap_or_else(|_| panic!("failed to run 'which {}'", binary));
    assert!(output.status.success(), "'which {}' failed", binary);
    PathBuf::from(String::from_utf8(output.stdout).unwrap().trim())
}

fn run(cmd: &mut Command) -> Output {
    cmd.output()
        .unwrap_or_else(|e| panic!("failed to spawn {:?}: {}", cmd.get_program(), e))
}

fn run_javap_tests(env: &TestEnv) -> Vec<String> {
    println!("\nEnd-to-End javap tests");
    println!("  system javap: {}", env.system_javap.display());
    println!("  our javap:    {}", env.our_javap.display());

    let mut failures = vec![];

    for case in TEST_CASES {
        let name = case.name;

        let class_file = env.class_file(name);

        let expected = run(Command::new(&env.system_javap)
            .args(["-l", "-v", "-p"])
            .arg(&class_file))
        .stdout;

        let actual = run(Command::new(&env.our_javap).arg(&class_file)).stdout;

        if expected != actual {
            let diff = diff_text(&expected, &actual);
            failures.push(format!(
                "{}: output differs from system javap\n{}",
                name, diff
            ));
            println!("  {} ... FAILED", name);
        } else {
            println!("  {} ... ok", name);
        }
    }

    failures
}

fn run_roundtrip_tests(env: &TestEnv) -> Vec<String> {
    println!("\nEnd-to-End roundtrip tests");
    println!("  jobf: {}", env.jobf.display());

    let mut failures = vec![];

    for case in TEST_CASES {
        let name = case.name;

        let class_file = env.class_file(name);
        let tmp = TempDir::new().unwrap();
        let output_file = tmp.path().join(format!("{}.class", name));

        let jobf_status = run(Command::new(&env.jobf)
            .arg("--input")
            .arg(&class_file)
            .arg("--output")
            .arg(&output_file)
            .arg("--quiet=true")
            .arg("--force"))
        .status;

        if !jobf_status.success() {
            failures.push(format!("{}: jobf exited with {}", name, jobf_status));
            println!("  {} ... FAILED", name);
            continue;
        }

        let expected = fs::read(&class_file).unwrap();
        let actual = fs::read(&output_file).unwrap();

        if expected != actual {
            failures.push(format!("{}: roundtrip produced different bytes", name));
            println!("  {} ... FAILED", name);
        } else {
            println!("  {} ... ok", name);
        }
    }

    failures
}

fn run_field_shuffle_tests(env: &TestEnv) -> Vec<String> {
    println!("\nField-shuffle tests");
    println!("  system javap: {}", env.system_javap.display());
    println!("  our javap:    {}", env.our_javap.display());
    println!("  jobf:         {}", env.jobf.display());

    let mut failures = vec![];

    for case in TEST_CASES {
        let name = case.name;

        let class_file = env.class_file(name);
        let tmp = TempDir::new().unwrap();
        let shuffled = tmp.path().join(format!("{}.class", name));

        let jobf_status = run(Command::new(&env.jobf)
            .arg("--input")
            .arg(&class_file)
            .arg("--output")
            .arg(&shuffled)
            .arg("--quiet=true")
            .arg("--seed=0x01020304")
            .arg("--shuffle-fields=true")
            .arg("--force"))
        .status;

        if !jobf_status.success() {
            failures.push(format!("{}: jobf exited with {}", name, jobf_status));
            println!("  {} ... FAILED", name);
            continue;
        }

        let expected = run(Command::new(&env.system_javap)
            .args(["-l", "-v", "-p"])
            .arg(&shuffled))
        .stdout;

        let actual = run(Command::new(&env.our_javap).arg(&shuffled)).stdout;

        if expected != actual {
            let diff = diff_text(&expected, &actual);
            failures.push(format!(
                "{}: javap output differs after field shuffle\n{}",
                name, diff
            ));
            println!("  {} ... FAILED", name);
        } else {
            println!("  {} ... ok", name);
        }
    }

    failures
}

fn run_execution_tests(env: &TestEnv) -> Vec<String> {
    println!("\nEnd-to-End execution tests");
    println!("  system java: {}", env.system_java.display());
    println!("  jobf:        {}", env.jobf.display());

    let mut failures = vec![];

    for case in TEST_CASES {
        if !case.executable {
            continue;
        }
        let name = case.name;

        let class_file = env.class_file(name);
        let tmp = TempDir::new().unwrap();
        let shuffled = tmp.path().join(format!("{}.class", name));

        // Original must be executable
        let original = run(Command::new(&env.system_java)
            .arg("-cp")
            .arg(env.data_dir())
            .arg(name));

        if !original.status.success() {
            failures.push(format!(
                "{}: original class is not executable (exit {:?})\n{}",
                name,
                original.status.code(),
                String::from_utf8_lossy(&original.stderr),
            ));
            println!("  {} ... FAILED", name);
            continue;
        }

        let jobf_status = run(Command::new(&env.jobf)
            .arg("--input")
            .arg(&class_file)
            .arg("--output")
            .arg(&shuffled)
            .arg("--quiet=true")
            .arg("--seed=0x01020304")
            .arg("--shuffle-fields=true")
            .arg("--force"))
        .status;

        if !jobf_status.success() {
            failures.push(format!("{}: jobf exited with {}", name, jobf_status));
            println!("  {} ... FAILED", name);
            continue;
        }

        let modified = run(Command::new(&env.system_java)
            .arg("-cp")
            .arg(tmp.path())
            .arg(name));

        if !modified.status.success() {
            failures.push(format!(
                "{}: modified class is not executable (exit {:?})\n{}",
                name,
                modified.status.code(),
                String::from_utf8_lossy(&modified.stderr),
            ));
            println!("  {} ... FAILED", name);
            continue;
        }

        if original.stdout != modified.stdout {
            let diff = diff_text(&original.stdout, &modified.stdout);
            failures.push(format!(
                "{}: output differs after field shuffle\n{}",
                name, diff
            ));
            println!("  {} ... FAILED", name);
        } else {
            println!("  {} ... ok", name);
        }
    }

    failures
}

fn diff_text(expected: &[u8], actual: &[u8]) -> String {
    let expected = String::from_utf8_lossy(expected);
    let actual = String::from_utf8_lossy(actual);
    let mut out = String::new();
    for line in expected.lines() {
        if !actual.contains(line) {
            out.push_str(&format!("- {}\n", line));
        }
    }
    for line in actual.lines() {
        if !expected.contains(line) {
            out.push_str(&format!("+ {}\n", line));
        }
    }
    out
}

fn main() {
    let build_mode = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "debug".to_string());
    if build_mode != "debug" && build_mode != "release" {
        eprintln!("Usage: e2e [debug|release]");
        std::process::exit(1);
    }

    println!("Build mode: {}", build_mode);
    let env = TestEnv::new(&build_mode);

    let mut all_failures: Vec<String> = vec![];
    all_failures.extend(run_javap_tests(&env));
    all_failures.extend(run_roundtrip_tests(&env));
    all_failures.extend(run_field_shuffle_tests(&env));
    all_failures.extend(run_execution_tests(&env));

    if !all_failures.is_empty() {
        println!("\n{} failure(s):", all_failures.len());
        for f in &all_failures {
            println!("  - {}", f);
        }
        std::process::exit(1);
    } else {
        println!("\nAll tests passed.");
    }
}
