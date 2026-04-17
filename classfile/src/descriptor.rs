#![forbid(unsafe_code)]

use std::{collections::BTreeMap, iter::Peekable, str::Chars};

fn decode_type_it(it: &mut Peekable<Chars>) -> String {
    let ch = it.next().unwrap();
    match ch {
        'B' => "byte".to_owned(),
        'C' => "char".to_owned(),
        'D' => "double".to_owned(),
        'F' => "float".to_owned(),
        'I' => "int".to_owned(),
        'J' => "long".to_owned(),
        'S' => "short".to_owned(),
        'Z' => "boolean".to_owned(),
        'V' => "void".to_owned(),
        'L' => {
            let mut s = String::new();
            while let Some(&x) = it.peek() {
                it.next();
                if x == ';' {
                    return s;
                }
                if x == '<' {
                    s.push('<');
                    break;
                }
                s.push(if x == '/' { '.' } else { x });
            }

            // parsing generics
            s.push_str(&decode_type_it(it));
            while let Some(&x) = it.peek() {
                it.next();
                if x == '>' {
                    s.push('>');
                    break;
                }
                it.next_back();
                s.push_str(", ");
                s.push_str(&decode_type_it(it));
            }

            // skipping the ending ';'
            let last = it.next().unwrap();
            if last != ';' {
                panic!(
                    "Expected to find ';' at the end of class name but was '{}'.",
                    last
                );
            }

            s
        }
        '(' => {
            let mut s = String::new();
            s.push('(');
            while let Some(&x) = it.peek() {
                it.next();
                if x == ')' {
                    s.push(')');
                    break;
                }

                if s.len() > 1 {
                    s.push_str(", ");
                }

                it.next_back();
                s.push_str(&decode_type_it(it));
            }

            // return type
            let return_type = decode_type_it(it);

            return_type + &s
        }
        '[' => decode_type_it(it) + "[]",
        'T' => {
            let mut s = String::new();
            while let Some(&x) = it.peek() {
                it.next();
                if x == ';' {
                    return s;
                }
                s.push(if x == '/' { '.' } else { x });
            }
            panic!();
        }
        '*' => "?".to_owned(),
        '+' => "? extends ".to_owned() + &decode_type_it(it),
        _ => panic!("Unknown or unexpected character '{}'.", ch),
    }
}

fn decode_generics(it: &mut Peekable<Chars>) -> BTreeMap<String, Vec<String>> {
    expect(it, '<');
    let mut generics: BTreeMap<String, Vec<String>> = BTreeMap::new();
    while let Some(&_) = it.peek() {
        if it.next().unwrap() == '>' {
            break;
        }
        it.next_back();

        let mut s = String::new();
        while let Some(&x) = it.peek() {
            it.next();
            if x == ':' {
                break;
            }
            s.push(x);
        }
        let generic_type_name = s;

        let mut generic_type_bounds: Vec<String> = Vec::new();

        // optional class bound
        if let Some(&x) = it.peek() {
            it.next();
            if x == ':' {
                // empty class bound, this means that there is an implicit bound on java.lang.Object, but we can
                // skip it
                it.next_back();
            } else {
                // actual class bound
                it.next_back();
                generic_type_bounds.push(decode_type_it(it));
            }
        }

        // 0-N interface bounds
        while let Some(&x) = it.peek() {
            it.next();
            if x != ':' {
                it.next_back();
                break;
            }
            generic_type_bounds.push(decode_type_it(it));
        }

        generics.insert(generic_type_name, generic_type_bounds);
    }
    generics
}

fn expect(it: &mut Peekable<Chars>, expected: char) {
    let x = it.next().unwrap();
    assert_eq!(expected, x, "Expected '{}' but was '{}'.", expected, x);
}

