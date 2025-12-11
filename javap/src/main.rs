use std::io::Result;
use std::{env, path::MAIN_SEPARATOR};

use classfile::constant_pool::ConstantPoolInfo;
use classfile::fields::FieldInfo;
use classfile::methods::MethodInfo;
use classfile::{ClassFile, parse_class_file};

/**
 * The index of the column (on the terminal) where the index of each constant pool entry ends.
 */
const CP_INDEX_WIDTH: usize = 6;

/**
 * The index of the column (on the terminal) where the information of each entry is displayed.
 */
const CP_INFO_START_INDEX: usize = 28;

/**
 * The index of the column (on the terminal) where the comments (the '//') starts
 */
const CP_COMMENT_START_INDEX: usize = 42;

fn print_class_file(cf: &ClassFile) {
    print_header(cf);
    print_constant_pool(cf);
    println!("{{");
    print_fields(cf);
    print_methods(cf);
    println!("}}");
}

fn print_header(cf: &ClassFile) {
    println!("Classfile {}", cf.absolute_file_path);
    println!("  Last modified Dec 5, 2025; size 12150 bytes");
    println!(
        "  SHA-256 checksum {}",
        cf.sha256_digest
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect::<Vec<String>>()
            .concat()
    );
    println!(
        "  Compiled from \"{}\"",
        cf.absolute_file_path
            .split(MAIN_SEPARATOR)
            .next_back()
            .unwrap()
            .split('.')
            .next()
            .unwrap()
            .to_owned()
            + ".java"
    );
    println!(
        "{} {}",
        cf.access_flags
            .iter()
            .map(|f| classfile::access_flags::modifier_repr(*f))
            .collect::<Vec<String>>()
            .join(" "),
        cf.constant_pool
            .get_class_name(cf.this_class)
            .replace('/', ".")
    );
    println!("  minor version: {}", cf.minor_version);
    println!("  major version: {}", cf.major_version);
    println!(
        "  flags: (0x{:04x}) {}",
        cf.access_flags
            .iter()
            .map(|f| *f as u16)
            .reduce(|a, b| a | b)
            .unwrap(),
        cf.access_flags
            .iter()
            .map(|f| classfile::access_flags::java_repr(*f))
            .collect::<Vec<String>>()
            .join(", ")
    );
    println!(
        "{:<width$}// {}",
        format!("  this_class: #{}", cf.this_class),
        cf.constant_pool.get_class_name(cf.this_class),
        width = CP_COMMENT_START_INDEX,
    );
    println!(
        "{:<width$}// {}",
        format!("  super_class: #{}", cf.super_class),
        cf.constant_pool.get_class_name(cf.super_class),
        width = CP_COMMENT_START_INDEX
    );
    println!(
        " interfaces: {}, fields: {}, methods: {}, attributes: {}",
        cf.interfaces.len(),
        cf.fields.len(),
        cf.methods.len(),
        cf.attributes.len()
    );
}

