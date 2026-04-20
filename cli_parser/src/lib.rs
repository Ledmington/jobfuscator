#![forbid(unsafe_code)]

use std::io::{self};
use std::{collections::HashMap, env::Args, fmt};

pub struct CommandLineParser {
    program_name: String,
    description: Option<String>,
    options: Vec<CommandLineOption>,
}

#[derive(Clone, Debug)]
pub struct CommandLineOption {
    short_name: Option<String>,
    long_name: Option<String>,
    description: String,
    option_type: CommandLineType,
}

#[derive(Clone, Debug)]
pub enum CommandLineType {
    Boolean { default_value: Option<bool> },
    String { default_value: Option<String> },
}

impl fmt::Display for CommandLineType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandLineType::Boolean { default_value } => match default_value {
                Some(v) => write!(f, "bool (default: {v})"),
                None => write!(f, "bool"),
            },
            CommandLineType::String { default_value } => match default_value {
                Some(v) => write!(f, "string (default: \"{v}\")"),
                None => write!(f, "string"),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum CommandLineValue {
    Boolean(bool),
    String(String),
}

impl CommandLineValue {
    pub fn as_bool(&self) -> bool {
        match self {
            CommandLineValue::Boolean(value) => *value,
            _ => panic!("This value is not a boolean."),
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            CommandLineValue::String(value) => value.to_string(),
            _ => panic!("This value is not a boolean."),
        }
    }
}

impl CommandLineOption {
    pub fn new(
        short_name: Option<String>,
        long_name: Option<String>,
        description: String,
        option_type: CommandLineType,
    ) -> Self {
        let has_short_name = short_name.is_some();
        let has_long_name = long_name.is_some();

        if !has_short_name && !has_long_name {
            panic!("A command line option must have at least a short name or a long name.");
        }
        if has_short_name && has_long_name && short_name == long_name {
            panic!(
                "This command line option has identical short name and long name: '{}'.",
                short_name.unwrap()
            );
        }
        if has_short_name && short_name.clone().unwrap().contains('=') {
            panic!("A command line option's short name can not contain '='.");
        }
        if has_long_name && long_name.clone().unwrap().contains('=') {
            panic!("A command line option's long name can not contain '='.");
        }
        CommandLineOption {
            short_name,
            long_name,
            description,
            option_type,
        }
    }

    fn is_mandatory(&self) -> bool {
        match &self.option_type {
            CommandLineType::Boolean { default_value } => default_value.is_none(),
            CommandLineType::String { default_value } => default_value.is_none(),
        }
    }

    /// Tries to match this option with the given value. Returns:
    ///  - `None`, if this option didn't match at all.
    ///  - `Some(Ok(...))`, if this option matched with a value.
    ///  - `Some(Err(msg))`, if an error occurred during matching of this option, like no value for a mandatory option or invalid value parsing.
    fn try_match(
        &self,
        argument_name: &str,
        argument_value: Option<&str>,
    ) -> Option<Result<CommandLineValue, String>> {
        let matches_short_name = self.short_name.as_deref() == Some(argument_name);
        let matches_long_name = self.long_name.as_deref() == Some(argument_name);
        if !matches_short_name && !matches_long_name {
            return None; // didn't match this option at all
        }

        Some(match &self.option_type {
            CommandLineType::Boolean { default_value } => match argument_value {
                Some("0") | Some("false") => Ok(CommandLineValue::Boolean(false)),
                Some("1") | Some("true") => Ok(CommandLineValue::Boolean(true)),
                None => Ok(CommandLineValue::Boolean(!default_value.unwrap())),
                Some(other) => Err(format!(
                    "'{other}' is not a valid value for '{argument_name}'."
                )),
            },
            CommandLineType::String { .. } => match argument_value {
                Some(s) => Ok(CommandLineValue::String(s.to_owned())),
                None => Err(format!("Option '{argument_name}' requires a value.")),
            },
        })
    }
}

impl CommandLineParser {
    pub fn new(
        program_name: &str,
        description: Option<String>,
        options: Vec<CommandLineOption>,
    ) -> Self {
        let mut actual_options = Vec::with_capacity(options.len() + 1);

        actual_options.push(CommandLineOption {
            short_name: Some("h".to_string()),
            long_name: Some("help".to_string()),
            description: "Prints this message and exits.".to_owned(),
            option_type: CommandLineType::Boolean {
                default_value: Some(false),
            },
        });

        for opt in options {
            actual_options.push(opt);
        }

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

    fn print_help(&self, out: &mut impl io::Write) {
        write!(out, "\n {}", self.program_name).unwrap();
        if let Some(desc) = &self.description {
            write!(out, " - {desc}").unwrap();
        }
        write!(out, "\n\nOptions:\n").unwrap();

        let max_short_name_length = self
            .options
            .iter()
            .filter_map(|o| o.short_name.as_deref())
            .map(|s| s.len())
            .max()
            .unwrap_or(0);

        let max_long_name_length = self
            .options
            .iter()
            .filter_map(|o| o.long_name.as_deref())
            .map(|s| s.len())
            .max()
            .unwrap_or(0);

        for option in self.options.iter() {
            match &option.short_name {
                Some(sn) => write!(out, " -{sn:<max_short_name_length$}").unwrap(),
                None => write!(out, "  {:<max_short_name_length$}", "").unwrap(),
            }

            match &option.long_name {
                Some(ln) => {
                    let sep = if option.short_name.is_some() {
                        ","
                    } else {
                        " "
                    };
                    write!(out, "{sep} --{ln:<max_long_name_length$}").unwrap();
                }
                None => {
                    write!(out, "  {:<max_long_name_length$}  ", "").unwrap();
                }
            }

            writeln!(out, "  {}", option.description).unwrap();
        }
    }

    fn load_defaults(&self) -> HashMap<String, CommandLineValue> {
        let mut values: HashMap<String, CommandLineValue> = HashMap::new();

        for option in &self.options {
            let value: Option<CommandLineValue> = match &option.option_type {
                CommandLineType::Boolean {
                    default_value: Some(v),
                } => Some(CommandLineValue::Boolean(*v)),
                CommandLineType::String {
                    default_value: Some(v),
                } => Some(CommandLineValue::String(v.clone())),
                _ => None,
            };

            if value.is_some() {
                if option.short_name.is_some() {
                    values.insert(option.short_name.clone().unwrap(), value.clone().unwrap());
                }
                if option.long_name.is_some() {
                    values.insert(option.long_name.clone().unwrap(), value.clone().unwrap());
                }
            }
        }

        values
    }

    // On CommandLineParser:
    pub fn parse(&self, args: Args) -> Arguments {
        let args_str: Vec<String> = args.skip(1).collect();
        self.parse_or_exit(&args_str)
    }

    pub fn parse_or_exit(&self, args: &[String]) -> Arguments {
        match self.parse_str(args) {
            Ok(arguments) => arguments,
            Err(e) => {
                eprintln!("Error: {e}");
                self.print_help(&mut io::stdout());
                std::process::exit(1);
            }
        }
    }

    pub fn parse_str(&self, args: &[String]) -> Result<Arguments, String> {
        let mut values = self.load_defaults();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];

            let (argument_name, argument_value) = if let Some(stripped) = arg.strip_prefix("--") {
                if let Some(eq) = arg.find('=') {
                    (&arg[2..eq], Some(&arg[eq + 1..]))
                } else {
                    i += 1;
                    (stripped, args.get(i).map(String::as_str))
                }
            } else if let Some(stripped) = arg.strip_prefix('-') {
                if let Some(eq) = arg.find('=') {
                    (&arg[1..eq], Some(&arg[eq + 1..]))
                } else {
                    i += 1;
                    (stripped, args.get(i).map(String::as_str))
                }
            } else {
                return Err(format!("Expected an option but found '{arg}'."));
            };

            let mut matched = false;
            for option in &self.options {
                match option.try_match(argument_name, argument_value) {
                    Some(Ok(parsed_value)) => {
                        if let Some(sn) = &option.short_name {
                            values.insert(sn.clone(), parsed_value.clone());
                        }
                        if let Some(ln) = &option.long_name {
                            values.insert(ln.clone(), parsed_value.clone());
                        }
                        matched = true;
                        break;
                    }
                    Some(Err(e)) => return Err(e),
                    None => {}
                }
            }

            if !matched {
                return Err(format!("Unrecognized option '{argument_name}'."));
            }

            i += 1;
        }

        if matches!(values.get("help"), Some(CommandLineValue::Boolean(true))) {
            self.print_help(&mut io::stdout());
            std::process::exit(0);
        }

        for option in &self.options {
            if option.is_mandatory() {
                let present = option
                    .short_name
                    .as_ref()
                    .is_some_and(|n| values.contains_key(n))
                    || option
                        .long_name
                        .as_ref()
                        .is_some_and(|n| values.contains_key(n));
                if !present {
                    let name = option
                        .long_name
                        .as_ref()
                        .map(|n| format!("--{n}"))
                        .or_else(|| option.short_name.as_ref().map(|n| format!("-{n}")))
                        .unwrap();
                    return Err(format!(
                        "Option '{name}' is mandatory but was not provided."
                    ));
                }
            }
        }

        Ok(Arguments { values })
    }
}

