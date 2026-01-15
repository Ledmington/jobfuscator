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
    Array {
        inner: Box<Type>,
    },
    Object {
        class_name: String,
    },
    Generic {
        class_name: String,
        types: Vec<Type>,
    },
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
            Type::Generic { class_name, types } => write!(
                f,
                "{}<{}>",
                class_name,
                types
                    .iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
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

struct Reader {
    content: String,
    pos: usize,
}

impl Reader {
    /**
     * Returns the next char without moving.
     */
    fn peek(&self) -> char {
        self.content.chars().nth(self.pos).unwrap()
    }

    /**
     * Moves the reader by one character.
     */
    fn move_1(&mut self) {
        self.pos += 1;
    }

    /**
     * Returns the next char and moves.
     */
    fn next(&mut self) -> char {
        let ch: char = self.content.chars().nth(self.pos).unwrap();
        self.pos += 1;
        ch
    }
}

fn parse_type(reader: &mut Reader) -> Type {
    match reader.peek() {
        'V' => {
            reader.move_1();
            Type::Void
        }
        'Z' => {
            reader.move_1();
            Type::Boolean
        }
        'C' => {
            reader.move_1();
            Type::Char
        }
        'B' => {
            reader.move_1();
            Type::Byte
        }
        'S' => {
            reader.move_1();
            Type::Short
        }
        'I' => {
            reader.move_1();
            Type::Int
        }
        'J' => {
            reader.move_1();
            Type::Long
        }
        'F' => {
            reader.move_1();
            Type::Float
        }
        'D' => {
            reader.move_1();
            Type::Double
        }
        '[' => {
            reader.move_1();
            Type::Array {
                inner: Box::new(parse_type(reader)),
            }
        }
        'L' => {
            reader.move_1();
            let mut s: String = "".to_owned();
            while (reader.pos < reader.content.len() - 1)
                && (reader.peek() != ';' && reader.peek() != '<')
            {
                let ch = reader.next();
                if ch == '/' {
                    s += ".";
                } else {
                    s += &ch.to_string();
                }
            }

            if reader.peek() == ';' {
                reader.move_1();
                Type::Object { class_name: s }
            } else {
                reader.move_1(); // '<'

                let mut types = Vec::new();

                while (reader.pos < reader.content.len()) && reader.peek() != '>' {
                    types.push(parse_type(reader));
                }

                if reader.peek() != '>' {
                    unreachable!("Invalid descriptor (expected '>'): '{}'.", reader.content);
                }
                reader.move_1(); // '>'

                if reader.peek() != ';' {
                    unreachable!("Invalid descriptor (expected ';'): '{}'.", reader.content);
                }
                reader.move_1(); // ';'

                Type::Generic {
                    class_name: s,
                    types,
                }
            }
        }
        _ => unreachable!("Invalid descriptor: '{}'.", reader.content),
    }
}

pub fn parse_field_descriptor(raw_descriptor: &str) -> FieldDescriptor {
    FieldDescriptor {
        field_type: parse_type(&mut Reader {
            content: raw_descriptor.to_owned(),
            pos: 0,
        }),
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

    let return_type: Type = parse_type(&mut Reader {
        content: raw_descriptor.split(')').next_back().unwrap().to_owned(),
        pos: 0,
    });

    let parameters_string: String = raw_descriptor.split(')').next().unwrap()[1..].to_owned();

    let mut parameter_types = Vec::new();

    let mut reader: Reader = Reader {
        content: parameters_string,
        pos: 0,
    };

    while reader.pos < reader.content.len() {
        parameter_types.push(parse_type(&mut reader));
    }

    MethodDescriptor {
        return_type,
        parameter_types,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_parsing() {
        let cases = [
            (Type::Void, "V"),
            (Type::Boolean, "Z"),
            (Type::Char, "C"),
            (Type::Byte, "B"),
            (Type::Short, "S"),
            (Type::Int, "I"),
            (Type::Long, "J"),
            (Type::Float, "F"),
            (Type::Double, "D"),
            (
                Type::Array {
                    inner: Box::new(Type::Double),
                },
                "[D",
            ),
            (
                Type::Array {
                    inner: Box::new(Type::Array {
                        inner: Box::new(Type::Byte),
                    }),
                },
                "[[B",
            ),
            (
                Type::Array {
                    inner: Box::new(Type::Array {
                        inner: Box::new(Type::Array {
                            inner: Box::new(Type::Long),
                        }),
                    }),
                },
                "[[[J",
            ),
            (
                Type::Object {
                    class_name: "java.lang.Object".to_string(),
                },
                "Ljava/lang/Object;",
            ),
            (
                Type::Generic {
                    class_name: "java.util.List".to_string(),
                    types: vec![Type::Object {
                        class_name: "java.lang.String".to_string(),
                    }],
                },
                "Ljava/util/List<Ljava/lang/String;>;",
            ),
            (
                Type::Generic {
                    class_name: "java.util.Map".to_string(),
                    types: vec![
                        Type::Object {
                            class_name: "java.lang.String".to_string(),
                        },
                        Type::Object {
                            class_name: "java.lang.Integer".to_string(),
                        },
                    ],
                },
                "Ljava/util/Map<Ljava/lang/String;Ljava/lang/Integer;>;",
            ),
            (
                Type::Generic {
                    class_name: "java.util.Map".to_string(),
                    types: vec![
                        Type::Generic {
                            class_name: "java.util.List".to_string(),
                            types: vec![Type::Object {
                                class_name: "java.lang.String".to_string(),
                            }],
                        },
                        Type::Generic {
                            class_name: "java.util.Set".to_string(),
                            types: vec![Type::Object {
                                class_name: "java.lang.Integer".to_string(),
                            }],
                        },
                    ],
                },
                "Ljava/util/Map<Ljava/util/List<Ljava/lang/String;>;Ljava/util/Set<Ljava/lang/Integer;>;>;",
            ),
        ];

        for (expected, input) in cases {
            assert_eq!(
                expected,
                parse_type(&mut Reader {
                    content: input.to_string(),
                    pos: 0
                })
            );
        }
    }

    #[test]
    #[should_panic]
    fn invalid_parsing() {
        let cases = ["Q", "[", "[]"];

        for input in cases {
            parse_type(&mut Reader {
                content: input.to_string(),
                pos: 0,
            });
        }
    }

    #[test]
    fn method_descriptor_parsing() {
        let cases = [(
            MethodDescriptor {
                return_type: Type::Void,
                parameter_types: vec![
                    Type::Object {
                        class_name: "java.lang.String".to_string(),
                    },
                    Type::Int,
                    Type::Long,
                ],
            },
            "(Ljava/lang/String;IJ)V",
        )];

        for (expected, input) in cases {
            assert_eq!(expected, parse_method_descriptor(input));
        }
    }
}
