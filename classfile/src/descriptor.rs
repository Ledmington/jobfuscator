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

fn collect_until(it: &mut Peekable<Chars>, stop: impl Fn(char) -> bool) -> String {
    let mut s = String::new();
    while let Some(&x) = it.peek() {
        if stop(x) {
            break;
        }
        it.next();
        s.push(if x == FORWARD_SLASH { DOT } else { x });
    }
    s
}

fn consume_class_name(it: &mut Peekable<Chars>) -> String {
    collect_until(it, |x| x == START_GENERIC || x == SEMICOLON)
}

/// Decodes a type variable declaration, for example `TX;`.
fn decode_type_variable(it: &mut Peekable<Chars>) -> String {
    expect(it, TYPE_VAR_START);
    let name = collect_until(it, |x| x == SEMICOLON);
    if name.is_empty() {
        panic!("Unterminated type variable.");
    }
    expect(it, SEMICOLON);
    name
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
        '-' => {
            it.next();
            "? super ".to_owned() + &decode_generic_arg(it)
        }

        // forbid everything else inside generics
        _ => panic!("Invalid generic argument type: '{ch}'."),
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

        _ => panic!("Unexpected '{ch}' at the start of type."),
    }
}

fn decode_ref_type(it: &mut Peekable<Chars>) -> String {
    expect(it, REFERENCE_START);

    let mut s = consume_class_name(it);

    // generics
    if it.peek() == Some(&START_GENERIC) {
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
#[derive(PartialEq, Debug)]
pub struct GenericTypeBound {
    /// Name of the generic type parameter (e.g. `X`).
    pub type_name: String,

    /// Fully qualified names of superclass and interfaces.
    /// The first entry is the superclass, if present.
    pub type_bounds: Vec<String>,
}

fn parse_generic_type_bounds(it: &mut Peekable<Chars>) -> Vec<GenericTypeBound> {
    expect(it, START_GENERIC);
    let mut generics = Vec::new();
    while let Some(&x) = it.peek() {
        if x == END_GENERIC {
            it.next();
            break;
        }

        let type_name = collect_until(it, |x| x == COLON);
        expect(it, COLON);

        let mut type_bounds: Vec<String> = Vec::new();

        // optional class bound
        if let Some(&x) = it.peek() {
            if x == COLON {
                // empty class bound — do NOT consume, the interface loop will handle it
            } else {
                // actual class bound
                type_bounds.push(decode_type_it(it));
            }
        }

        // 0-N interface bounds
        while let Some(&x) = it.peek() {
            if x != COLON {
                break;
            }
            it.next(); // consume ':'
            type_bounds.push(decode_type_it(it));
        }

        generics.push(GenericTypeBound {
            type_name,
            type_bounds,
        });
    }
    generics
}

/// Checks if the next character in the iterator is the given one and consumes it.
fn expect(it: &mut Peekable<Chars>, expected: char) {
    let x = it
        .next()
        .unwrap_or_else(|| panic!("Expected '{expected}' but found end of input."));
    assert_eq!(expected, x, "Expected '{expected}' but was '{x}'.");
}

fn format_generic_bound(gtb: &GenericTypeBound) -> String {
    assert!(!gtb.type_bounds.is_empty());
    format!(
        "{} extends {}",
        gtb.type_name,
        gtb.type_bounds
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" & ")
    )
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
                .map(format_generic_bound)
                .collect::<Vec<String>>()
                .join(", "),
        );
        s.push(END_GENERIC);
        s.push(' ');
    }

    while it.peek().is_some() {
        s.push_str(&decode_type_it(&mut it));
    }

    s
}

/// From the given iterator, this function extracts the full class name included with generic declarations from the raw descriptor.
/// For example, `Ljava/util/List<Ljava/lang/String;>;`.
fn split_class_name(it: &mut Peekable<Chars>) -> String {
    let mut s = String::new();
    let mut depth: usize = 0;
    while let Some(&x) = it.peek() {
        it.next();
        s.push(x);
        match x {
            SEMICOLON if depth == 0 => break,
            START_GENERIC => depth += 1,
            END_GENERIC => {
                depth = depth
                    .checked_sub(1)
                    .unwrap_or_else(|| panic!("Unexpected '{END_GENERIC}' in class name."))
            }
            _ => {}
        }
    }
    s
}

#[derive(PartialEq, Debug)]
pub struct ClassSignature {
    pub generic_type_bounds: Vec<GenericTypeBound>,
    pub super_class_name: String,
    pub interfaces: Vec<String>,
}

