#![forbid(unsafe_code)]

use std::fmt::{Display, Formatter, Result};

#[derive(Debug, PartialEq)]
pub enum Type {
    Void,
    Boolean,
    Char,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    Array { inner: Box<Type> },
    Object { class_name: String },
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Boolean => write!(f, "boolean"),
            Type::Char => write!(f, "char"),
            Type::Byte => write!(f, "byte"),
            Type::Short => write!(f, "short"),
            Type::Int => write!(f, "int"),
            Type::Long => write!(f, "long"),
            Type::Float => write!(f, "float"),
            Type::Double => write!(f, "double"),
            Type::Array { inner } => write!(f, "{}[]", inner),
            Type::Object { class_name } => write!(f, "{}", class_name),
        }
    }
}

pub trait Descriptor: Display {}

pub struct FieldDescriptor {
    field_type: Type,
}

impl Descriptor for FieldDescriptor {}

impl Display for FieldDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.field_type)
    }
}

#[derive(Debug, PartialEq)]
pub struct MethodDescriptor {
    pub return_type: Type,
    pub parameter_types: Vec<Type>,
}

impl Descriptor for MethodDescriptor {}

impl Display for MethodDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}({})",
            self.return_type,
            self.parameter_types
                .iter()
                .map(|t| format!("{}", t))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

fn parse_type(raw_descriptor: &str) -> Type {
    assert!(!raw_descriptor.is_empty(), "Empty type descriptor.");

    match raw_descriptor {
        "V" => Type::Void,
        "Z" => Type::Boolean,
        "C" => Type::Char,
        "B" => Type::Byte,
        "S" => Type::Short,
        "I" => Type::Int,
        "F" => Type::Float,
        "D" => Type::Double,
        "J" => Type::Long,
        _ => {
            if let Some(stripped) = raw_descriptor.strip_prefix('[') {
                return Type::Array {
                    inner: Box::new(parse_type(stripped)),
                };
            }

            if raw_descriptor.starts_with('L') {
                assert!(raw_descriptor.ends_with(';'));
                assert!(raw_descriptor.len() > 2);

                return Type::Object {
                    class_name: raw_descriptor[1..(raw_descriptor.len() - 1)].replace('/', "."),
                };
            }

            unreachable!("Invalid descriptor: '{}'.", raw_descriptor);
        }
    }
}

pub fn parse_field_descriptor(raw_descriptor: &str) -> FieldDescriptor {
    FieldDescriptor {
        field_type: parse_type(raw_descriptor),
    }
}

pub fn parse_method_descriptor(raw_descriptor: &str) -> MethodDescriptor {
    assert!(!raw_descriptor.is_empty(), "Empty method descriptor.");
    debug_assert!(
        raw_descriptor.starts_with('(')
            && raw_descriptor.chars().filter(|c| *c == '(').count() == 1
            && raw_descriptor.chars().filter(|c| *c == ')').count() == 1
            && !raw_descriptor.ends_with(')'),
        "Invalid method descriptor: '{}'.",
        raw_descriptor
    );

    let return_type: Type = parse_type(raw_descriptor.split(')').next_back().unwrap());

    let parameters_string: String = raw_descriptor.split(')').next().unwrap()[1..].to_owned();

    let mut parameter_types = Vec::new();
    let mut chars = parameters_string.as_str();

    while !chars.is_empty() {
        let (ty, consumed) = match chars.chars().next().unwrap() {
            'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' | 'V' => (parse_type(&chars[..1]), 1),
            '[' => {
                // consume all leading '['
                let array_len = chars.chars().take_while(|c| *c == '[').count();

                if chars.chars().nth(array_len).unwrap() == 'L' {
                    let semicolon = chars.find(';').unwrap();
                    (parse_type(&chars[..=semicolon]), semicolon + 1)
                } else {
                    (parse_type(&chars[..array_len + 1]), array_len + 1)
                }
            }
            'L' => {
                let semicolon = chars.find(';').unwrap();
                (parse_type(&chars[..=semicolon]), semicolon + 1)
            }
            _ => unreachable!("Invalid parameter descriptor: '{}'", chars),
        };

        parameter_types.push(ty);
        chars = &chars[consumed..];
    }

    MethodDescriptor {
        return_type,
        parameter_types,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(Type::Void, "V")]
    #[test_case(Type::Boolean, "Z")]
    #[test_case(Type::Char, "C")]
    #[test_case(Type::Byte, "B")]
    #[test_case(Type::Short, "S")]
    #[test_case(Type::Int, "I")]
    #[test_case(Type::Long, "J")]
    #[test_case(Type::Float, "F")]
    #[test_case(Type::Double, "D")]
    #[test_case(Type::Array{inner:Box::new(Type::Double)}, "[D")]
    #[test_case(Type::Array{inner:Box::new(Type::Array{inner:Box::new(Type::Byte)})}, "[[B")]
    #[test_case(Type::Array{inner:Box::new(Type::Array{inner:Box::new(Type::Array{inner:Box::new(Type::Long)})})}, "[[[J")]
    #[test_case(Type::Object{class_name:"java.lang.Object".to_string()}, "Ljava/lang/Object;")]
    fn descriptor_parsing(expected: Type, input: &str) {
        assert_eq!(expected, parse_type(input));
    }

    #[test_case(
        MethodDescriptor{
            return_type: Type::Void,
            parameter_types: vec![
                Type::Object{class_name:"java.lang.String".to_string()},
                Type::Int,
                Type::Long
            ],
        }, "(Ljava/lang/String;IJ)V"
    )]
    fn method_descriptor_parsing(expected: MethodDescriptor, input: &str) {
        assert_eq!(expected, parse_method_descriptor(input));
    }
}
