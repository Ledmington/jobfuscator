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
        if short_name.is_none() && long_name.is_none() {
            panic!("A command line option must have at least a short name or a long name.");
        }
        if short_name.is_some() && long_name.is_some() && short_name == long_name {
            panic!(
                "This command line option has identical short name and long name: '{}'.",
                short_name.unwrap()
            );
        }
        if short_name.is_some() && short_name.clone().unwrap().contains('=') {
            panic!("A command line option's short name can not contain '='.");
        }
        if long_name.is_some() && long_name.clone().unwrap().contains('=') {
            panic!("A command line option's long name can not contain '='.");
        }
        CommandLineOption {
            short_name,
            long_name,
            description,
            option_type,
        }
    }

    fn try_match(
        &self,
        argument_name: &str,
        argument_value: Option<&str>,
    ) -> Option<CommandLineValue> {
        let matches_short_name: bool =
            self.short_name.is_some() && self.short_name.clone().unwrap() == argument_name;
        let matches_long_name: bool =
            self.long_name.is_some() && self.long_name.clone().unwrap() == argument_name;
        if !matches_short_name && !matches_long_name {
            return None;
        }

        match self.option_type {
            CommandLineType::Boolean { default_value } => match argument_value {
                Some("0") | Some("false") => Some(CommandLineValue::Boolean(false)),
                Some("1") | Some("true") => Some(CommandLineValue::Boolean(true)),
                None => Some(CommandLineValue::Boolean(!default_value.unwrap())),
                _ => panic!(
                    "'{}' is not a valid boolean value.",
                    argument_value.unwrap()
                ),
            },
            CommandLineType::String { .. } => {
                Some(CommandLineValue::String(argument_value.unwrap().to_owned()))
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
                Some(sn) => write!(out, " -{:<max_short_name_length$}", sn).unwrap(),
                None => write!(out, "  {:<max_short_name_length$}", "").unwrap(),
            }

            match &option.long_name {
                Some(ln) => {
                    let sep = if option.short_name.is_some() {
                        ","
                    } else {
                        " "
                    };
                    write!(out, "{} --{:<max_long_name_length$}", sep, ln).unwrap();
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

        return values;
    }

    pub fn parse(&self, args: Args) -> Arguments {
        let args_str: Vec<String> = args.skip(1).collect();
        self.parse_str(&args_str)
    }

    pub fn parse_str(&self, args: &Vec<String>) -> Arguments {
        let mut values: HashMap<String, CommandLineValue> = self.load_defaults();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];

            let argument_name: &str;
            let argument_value: Option<&str>;

            if arg.starts_with("--") {
                if let Some(equals_pos) = arg.find('=') {
                    argument_name = &arg[2..equals_pos];
                    argument_value = Some(&arg[(equals_pos + 1)..]);
                } else {
                    argument_name = &arg[2..];
                    argument_value = if i + 1 < args.len() {
                        Some(&args[i + 1])
                    } else {
                        None
                    };
                    i += 1;
                }
            } else if arg.starts_with('-') {
                if let Some(equals_pos) = arg.find('=') {
                    argument_name = &arg[1..equals_pos];
                    argument_value = Some(&arg[(equals_pos + 1)..]);
                } else {
                    argument_name = &arg[1..];
                    argument_value = if i + 1 < args.len() {
                        Some(&args[i + 1])
                    } else {
                        None
                    };
                    i += 1;
                }
            } else {
                eprintln!("Error: expected an option but found '{arg}'.");
                self.print_help(&mut io::stdout());
                std::process::exit(1);
            }

            let mut matched = false;
            for option in &self.options {
                match option.try_match(argument_name, argument_value) {
                    Some(parsed_value) => {
                        if let Some(sn) = &option.short_name {
                            values.insert(sn.to_string(), parsed_value.clone());
                        }
                        if let Some(ln) = &option.long_name {
                            values.insert(ln.to_string(), parsed_value.clone());
                        }
                        matched = true;
                        break;
                    }
                    None if option.short_name.as_deref() == Some(argument_name)
                        || option.long_name.as_deref() == Some(argument_name) =>
                    {
                        // The option matched by name but the value was invalid.
                        eprintln!(
                            "Error: '{}' is not a valid value for '{argument_name}'.",
                            argument_value.unwrap_or("")
                        );
                        self.print_help(&mut io::stdout());
                        std::process::exit(1);
                    }
                    None => {}
                }
            }

            if !matched {
                eprintln!("Error: unrecognized option '{argument_name}'.");
                self.print_help(&mut io::stdout());
                std::process::exit(1);
            }

            i += 1;
        }

        if matches!(values.get("help"), Some(CommandLineValue::Boolean(true))) {
            self.print_help(&mut io::stdout());
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
                short_name: Some("q".to_string()),
                long_name: Some("quiet".to_string()),
                description: "Does not print anything on stdout.".to_owned(),
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
                short_name: Some("q".to_string()),
                long_name: Some("quiet".to_string()),
                description: "Does not print anything on stdout.".to_owned(),
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
                short_name: Some("q".to_string()),
                long_name: Some("quiet".to_string()),
                description: "Does not print anything on stdout.".to_owned(),
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
        let args = parser.parse_str(&string_args);
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

        let expected_message: String = vec![
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
            "Expected help message to be '''\n{}\n''' but was '''\n{}\n'''",
            expected_message, text
        );
    }
}