fn print_constant_pool(cf: &ClassFile) {
    println!("Constant pool:");
    for i in 0..cf.constant_pool.len() {
        if i > 0
            && (matches!(cf.constant_pool[i - 1], ConstantPoolInfo::Long { .. })
                || matches!(cf.constant_pool[i - 1], ConstantPoolInfo::Double { .. }))
        {
            continue;
        }

        match &cf.constant_pool[i] {
            ConstantPoolInfo::Utf8 { bytes } => {
                println!(
                    "{:<width$}{}",
                    format!(
                        "{:>width$} = Utf8",
                        format!("#{}", i + 1),
                        width = CP_INDEX_WIDTH
                    ),
                    classfile::constant_pool::convert_utf8(bytes),
                    width = CP_INFO_START_INDEX
                )
            }
            ConstantPoolInfo::Long {
                high_bytes,
                low_bytes,
            } => println!(
                "{:<width$}{}l",
                format!(
                    "{:>width$} = Long",
                    format!("#{}", i + 1),
                    width = CP_INDEX_WIDTH
                ),
                ((*high_bytes as u64) << 32) | (*low_bytes as u64),
                width = CP_INFO_START_INDEX
            ),
            ConstantPoolInfo::Double {
                high_bytes: _,
                low_bytes: _,
            } => print!("Double"),
            ConstantPoolInfo::String { string_index } => println!(
                "{:<width$}// {}",
                format!(
                    "{:<width$}#{}",
                    format!(
                        "{:>width$} = String",
                        format!("#{}", i + 1),
                        width = CP_INDEX_WIDTH
                    ),
                    string_index,
                    width = CP_INFO_START_INDEX
                ),
                cf.constant_pool.get_utf8_content(*string_index),
                width = CP_COMMENT_START_INDEX
            ),
            ConstantPoolInfo::Class { name_index } => println!(
                "{:<width$}// {}",
                format!(
                    "{:<width$}#{}",
                    format!(
                        "{:>width$} = Class",
                        format!("#{}", i + 1),
                        width = CP_INDEX_WIDTH
                    ),
                    name_index,
                    width = CP_INFO_START_INDEX
                ),
                cf.constant_pool.get_utf8_content(*name_index),
                width = CP_COMMENT_START_INDEX
            ),
            ConstantPoolInfo::FieldRef {
                class_index,
                name_and_type_index,
            } => println!(
                "{:<width$}// {}.{}",
                format!(
                    "{:<width$}#{}.#{}",
                    format!(
                        "{:>width$} = Fieldref",
                        format!("#{}", i + 1),
                        width = CP_INDEX_WIDTH
                    ),
                    class_index,
                    name_and_type_index,
                    width = CP_INFO_START_INDEX
                ),
                cf.constant_pool.get_class_name(*class_index),
                cf.constant_pool.get_name_and_type(*name_and_type_index),
                width = CP_COMMENT_START_INDEX
            ),
            ConstantPoolInfo::MethodRef {
                class_index,
                name_and_type_index,
            } => println!(
                "{:<width$}// {}",
                format!(
                    "{:<width$}#{}.#{}",
                    format!(
                        "{:>width$} = Methodref",
                        format!("#{}", i + 1),
                        width = CP_INDEX_WIDTH
                    ),
                    class_index,
                    name_and_type_index,
                    width = CP_INFO_START_INDEX
                ),
                cf.constant_pool
                    .get_method_ref_string(*class_index, *name_and_type_index),
                width = CP_COMMENT_START_INDEX
            ),
            ConstantPoolInfo::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => println!(
                "{:<width$}// {}",
                format!(
                    "{:<width$}#{}.#{}",
                    format!(
                        "{:>width$} = InterfaceMethodref",
                        format!("#{}", i + 1),
                        width = CP_INDEX_WIDTH
                    ),
                    class_index,
                    name_and_type_index,
                    width = CP_INFO_START_INDEX
                ),
                cf.constant_pool
                    .get_method_ref_string(*class_index, *name_and_type_index),
                width = CP_COMMENT_START_INDEX
            ),
            ConstantPoolInfo::NameAndType {
                name_index,
                descriptor_index,
            } => println!(
                "{:<width$}// {}",
                format!(
                    "{:<width$}#{}:#{}",
                    format!(
                        "{:>width$} = NameAndType",
                        format!("#{}", i + 1),
                        width = CP_INDEX_WIDTH
                    ),
                    name_index,
                    descriptor_index,
                    width = CP_INFO_START_INDEX
                ),
                cf.constant_pool
                    .get_name_and_type_string(*name_index, *descriptor_index),
                width = CP_COMMENT_START_INDEX
            ),
            ConstantPoolInfo::MethodType { descriptor_index } => println!(
                "{:<width$}//  {}",
                format!(
                    "{:<width$}#{}",
                    format!(
                        "{:>width$} = MethodType",
                        format!("#{}", i + 1),
                        width = CP_INDEX_WIDTH
                    ),
                    descriptor_index,
                    width = CP_INFO_START_INDEX
                ),
                cf.constant_pool.get_utf8_content(*descriptor_index),
                width = CP_COMMENT_START_INDEX
            ),
            ConstantPoolInfo::MethodHandle {
                reference_kind,
                reference_index,
            } => println!(
                "{:<width$}// {} {}",
                format!(
                    "{:<width$}{}:#{}",
                    format!(
                        "{:>width$} = MethodHandle",
                        format!("#{}", i + 1),
                        width = CP_INDEX_WIDTH
                    ),
                    *reference_kind as u8,
                    reference_index,
                    width = CP_INFO_START_INDEX
                ),
                classfile::reference_kind::java_repr(*reference_kind),
                cf.constant_pool.get_method_ref(*reference_index),
                width = CP_COMMENT_START_INDEX
            ),
            ConstantPoolInfo::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => println!(
                "{:<width$}// #{}:{}",
                format!(
                    "{:<width$}#{}:#{}",
                    format!(
                        "{:>width$} = InvokeDynamic",
                        format!("#{}", i + 1),
                        width = CP_INDEX_WIDTH
                    ),
                    bootstrap_method_attr_index,
                    name_and_type_index,
                    width = CP_INFO_START_INDEX
                ),
                bootstrap_method_attr_index,
                cf.constant_pool.get_name_and_type(*name_and_type_index),
                width = CP_COMMENT_START_INDEX
            ),
            ConstantPoolInfo::Null {} => unreachable!(),
        }
    }
}

fn print_fields(cf: &ClassFile) {
    for i in 0..cf.fields.len() {
        let field: &FieldInfo = &cf.fields[i];
        let descriptor: String = cf.constant_pool.get_utf8_content(field.descriptor_index);
        println!(
            "  {} {} {};",
            field
                .access_flags
                .iter()
                .map(|f| classfile::access_flags::modifier_repr(*f))
                .collect::<Vec<String>>()
                .join(" "),
            classfile::convert_descriptor(descriptor.clone()),
            cf.constant_pool.get_utf8_content(field.name_index)
        );
        println!("    descriptor: {}", descriptor);
        println!(
            "    flags: (0x{:04x}) {}",
            field
                .access_flags
                .iter()
                .map(|f| *f as u16)
                .reduce(|a, b| a | b)
                .unwrap(),
            field
                .access_flags
                .iter()
                .map(|f| classfile::access_flags::java_repr(*f))
                .collect::<Vec<String>>()
                .join(", ")
        );
        println!();
    }
}

fn print_methods(cf: &ClassFile) {
    for i in 0..cf.methods.len() {
        let method: &MethodInfo = &cf.methods[i];
        let descriptor: String = cf.constant_pool.get_utf8_content(method.descriptor_index);
        println!(
            "  {} {}",
            method
                .access_flags
                .iter()
                .map(|f| classfile::access_flags::modifier_repr(*f))
                .collect::<Vec<String>>()
                .join(" "),
            classfile::convert_descriptor(descriptor.clone())
        );
        println!("    descriptor: {}", descriptor);
        println!(
            "    flags: (0x{:04x}) {}",
            method
                .access_flags
                .iter()
                .map(|f| *f as u16)
                .reduce(|a, b| a | b)
                .unwrap(),
            method
                .access_flags
                .iter()
                .map(|f| classfile::access_flags::java_repr(*f))
                .collect::<Vec<String>>()
                .join(", ")
        );
        println!();
    }
}

fn main() -> Result<()> {
    let filename = env::args().nth(1).expect("Usage: program <filename>");

    let classfile: ClassFile = parse_class_file(filename);

    print_class_file(&classfile);

    Ok(())
}
