#![forbid(unsafe_code)]

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

    fn canonical_name(&self) -> &str {
        self.long_name
            .as_deref()
            .or(self.short_name.as_deref())
            .unwrap()
    }

    fn usage_names(&self) -> String {
        match (&self.short_name, &self.long_name) {
            (Some(s), Some(l)) => format!("{s}, {l}"),
            (Some(s), None) => s.clone(),
            (None, Some(l)) => l.clone(),
            (None, None) => unreachable!(),
        }
    }

    fn try_match(&self, argument_name: &str, argument_value: &str) -> Option<CommandLineValue> {
        let matches_short_name: bool =
            self.short_name.is_some() && self.short_name.clone().unwrap() == argument_name;
        let matches_long_name: bool =
            self.long_name.is_some() && self.long_name.clone().unwrap() == argument_name;
        if !matches_short_name && !matches_long_name {
            return None;
        }

        match self.option_type {
            CommandLineType::Boolean { .. } => match argument_value {
                "0" | "false" => Some(CommandLineValue::Boolean(false)),
                "1" | "true" => Some(CommandLineValue::Boolean(true)),
                _ => panic!("'{}' is not a valid boolean value.", argument_value),
            },
            CommandLineType::String { .. } => {
                Some(CommandLineValue::String(argument_value.to_owned()))
            }
        }
    }
}

impl CommandLineParser {
    pub fn new(
        program_name: &str,
        description: Option<String>,
        options: Vec<CommandLineOption>,
    ) -> Self {
        let mut actual_options = options;

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

    fn print_help(&self) {
        println!("Usage: {}", self.program_name);

        if let Some(desc) = &self.description {
            println!("{desc}");
        }

        println!("\nOptions:");

        for option in &self.options {
            println!("  {:<24} {}", option.usage_names(), option.option_type);
        }
    }

    pub fn parse(&self, args: Args) -> Arguments {
        let args_str: Vec<String> = args.skip(1).collect();
        self.parse_str(&args_str)
    }

    pub fn parse_str(&self, args: &Vec<String>) -> Arguments {
        let mut values: HashMap<String, CommandLineValue> = HashMap::new();

        // Load defaults
        for option in &self.options {
            let key = option.canonical_name().to_string();
            match &option.option_type {
                CommandLineType::Boolean {
                    default_value: Some(v),
                } => {
                    values.insert(key, CommandLineValue::Boolean(*v));
                }
                CommandLineType::String {
                    default_value: Some(v),
                } => {
                    values.insert(key, CommandLineValue::String(v.clone()));
                }
                _ => {}
            }
        }

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];

            let argument_name: &str;
            let argument_value: &str;

            if arg.starts_with('-') {
                let contains_equals: bool = arg.contains('=');
                if contains_equals {
                    let equals_pos = arg.find('=').unwrap();
                    argument_name = &arg[1..equals_pos];
                    argument_value = &arg[(equals_pos + 1)..];
                } else {
                    if i + 1 >= args.len() {
                        panic!(
                            "Option '{}' expected an argument but found end of input.",
                            arg
                        );
                    }
                    argument_name = &arg[1..];
                    argument_value = &args[i + 1];
                    i += 1;
                }
            } else if arg.starts_with("--") {
                let contains_equals: bool = arg.contains('=');
                if contains_equals {
                    let equals_pos = arg.find('=').unwrap();
                    argument_name = &arg[2..equals_pos];
                    argument_value = &arg[(equals_pos + 1)..];
                } else {
                    if i + 1 >= args.len() {
                        panic!(
                            "Option '{}' expected an argument but found end of input.",
                            arg
                        );
                    }
                    argument_name = &arg[2..];
                    argument_value = &args[i + 1];
                    i += 1;
                }
            } else {
                panic!("Expected an argument but found '{}'.", arg);
            }

            let mut matched = false;
            for option in &self.options {
                if let Some(parsed_value) = option.try_match(argument_name, argument_value) {
                    if let Some(sn) = &option.short_name {
                        values.insert(sn.to_string(), parsed_value.clone());
                    }
                    if let Some(ln) = &option.long_name {
                        values.insert(ln.to_string(), parsed_value.clone());
                    }
                    matched = true;
                    break;
                }
            }

            if !matched {
                eprintln!("Warning: unrecognized argument '{argument_name}'");
            }

            i += 1;
        }

        if matches!(values.get("--help"), Some(CommandLineValue::Boolean(true))) {
            self.print_help();
            std::process::exit(0);
        }

        Arguments { values }
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
                short_name: Some("-q".to_string()),
                long_name: Some("--quiet".to_string()),
                option_type: CommandLineType::Boolean {
                    default_value: Some(false),
                },
            }],
        );

        let args = parser.parse_str(&vec![]);
        assert_eq!(false, args.get("q").unwrap().as_bool());
        assert_eq!(false, args.get("quiet").unwrap().as_bool());
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
                short_name: Some("-q".to_string()),
                long_name: Some("--quiet".to_string()),
                option_type: CommandLineType::Boolean {
                    default_value: Some(false),
                },
            }],
        );

        let string_args: Vec<String> = input.iter().map(|s| s.to_string()).collect();
        let args = parser.parse_str(&string_args);
        assert_eq!(true, args.get("q").unwrap().as_bool());
        assert_eq!(true, args.get("quiet").unwrap().as_bool());
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
                short_name: Some("-q".to_string()),
                long_name: Some("--quiet".to_string()),
                option_type: CommandLineType::Boolean {
                    default_value: Some(false),
                },
            }],
        );

        let string_args: Vec<String> = input.iter().map(|s| s.to_string()).collect();
        let args = parser.parse_str(&string_args);
        assert_eq!(false, args.get("q").unwrap().as_bool());
        assert_eq!(false, args.get("quiet").unwrap().as_bool());
    }
}
