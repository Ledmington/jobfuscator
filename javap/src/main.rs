use std::io::Result;
use std::{env, path::MAIN_SEPARATOR};

use classfile::{ClassFile, ConstantPoolInfo, parse_class_file};

fn print_class_file(classfile: &ClassFile) {
    println!("Classfile {}", classfile.absolute_file_path);
    println!("  Last modified Dec 5, 2025; size 12150 bytes");
    println!(
        "  SHA-256 checksum {}",
        classfile
            .sha256_digest
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect::<Vec<String>>()
            .concat()
    );
    println!(
        "  Compiled from \"{}\"",
        classfile
            .absolute_file_path
            .split(MAIN_SEPARATOR)
            .next_back()
            .unwrap()
            .split('.')
            .next()
            .unwrap()
            .to_owned()
            + ".java"
    );
    println!("  minor version: {}", classfile.minor_version);
    println!("  major version: {}", classfile.major_version);
    println!(
        "  flags: (0x{:04x}) {}",
        classfile
            .access_flags
            .iter()
            .map(|f| *f as u16)
            .reduce(|a, b| a | b)
            .unwrap(),
        classfile
            .access_flags
            .iter()
            .map(|f| classfile::access_flags::java_repr(*f))
            .collect::<Vec<String>>()
            .join(", ")
    );
    println!(
        "  this_class: #{}                         // {}",
        classfile.this_class,
        classfile.get_class_name(classfile.this_class)
    );
    println!(
        "  super_class: #{}                         // {}",
        classfile.super_class,
        classfile.get_class_name(classfile.super_class)
    );
    println!(
        " interfaces: {}, fields: {}, methods: {}, attributes: {}",
        classfile.interfaces.len(),
        classfile.fields.len(),
        classfile.methods.len(),
        classfile.attributes.len()
    );
    println!("Constant pool:");
    for i in 0..classfile.constant_pool.len() {
        if i > 0
            && (matches!(
                classfile.constant_pool[i - 1],
                ConstantPoolInfo::Long { .. }
            ) || matches!(
                classfile.constant_pool[i - 1],
                ConstantPoolInfo::Double { .. }
            ))
        {
            continue;
        }
        print!("  #{} = ", i + 1);
        match &classfile.constant_pool[i] {
            ConstantPoolInfo::Utf8 { bytes } => {
                print!("Utf8               {}", classfile::convert_utf8(bytes))
            }
            ConstantPoolInfo::Long {
                high_bytes,
                low_bytes,
            } => print!(
                "Long               {}l",
                ((*high_bytes as u64) << 32) | (*low_bytes as u64)
            ),
            ConstantPoolInfo::Double {
                high_bytes: _,
                low_bytes: _,
            } => print!("Double"),
            ConstantPoolInfo::String { string_index } => {
                print!(
                    "String             #{}           // {}",
                    string_index,
                    classfile.get_utf8_content(*string_index)
                )
            }
            ConstantPoolInfo::Class { name_index } => print!(
                "Class              #{}            // {}",
                name_index,
                classfile.get_utf8_content(*name_index)
            ),
            ConstantPoolInfo::FieldRef {
                class_index,
                name_and_type_index,
            } => print!(
                "Fieldref           #{}.#{}         // {}.{}",
                class_index,
                name_and_type_index,
                classfile.get_class_name(*class_index),
                classfile.get_name_and_type(*name_and_type_index)
            ),
            ConstantPoolInfo::MethodRef {
                class_index,
                name_and_type_index,
            } => print!(
                "Methodref          #{}.#{}         // {}.{}",
                class_index,
                name_and_type_index,
                classfile.get_class_name(*class_index),
                classfile.get_name_and_type(*name_and_type_index)
            ),
            ConstantPoolInfo::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => print!(
                "InterfaceMethodref #{}.#{}",
                class_index, name_and_type_index
            ),
            ConstantPoolInfo::NameAndType {
                name_index,
                descriptor_index,
            } => print!(
                "NameAndType        #{}:#{}       // {}",
                name_index,
                descriptor_index,
                classfile.get_name_and_type_string(*name_index, *descriptor_index)
            ),
            ConstantPoolInfo::MethodType {
                descriptor_index: _,
            } => print!("MethodType"),
            ConstantPoolInfo::MethodHandle {
                reference_kind: _,
                reference_index: _,
            } => print!("MethodHandle"),
            ConstantPoolInfo::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => print!(
                "InvokeDynamic      #{}:#{}",
                bootstrap_method_attr_index, name_and_type_index
            ),
            ConstantPoolInfo::Null {} => unreachable!(),
        }
        println!();
    }
    println!("{{");
}

fn main() -> Result<()> {
    let filename = env::args().nth(1).expect("Usage: program <filename>");

    let classfile: ClassFile = parse_class_file(filename);

    print_class_file(&classfile);

    Ok(())
}