pub fn decode_class_signature(class_signature: &str) -> ClassSignature {
    let mut it = class_signature.chars().peekable();

    let generic_type_bounds = if it.peek() == Some(&START_GENERIC) {
        parse_generic_type_bounds(&mut it)
    } else {
        Vec::new()
    };

    let super_class_name = decode_type(&split_class_name(&mut it));
    let mut interfaces: Vec<String> = Vec::new();

    while it.peek().is_some() {
        interfaces.push(decode_type(&split_class_name(&mut it)));
    }

    ClassSignature {
        generic_type_bounds,
        super_class_name,
        interfaces,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("V", "void")]
    #[case("Z", "boolean")]
    #[case("B", "byte")]
    #[case("S", "short")]
    #[case("I", "int")]
    #[case("J", "long")]
    #[case("F", "float")]
    #[case("D", "double")]
    #[case("C", "char")]
    // single-dimensional primitive arrays
    #[case("[B", "byte[]")]
    #[case("[Z", "boolean[]")]
    #[case("[S", "short[]")]
    #[case("[I", "int[]")]
    #[case("[J", "long[]")]
    #[case("[F", "float[]")]
    #[case("[D", "double[]")]
    #[case("[C", "char[]")]
    // two-dimensional primitive arrays
    #[case("[[B", "byte[][]")]
    #[case("[[Z", "boolean[][]")]
    #[case("[[S", "short[][]")]
    #[case("[[I", "int[][]")]
    #[case("[[J", "long[][]")]
    #[case("[[F", "float[][]")]
    #[case("[[D", "double[][]")]
    #[case("[[C", "char[][]")]
    // object types
    #[case("Ljava/lang/Object;", "java.lang.Object")]
    #[case("[Ljava/lang/String;", "java.lang.String[]")]
    #[case(
        "[[La/very/long/package/name/followed/by/another/much/longer/class/name/ProjectContractChargingPeriodProjectAccountReferenceVMFactoryBuilderStrategy;",
        "a.very.long.package.name.followed.by.another.much.longer.class.name.ProjectContractChargingPeriodProjectAccountReferenceVMFactoryBuilderStrategy[][]"
    )]
    // generics
    #[case(
        "Ljava/util/List<Ljava/lang/String;>;",
        "java.util.List<java.lang.String>"
    )]
    #[case(
        "Ljava/util/List<[Ljava/lang/String;>;",
        "java.util.List<java.lang.String[]>"
    )]
    #[case(
        "Ljava/util/List<[[Ljava/lang/String;>;",
        "java.util.List<java.lang.String[][]>"
    )]
    #[case(
        "Ljava/util/Map<Ljava/lang/String;Ljava/lang/Integer;>;",
        "java.util.Map<java.lang.String, java.lang.Integer>"
    )]
    #[case(
        "Lmy/personal/Class<Ljava/lang/String;[Ljava/lang/Integer;[[Ljava/lang/Long;>;",
        "my.personal.Class<java.lang.String, java.lang.Integer[], java.lang.Long[][]>"
    )]
    #[case(
        "Ljava/util/List<Ljava/util/List<Ljava/lang/String;>;>;",
        "java.util.List<java.util.List<java.lang.String>>"
    )]
    #[case(
        "Ljava/util/List<Ljava/util/List<Ljava/util/List<Ljava/lang/String;>;>;>;",
        "java.util.List<java.util.List<java.util.List<java.lang.String>>>"
    )]
    #[case(
        "Ljava/util/Map<Ljava/util/Map<Ljava/lang/String;Ljava/lang/Integer;>;Ljava/util/Map<Ljava/lang/Float;Ljava/lang/Double;>;>;",
        "java.util.Map<java.util.Map<java.lang.String, java.lang.Integer>, java.util.Map<java.lang.Float, java.lang.Double>>"
    )]
    // method signatures
    #[case("()V", "void()")]
    #[case("()Ljava/lang/String;", "java.lang.String()")]
    #[case("(I)S", "short(int)")]
    #[case("(IFS)D", "double(int, float, short)")]
    #[case(
        "([ZI[CJ[[S)[[[C",
        "char[][][](boolean[], int, char[], long, short[][])"
    )]
    #[case(
        "(Ljava/lang/Object;ILjava/lang/String;)Ljava/util/List;",
        "java.util.List(java.lang.Object, int, java.lang.String)"
    )]
    #[case(
        "(ILjava/util/List<Ljava/lang/String;>;I)Ljava/util/List<Ljava/lang/String;>;",
        "java.util.List<java.lang.String>(int, java.util.List<java.lang.String>, int)"
    )]
    // generic methods
    #[case(
        "<X:Ljava/lang/Object;>(Ljava/lang/String;TX;)TX;",
        "<X extends java.lang.Object> X(java.lang.String, X)"
    )]
    #[case(
        "<K:Ljava/lang/Object;V:Ljava/lang/Integer;>Ljava/lang/String;",
        "<K extends java.lang.Object, V extends java.lang.Integer> java.lang.String"
    )]
    #[case(
        "(Ljava.lang.String;)Ljava/util/Set<Ljava.util.List<*>;>;",
        "java.util.Set<java.util.List<?>>(java.lang.String)"
    )]
    #[case(
        "(Ljava/util/Collection<+TX;>;)Z",
        "boolean(java.util.Collection<? extends X>)"
    )]
    #[case(
        "(Ljava/util/function/Predicate<-TT;>;)Ljava/util/stream/Stream<TT;>;",
        "java.util.stream.Stream<T>(java.util.function.Predicate<? super T>)"
    )]
    #[case(
        "<X::Ljava/io/Serializable;>(Ljava/lang/Class<TX;>;)Ljava/util/Optional<TX;>;",
        "<X extends java.io.Serializable> java.util.Optional<X>(java.lang.Class<X>)"
    )]
    #[case(
        "<X::Ljava/io/Serializable;:Ljava/lang/Comparable;>(Ljava/lang/Class<TX;>;)Ljava/util/Optional<TX;>;",
        "<X extends java.io.Serializable & java.lang.Comparable> java.util.Optional<X>(java.lang.Class<X>)"
    )]
    #[case(
        "<X:Ljava/lang/Integer;:Ljava/io/Serializable;:Ljava/lang/Comparable;>(Ljava/lang/Class<TX;>;)Ljava/util/Optional<TX;>;",
        "<X extends java.lang.Integer & java.io.Serializable & java.lang.Comparable> java.util.Optional<X>(java.lang.Class<X>)"
    )]
    fn decode_signatures(#[case] input: &str, #[case] expected: &str) {
        let actual = decode_type(input);
        assert_eq!(
            expected, actual,
            "Expected '{input}' to be decoded into '{expected}' but was '{actual}'."
        );
    }

    #[rstest]
    #[case("Q")]
    #[case("[")]
    #[case("[]")]
    #[case("[I]")]
    #[case("Ljava/lang/String")]
    #[case("java.lang.String")]
    #[case("I<Ljava/lang/String;>")]
    #[case("Ljava/util/List<I>;")]
    #[case("Ljava/util/List<Ljava/lang/String>;")]
    fn invalid_parsing(#[case] input: &str) {
        let result = std::panic::catch_unwind(|| decode_type(input));
        assert!(
            result.is_err(),
            "Parsing of '{input}' should have panicked but did not.",
        );
    }

    #[rstest]
    #[case(
        "Ljava/lang/Enum<Ljava/lang/String;>;",
        ClassSignature {
            generic_type_bounds: vec![],
            super_class_name: "java.lang.Enum<java.lang.String>".to_owned(),
            interfaces: vec![],
        }
    )]
    #[case(
        "Ljava/lang/Object;Ljava/util/function/Supplier<Ljava/lang/String;>;",
        ClassSignature {
            generic_type_bounds: vec![],
            super_class_name: "java.lang.Object".to_owned(),
            interfaces: vec!["java.util.function.Supplier<java.lang.String>".to_owned()],
        }
    )]
    #[case(
        "<T:Ljava/lang/Object;>Ljava/lang/Object;Ljava/util/stream/BaseStream<TT;Ljava/util/stream/Stream<TT;>;>;",
        ClassSignature {
            generic_type_bounds: vec![
                GenericTypeBound {
                    type_name: "T".to_owned(),
                    type_bounds: vec!["java.lang.Object".to_owned()]
                }
            ],
            super_class_name: "java.lang.Object".to_owned(),
            interfaces: vec!["java.util.stream.BaseStream<T, java.util.stream.Stream<T>>".to_owned()],
        }
    )]
    #[case(
        "<K:Ljava/lang/Object;V:Ljava/lang/Object;>Ljava/lang/Object;Ljava/util/Map<TK;TV;>;",
        ClassSignature {
            generic_type_bounds: vec![
                GenericTypeBound {
                    type_name: "K".to_owned(),
                    type_bounds: vec!["java.lang.Object".to_owned()]
                },
                GenericTypeBound {
                    type_name: "V".to_owned(),
                    type_bounds: vec!["java.lang.Object".to_owned()]
                }
            ],
            super_class_name: "java.lang.Object".to_owned(),
            interfaces: vec!["java.util.Map<K, V>".to_owned()],
        }
    )]
    #[case(
        "<E_IN:Ljava/lang/Object;E_OUT:Ljava/lang/Object;S::Ljava/util/stream/BaseStream<TE_OUT;TS;>;>Ljava/util/stream/PipelineHelper<TE_OUT;>;Ljava/util/stream/BaseStream<TE_OUT;TS;>;",
        ClassSignature {
            generic_type_bounds: vec![
                GenericTypeBound {
                    type_name: "E_IN".to_owned(),
                    type_bounds: vec!["java.lang.Object".to_owned()]
                },
                GenericTypeBound {
                    type_name: "E_OUT".to_owned(),
                    type_bounds: vec!["java.lang.Object".to_owned()]
                },
                GenericTypeBound {
                    type_name: "S".to_owned(),
                    type_bounds: vec!["java.util.stream.BaseStream<E_OUT, S>".to_owned()]
                }
            ],
            super_class_name: "java.util.stream.PipelineHelper<E_OUT>".to_owned(),
            interfaces: vec!["java.util.stream.BaseStream<E_OUT, S>".to_owned()],
        }
    )]
    fn decode_class_signatures(#[case] input: &str, #[case] expected: ClassSignature) {
        let actual = decode_class_signature(input);
        assert_eq!(
            expected, actual,
            "Expected class signature '{input}' to be decoded into '{expected:?}' but was '{actual:?}'."
        );
    }
}
