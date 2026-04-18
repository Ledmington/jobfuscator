#![forbid(unsafe_code)]

use std::{iter::Peekable, str::Chars};

const START_GENERIC: char = '<';
const END_GENERIC: char = '>';
const SEMICOLON: char = ';';
const COLON: char = ':';
const FORWARD_SLASH: char = '/';
const DOT: char = '.';
const REFERENCE_START: char = 'L';
const TYPE_VAR_START: char = 'T';
const LEFT_SQUARE_BRACKET: char = '[';
const LEFT_BRACKET: char = '(';
const RIGHT_BRACKET: char = ')';

fn consume_class_name(it: &mut Peekable<Chars>) -> String {
    let mut s = String::new();

    while let Some(&x) = it.peek() {
        if x == START_GENERIC || x == SEMICOLON {
            break;
        }
        it.next();
        s.push(if x == FORWARD_SLASH { DOT } else { x });
    }

    s
}

/// Decodes a type variable declaration, for example `TX;`.
fn decode_type_variable(it: &mut Peekable<Chars>) -> String {
    expect(it, TYPE_VAR_START);
    let mut s = String::new();
    while let Some(&x) = it.peek() {
        if x == SEMICOLON {
            it.next();
            return s;
        }
        it.next();
        s.push(if x == FORWARD_SLASH { DOT } else { x });
    }
    panic!("Unterminated type variable: '{}'.", s);
}

fn decode_generic_arg(it: &mut Peekable<Chars>) -> String {
    let ch = it
        .peek()
        .expect("Expected generic argument but found end of input.");
    match *ch {
        REFERENCE_START => decode_ref_type(it),
        LEFT_SQUARE_BRACKET => {
            it.next();
            let inner = decode_generic_arg(it);
            inner + "[]"
        }
        TYPE_VAR_START => decode_type_variable(it),
        '*' => {
            it.next();
            "?".to_owned()
        }
        '+' => {
            it.next();
            "? extends ".to_owned() + &decode_generic_arg(it)
        }

        // forbid everything else inside generics
        _ => panic!("Invalid generic argument type: '{}'.", ch),
    }
}

fn decode_type_it(it: &mut Peekable<Chars>) -> String {
    let ch = it
        .peek()
        .expect("Expected start of type but found end of input.");
    match *ch {
        'B' => {
            it.next();
            "byte".to_owned()
        }
        'C' => {
            it.next();
            "char".to_owned()
        }
        'D' => {
            it.next();
            "double".to_owned()
        }
        'F' => {
            it.next();
            "float".to_owned()
        }
        'I' => {
            it.next();
            "int".to_owned()
        }
        'J' => {
            it.next();
            "long".to_owned()
        }
        'S' => {
            it.next();
            "short".to_owned()
        }
        'Z' => {
            it.next();
            "boolean".to_owned()
        }
        'V' => {
            it.next();
            "void".to_owned()
        }

        REFERENCE_START => decode_ref_type(it),

        LEFT_SQUARE_BRACKET => {
            it.next();
            let inner = decode_type_it(it);
            inner + "[]"
        }

        LEFT_BRACKET => {
            it.next();
            let mut args = Vec::new();
            while let Some(&x) = it.peek() {
                if x == RIGHT_BRACKET {
                    it.next();
                    break;
                }
                args.push(decode_type_it(it));
            }

            let ret = decode_type_it(it);
            format!("{}({})", ret, args.join(", "))
        }

        TYPE_VAR_START => decode_type_variable(it),

        _ => panic!("Unexpected '{}' at the start of type.", ch),
    }
}

fn decode_ref_type(it: &mut Peekable<Chars>) -> String {
    expect(it, REFERENCE_START);

    let mut s = consume_class_name(it);

    // generics
    if let Some(&START_GENERIC) = it.peek() {
        it.next();
        s.push(START_GENERIC);

        let mut first = true;

        while let Some(&x) = it.peek() {
            if x == END_GENERIC {
                it.next();
                break;
            }

            if !first {
                s.push_str(", ");
            }
            first = false;

            s.push_str(&decode_generic_arg(it));
        }

        s.push(END_GENERIC);
    }

    expect(it, SEMICOLON);
    s
}

/// Describes the bounds of a generic type parameter in a JVM-style descriptor.
///
/// For example, the descriptor:
/// `<X:Ljava/lang/String;:Ljava/io/Serializable;>`
///
/// corresponds to:
/// `X extends java.lang.String implements java.io.Serializable`.
///
/// If no explicit superclass is provided, `java.lang.Object` is implied.
struct GenericTypeBound {
    /// Name of the generic type parameter (e.g. `X`).
    type_name: String,

    /// Fully qualified name of the superclass.
    ///
    /// If empty, this implies `java.lang.Object`.
    super_class_name: Option<String>,

    /// Fully qualified names of implemented interfaces.
    interfaces: Vec<String>,
}

