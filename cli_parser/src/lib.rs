#![forbid(unsafe_code)]

use std::env::Args;

pub struct CommandLineParser {
    program_name: String,
    description: Option<String>,
    options: Vec<CommandLineOption>,
}

#[derive(Clone)]
pub struct CommandLineOption {
    short_name: Option<String>,
    long_name: Option<String>,
    option_type: CommandLineType,
}

#[derive(Clone)]
pub enum CommandLineType {
    Boolean { default_value: Option<bool> },
    String { default_value: Option<String> },
}

impl CommandLineOption {
    pub fn new(
        short_name: Option<String>,
        long_name: Option<String>,
        option_type: CommandLineType,
    ) -> Self {
        if short_name.is_none() && long_name.is_none() {
            panic!("A command line option must have at least a short name or a long name.");
        }
        if short_name.is_some() && long_name.is_some() && short_name == long_name {
            panic!(
                "This command line option has identical short name and long name: '{}'.",
                short_name.unwrap()
            );
        }
        CommandLineOption {
            short_name,
            long_name,
            option_type,
        }
    }
}

impl CommandLineParser {
    pub fn new(
        program_name: &str,
        description: Option<String>,
        options: Vec<CommandLineOption>,
    ) -> Self {
        let mut actual_options: Vec<CommandLineOption> = options.clone();

        // Adding the default '-h'/'--help' option
        actual_options.push(CommandLineOption {
            short_name: Some("-h".to_string()),
            long_name: Some("--help".to_string()),
            option_type: CommandLineType::Boolean {
                default_value: Some(false),
            },
        });

        for i in 0..actual_options.len() {
            for j in (i + 1)..actual_options.len() {
                let a = &actual_options[i];
                let b = &actual_options[j];
                if a.short_name.is_some() && b.short_name.is_some() && a.short_name == b.short_name
                {
                    panic!("Options {i} and {j} have the same short name.");
                }
                if a.long_name.is_some() && b.long_name.is_some() && a.long_name == b.long_name {
                    panic!("Options {i} and {j} have the same long name.");
                }
            }
        }

        CommandLineParser {
            program_name: program_name.to_string(),
            description,
            options: actual_options,
        }
    }

    pub fn parse(&self, args: &Args) -> Arguments {
        todo!()
    }
}

pub struct Arguments {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_boolean_option() {
        let parser = CommandLineParser::new(
            "test-parser",
            Some("A parser for testing".to_string()),
            vec![CommandLineOption {
                short_name: Some("-q".to_string()),
                long_name: Some("--quiet".to_string()),
                option_type: CommandLineType::Boolean {
                    default_value: Some(false),
                },
            }],
        );

        for input in [
            "-q",
            "-q 1",
            "-q true",
            "-q=1",
            "-q=true",
            "--quiet",
            "--quiet 1",
            "--quiet true",
            "--quiet=1",
            "--quiet=true",
        ] {
            let args = parser.parse(Args);
        }
    }
}
