# jObfuscator

![CI](https://github.com/Ledmington/jobfuscator/actions/workflows/ci.yaml/badge.svg)

A zero-dependency Java class file obfuscator written in Rust.

Note: at the moment, this project is not entirely zero-dependency. The target, however, is to have zero dependencies in production code.

## How to use
This project produces two executables:
- `jobf`: the Java class file obfuscator
- `javap`: mimicks OpenJDK's `javap` utility to display class files information

At the moment this project supports only Java 25 class files.

### `jobf`
You can use the obfuscator either on a whole `.jar` file (the most common case) or on a single `.class` file (mostly for testing) like so (the command line is the same):
```
jobf -i my-app.jar -o my-shadowed-app.jar
```

Command-line options:
```
 -h, --help                    Prints this message and exits.
 -i, --input                   The file to read from.
 -o, --output                  The file to write to.
 -f, --force                   When enabled, overwrites the output file if it already exists.
 -q, --quiet                   Avoids printing on stdout.
 -s, --seed                    64-bit seed for RNG-based transformations (accepts hexadecimal and decimal).
     --make-everything-public  Converts all classes, fields and methods to public.
     --shuffle-fields          Shuffles the fields inside a class.
```

It's a strong requirement that the produced jar file must have the same behavior of the input jar file, so if happen to find a case in which behavior is modified, please open an issue.

However, it must be noted that it is not a goal of this project to support/handle reflection.

### `javap`
An executable to mimick OpenJDK's `javap` utility to print the class file contents in human-readable way on the terminal. This executable exists mainly to test the correctness of the class file parser.

You can use it like so:
```
./target/debug/javap Example.class
```
and its output will be identical to:
```
javap -l -v -p Example.class
```

It's a strong requirement that the output of these two commands must be identical, so if you happen to find a file which produces different outputs, please open an issue.

## How to build
You need Rust >= 1.88.0.

```
cargo build --bin jobf --release
```

## How to contribute
Compile all targets:
```
cargo build
```

Run unit tests:
```
cargo test
```

Run end-to-end integration tests:
```
./test/run_e2e_tests.sh
```

Linting
```
cargo clippy --all-targets --all-features
```

Documentation
```
cargo doc
```

This project supports Rust >= 1.88.0.