pub struct Arguments {
    values: HashMap<String, CommandLineValue>,
}

impl Arguments {
    pub fn get(&self, name: &str) -> Option<&CommandLineValue> {
        self.values.get(name)
    }

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        match self.values.get(name) {
            Some(CommandLineValue::Boolean(b)) => Some(*b),
            _ => None,
        }
    }

    pub fn get_string(&self, name: &str) -> Option<&str> {
        match self.values.get(name) {
            Some(CommandLineValue::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn parse_default_boolean_option() {
        let parser = CommandLineParser::new(
            "test-parser",
            Some("A parser for testing".to_string()),
            vec![CommandLineOption {
                short_name: Some("q".to_string()),
                long_name: Some("quiet".to_string()),
                description: "Does not print anything on stdout.".to_owned(),
                option_type: CommandLineType::Boolean {
                    default_value: Some(false),
                },
            }],
        );

        let args = parser.parse_str(&[]).unwrap();
        assert!(!args.get("q").unwrap().as_bool());
        assert!(!args.get("quiet").unwrap().as_bool());
    }

    #[rstest]
    #[case(vec!["-q"])]
    #[case(vec!["-q", "1"])]
    #[case(vec!["-q", "true"])]
    #[case(vec!["-q=1"])]
    #[case(vec!["-q=true"])]
    #[case(vec!["--quiet"])]
    #[case(vec!["--quiet", "1"])]
    #[case(vec!["--quiet", "true"])]
    #[case(vec!["--quiet=1"])]
    #[case(vec!["--quiet=true"])]
    fn parse_boolean_option_true(#[case] input: Vec<&str>) {
        let parser = CommandLineParser::new(
            "test-parser",
            Some("A parser for testing".to_string()),
            vec![CommandLineOption {
                short_name: Some("q".to_string()),
                long_name: Some("quiet".to_string()),
                description: "Does not print anything on stdout.".to_owned(),
                option_type: CommandLineType::Boolean {
                    default_value: Some(false),
                },
            }],
        );

        let string_args: Vec<String> = input.iter().map(|s| s.to_string()).collect();
        let args = parser.parse_str(&string_args).unwrap();
        assert!(args.get("q").unwrap().as_bool());
        assert!(args.get("quiet").unwrap().as_bool());
    }

    #[rstest]
    #[case(vec!["-q", "0"])]
    #[case(vec!["-q", "false"])]
    #[case(vec!["-q=0"])]
    #[case(vec!["-q=false"])]
    #[case(vec!["--quiet", "0"])]
    #[case(vec!["--quiet", "false"])]
    #[case(vec!["--quiet=0"])]
    #[case(vec!["--quiet=false"])]
    fn parse_boolean_option_false(#[case] input: Vec<&str>) {
        let parser = CommandLineParser::new(
            "test-parser",
            Some("A parser for testing".to_string()),
            vec![CommandLineOption {
                short_name: Some("q".to_string()),
                long_name: Some("quiet".to_string()),
                description: "Does not print anything on stdout.".to_owned(),
                option_type: CommandLineType::Boolean {
                    default_value: Some(false),
                },
            }],
        );

        let string_args: Vec<String> = input.iter().map(|s| s.to_string()).collect();
        let args = parser.parse_str(&string_args).unwrap();
        assert!(!args.get("q").unwrap().as_bool());
        assert!(!args.get("quiet").unwrap().as_bool());
    }

    #[rstest]
    #[case(vec!["-i", "input.txt"])]
    #[case(vec!["-i=input.txt"])]
    #[case(vec!["--input", "input.txt"])]
    #[case(vec!["--input=input.txt"])]
    fn parse_string_option(#[case] input: Vec<&str>) {
        let parser = CommandLineParser::new(
            "test-parser",
            Some("A parser for testing".to_string()),
            vec![CommandLineOption {
                short_name: Some("i".to_string()),
                long_name: Some("input".to_string()),
                description: "The input file to read from.".to_owned(),
                option_type: CommandLineType::String {
                    default_value: None,
                },
            }],
        );

        let string_args: Vec<String> = input.iter().map(|s| s.to_string()).collect();
        let args = parser.parse_str(&string_args).unwrap();
        assert_eq!("input.txt", args.get("i").unwrap().as_str());
        assert_eq!("input.txt", args.get("input").unwrap().as_str());
    }

    #[test]
    fn help_message() {
        let parser = CommandLineParser::new(
            "test-parser",
            Some("A parser for testing".to_string()),
            vec![
                CommandLineOption {
                    short_name: Some("i".to_string()),
                    long_name: Some("input".to_string()),
                    description: "The input file to read from.".to_owned(),
                    option_type: CommandLineType::String {
                        default_value: None,
                    },
                },
                CommandLineOption {
                    short_name: None,
                    long_name: Some("output".to_string()),
                    description: "The output file to write to.".to_owned(),
                    option_type: CommandLineType::String {
                        default_value: None,
                    },
                },
                CommandLineOption {
                    short_name: Some("v".to_string()),
                    long_name: None,
                    description: "Prints verbose messages.".to_owned(),
                    option_type: CommandLineType::String {
                        default_value: None,
                    },
                },
            ],
        );

        let expected_message: String = [
            "",
            " test-parser - A parser for testing",
            "",
            "Options:",
            " -h, --help    Prints this message and exits.",
            " -i, --input   The input file to read from.",
            "     --output  The output file to write to.",
            " -v            Prints verbose messages.",
            "",
        ]
        .join("\n");

        let mut out: Vec<u8> = Vec::new();
        parser.print_help(&mut out);
        let text = String::from_utf8(out).unwrap();
        assert_eq!(
            expected_message, text,
            "Expected help message to be '''\n{expected_message}\n''' but was '''\n{text}\n'''",
        );
    }
}