fn parse_generic_type_bounds(it: &mut Peekable<Chars>) -> Vec<GenericTypeBound> {
    expect(it, START_GENERIC);
    let mut generics = Vec::new();
    while let Some(&x) = it.peek() {
        if x == END_GENERIC {
            it.next();
            break;
        }

        let mut s = String::new();
        while let Some(&x) = it.peek() {
            if x == COLON {
                it.next();
                break;
            }

            s.push(x);
            it.next();
        }
        let type_name = s;

        // optional class bound
        let mut super_class_name: Option<String> = None;
        if let Some(&x) = it.peek() {
            if x == COLON {
                // empty class bound, this means that there is an implicit bound on java.lang.Object, but we can
                // skip it
                it.next();
            } else {
                // actual class bound
                super_class_name = Some(decode_type_it(it));
            }
        }

        // 0-N interface bounds
        let mut interfaces: Vec<String> = Vec::new();
        while let Some(&x) = it.peek() {
            if x != COLON {
                break;
            }
            it.next(); // consume ':'
            interfaces.push(decode_type_it(it));
        }

        generics.push(GenericTypeBound {
            type_name,
            super_class_name,
            interfaces,
        });
    }
    generics
}

/// Checks if the next character in the iterator is the given one and consumes it.
fn expect(it: &mut Peekable<Chars>, expected: char) {
    let x = it.next().unwrap();
    assert_eq!(expected, x, "Expected '{}' but was '{}'.", expected, x);
}

pub fn decode_type(descriptor: &str) -> String {
    let mut s = String::new();
    let mut it = descriptor.chars().peekable();

    if descriptor.starts_with(START_GENERIC) {
        let generic_type_bounds: Vec<GenericTypeBound> = parse_generic_type_bounds(&mut it);
        s.push(START_GENERIC);
        s.push_str(
            &generic_type_bounds
                .iter()
                .map(|gtb| {
                    let mut tmp: Vec<String> = Vec::new();
                    if let Some(scn) = &gtb.super_class_name {
                        tmp.push(scn.to_string());
                    }
                    for x in &gtb.interfaces {
                        tmp.push(x.to_string());
                    }
                    format!("{} extends {}", gtb.type_name, tmp.join(" & "))
                })
                .collect::<Vec<String>>()
                .join(", "),
        );
        s.push(END_GENERIC);
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

        if x == SEMICOLON && n_generics == 0 {
            break;
        } else if x == START_GENERIC {
            n_generics += 1;
        } else if x == END_GENERIC {
            n_generics -= 1;
            if n_generics < 0 {
                panic!("Unexpected '{}' in class name.", END_GENERIC);
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
                "Ljava/util/List<[Ljava/lang/String;>;",
                "java.util.List<java.lang.String[]>",
            ),
            (
                "Ljava/util/List<[[Ljava/lang/String;>;",
                "java.util.List<java.lang.String[][]>",
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
            "[I]",
            "Ljava/lang/String",
            "java.lang.String",
            "I<Ljava/lang/String;>",
            "Ljava/util/List<I>;",
            "Ljava/util/List<Ljava/lang/String>;",
        ];

        for input in cases {
            // This exchanging of hooks is to catch any panic/unwind that happens and avoid printing anything on stdout unless there was not a panic.
            let default_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(|_| {}));
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| decode_type(input)));
            std::panic::set_hook(default_hook);
            assert!(
                result.is_err(),
                "Parsing of '{}' should have panicked but did not.",
                input
            );
        }
    }

    // #[test]
    // fn decode_class_signatures() {
    //     let cases = [
    //         (
    //             "Ljava/lang/Enum<Ljava/lang/String;>;",
    //             ClassSignature {
    //                 super_class_name: "java.lang.Enum<java.lang.String>".to_owned(),
    //                 interfaces: Vec::new(),
    //             },
    //         ),
    //         (
    //             "Ljava/lang/Object;Ljava/util/function/Supplier<Ljava/lang/String;>;",
    //             ClassSignature {
    //                 super_class_name: "java.lang.Object".to_owned(),
    //                 interfaces: vec!["java.util.function.Supplier<java.lang.String>".to_owned()],
    //             },
    //         ),
    //         (
    //             "<T:Ljava/lang/Object;>Ljava/lang/Object;Ljava/util/stream/BaseStream<TT;Ljava/util/stream/Stream<TT;>;>;",
    //             ClassSignature {
    //                 super_class_name: "java.util.stream.BaseStream<T, java.util.stream.Stream<T>>"
    //                     .to_owned(),
    //                 interfaces: Vec::new(),
    //             },
    //         ),
    //         (
    //             "<K:Ljava/lang/Object;V:Ljava/lang/Object;>Ljava/lang/Object;Ljava/util/Map<TK;TV;>;",
    //             ClassSignature {
    //                 super_class_name: "java.util.Object".to_owned(),
    //                 interfaces: vec!["java.util.map<K, V>".to_owned()],
    //             },
    //         ),
    //         (
    //             "<E_IN:Ljava/lang/Object;E_OUT:Ljava/lang/Object;S::Ljava/util/stream/BaseStream<TE_OUT;TS;>;>Ljava/util/stream/PipelineHelper<TE_OUT;>;Ljava/util/stream/BaseStream<TE_OUT;TS;>;",
    //             ClassSignature {
    //                 super_class_name: "java.util.stream.PipelineHelper<E_OUT>".to_owned(),
    //                 interfaces: vec!["java.util.stream.BaseStream<E_OUT, S>".to_owned()],
    //             },
    //         ),
    //     ];

    //     for (input, expected) in cases {
    //         let actual = decode_class_signature(input);
    //         assert_eq!(
    //             expected, actual,
    //             "Expected class signature '{}' to be decoded into '{:?}' but was '{:?}'.",
    //             input, expected, actual
    //         );
    //     }
    // }
}