pub fn decode_type(descriptor: &str) -> String {
    let mut s = String::new();
    let mut it = descriptor.chars().peekable();

    if descriptor.starts_with('<') {
        let generic_mappings: BTreeMap<String, Vec<String>> = decode_generics(&mut it);
        s.push('<');
        s.push_str(
            &generic_mappings
                .iter()
                .map(|(key, value)| format!("{} extends {}", key, value.join(" & ")))
                .collect::<Vec<String>>()
                .join(", "),
        );
        s.push('>');
        s.push(' ');
    }

    while let Some(&_) = it.peek() {
        s.push_str(&decode_type_it(&mut it));
    }
    s
}

fn split_class_name(it: &mut Peekable<Chars>) -> String {
    let mut s = String::new();
    let mut n_generics = 0;
    while let Some(&x) = it.peek() {
        it.next();
        s.push(x);

        if x == ';' && n_generics == 0 {
            break;
        } else if x == '<' {
            n_generics += 1;
        } else if x == '>' {
            n_generics -= 1;
            if n_generics < 0 {
                panic!();
            }
        }
    }
    s
}

#[derive(PartialEq, Debug)]
pub struct ClassSignature {
    pub super_class_name: String,
    interfaces: Vec<String>,
}

pub fn decode_class_signature(class_signature: &str) -> ClassSignature {
    let mut it = class_signature.chars().peekable();
    let super_class_name = decode_type(&split_class_name(&mut it));
    let mut interfaces: Vec<String> = Vec::new();

    while let Some(&_) = it.peek() {
        interfaces.push(decode_type(&split_class_name(&mut it)));
    }

    ClassSignature {
        super_class_name,
        interfaces,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_signatures() {
        let cases = [
            ("V", "void"),
            ("Z", "boolean"),
            ("B", "byte"),
            ("S", "short"),
            ("I", "int"),
            ("J", "long"),
            ("F", "float"),
            ("D", "double"),
            ("C", "char"),
            //
            ("[B", "byte[]"),
            ("[Z", "boolean[]"),
            ("[S", "short[]"),
            ("[I", "int[]"),
            ("[J", "long[]"),
            ("[F", "float[]"),
            ("[D", "double[]"),
            ("[C", "char[]"),
            //
            ("[[B", "byte[][]"),
            ("[[Z", "boolean[][]"),
            ("[[S", "short[][]"),
            ("[[I", "int[][]"),
            ("[[J", "long[][]"),
            ("[[F", "float[][]"),
            ("[[D", "double[][]"),
            ("[[C", "char[][]"),
            //
            ("Ljava/lang/Object;", "java.lang.Object"),
            ("[Ljava/lang/String;", "java.lang.String[]"),
            (
                "[[La/very/long/package/name/followed/by/another/much/longer/class/name/ProjectContractChargingPeriodProjectAccountReferenceVMFactoryBuilderStrategy;",
                "a.very.long.package.name.followed.by.another.much.longer.class.name.ProjectContractChargingPeriodProjectAccountReferenceVMFactoryBuilderStrategy[][]",
            ),
            //
            (
                "Ljava/util/List<Ljava/lang/String;>;",
                "java.util.List<java.lang.String>",
            ),
            (
                "Ljava/util/Map<Ljava/lang/String;Ljava/lang/Integer;>;",
                "java.util.Map<java.lang.String, java.lang.Integer>",
            ),
            (
                "Lmy/personal/Class<Ljava/lang/String;[Ljava/lang/Integer;[[Ljava/lang/Long;>;",
                "my.personal.Class<java.lang.String, java.lang.Integer[], java.lang.Long[][]>",
            ),
            (
                "Ljava/util/List<Ljava/util/List<Ljava/lang/String;>;>;",
                "java.util.List<java.util.List<java.lang.String>>",
            ),
            (
                "Ljava/util/List<Ljava/util/List<Ljava/util/List<Ljava/lang/String;>;>;>;",
                "java.util.List<java.util.List<java.util.List<java.lang.String>>>",
            ),
            (
                "Ljava/util/Map<Ljava/util/Map<Ljava/lang/String;Ljava/lang/Integer;>;Ljava/util/Map<Ljava/lang/Float;Ljava/lang/Double;>;>;",
                "java.util.Map<java.util.Map<java.lang.String, java.lang.Integer>, java.util.Map<java.lang.Float, java.lang.Double>>",
            ),
            //
            ("()V", "void()"),
            ("()Ljava/lang/String;", "java.lang.String()"),
            ("(I)S", "short(int)"),
            ("(IFS)D", "double(int, float, short)"),
            (
                "([ZI[CJ[[S)[[[C",
                "char[][][](boolean[], int, char[], long, short[][])",
            ),
            (
                "(Ljava/lang/Object;ILjava/lang/String;)Ljava/util/List;",
                "java.util.List(java.lang.Object, int, java.lang.String)",
            ),
            (
                "(ILjava/util/List<Ljava/lang/String;>;I)Ljava/util/List<Ljava/lang/String;>;",
                "java.util.List<java.lang.String>(int, java.util.List<java.lang.String>, int)",
            ),
            // generic methods
            (
                "<X:Ljava/lang/Object;>(Ljava/lang/String;TX;)TX;",
                "<X extends java.lang.Object> X(java.lang.String, X)",
            ),
            (
                "<K:Ljava/lang/Object;V:Ljava/lang/Integer;>Ljava/lang/String;",
                "<K extends java.lang.Object, V extends java.lang.Integer> java.lang.String",
            ),
            (
                "(Ljava.lang.String;)Ljava/util/Set<Ljava.util.List<*>;>;",
                "java.util.Set<java.util.List<?>>(java.lang.String)",
            ),
            (
                "(Ljava/util/Collection<+TX;>;)Z",
                "boolean(java.util.Collection<? extends X>)",
            ),
            (
                "<X::Ljava/io/Serializable;>(Ljava/lang/Class<TX;>;)Ljava/util/Optional<TX;>;",
                "<X extends java.io.Serializable> java.util.Optional<X>(java.lang.Class<X>)",
            ),
            (
                "<X::Ljava/io/Serializable;:Ljava/lang/Comparable;>(Ljava/lang/Class<TX;>;)Ljava/util/Optional<TX;>;",
                "<X extends java.io.Serializable & java.lang.Comparable> java.util.Optional<X>(java.lang.Class<X>)",
            ),
            (
                "<X:Ljava/lang/Integer;:Ljava/io/Serializable;:Ljava/lang/Comparable;>(Ljava/lang/Class<TX;>;)Ljava/util/Optional<TX;>;",
                "<X extends java.lang.Integer & java.io.Serializable & java.lang.Comparable> java.util.Optional<X>(java.lang.Class<X>)",
            ),
        ];

        for (input, expected) in cases {
            let actual = decode_type(input);
            assert_eq!(
                expected, actual,
                "Expected '{}' to be decoded into '{}' but was '{}'.",
                input, expected, actual
            );
        }
    }

    #[test]
    fn invalid_parsing() {
        let cases = [
            "Q",
            "[",
            "[]",
            "Ljava/lang/String",
            "java.lang.String",
            "I<Ljava/lang/String;>",
            "Ljava/util/List<I>;",
            "Ljava/util/List<Ljava/lang/String>;",
        ];

        for input in cases {
            let result = std::panic::catch_unwind(|| decode_type(input));
            assert!(
                result.is_err(),
                "Parsing of '{}' should have panicked but did not.",
                input
            );
        }
    }

    #[test]
    fn decode_class_signatures() {
        let cases = [
            (
                "Ljava/lang/Enum<Ljava/lang/String;>;",
                ClassSignature {
                    super_class_name: "java.lang.Enum<java.lang.String>".to_owned(),
                    interfaces: Vec::new(),
                },
            ),
            (
                "Ljava/lang/Object;Ljava/util/function/Supplier<Ljava/lang/String;>;",
                ClassSignature {
                    super_class_name: "java.lang.Object".to_owned(),
                    interfaces: vec!["java.util.function.Supplier<java.lang.String>".to_owned()],
                },
            ),
        ];

        for (input, expected) in cases {
            let actual = decode_class_signature(input);
            assert_eq!(
                expected, actual,
                "Expected class signature '{}' to be decoded into '{:?}' but was '{:?}'.",
                input, expected, actual
            );
        }
    }
}
