use std::env;
use std::io::Result;

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
    println!("  minor version: {}", classfile.minor_version);
    println!("  major version: {}", classfile.major_version);
    println!("  flags: (0x{:04x})", classfile.access_flags);
    println!("  this_class: #{}", classfile.this_class);
    println!("  super_class: #{}", classfile.super_class);
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
        match classfile.constant_pool[i] {
            ConstantPoolInfo::Utf8 { bytes: _ } => print!("Utf8"),
            ConstantPoolInfo::Long {
                high_bytes: _,
                low_bytes: _,
            } => print!("Long"),
            ConstantPoolInfo::Double {
                high_bytes: _,
                low_bytes: _,
            } => print!("Double"),
            ConstantPoolInfo::String { string_index: _ } => print!("String"),
            ConstantPoolInfo::Class { name_index: _ } => print!("Class"),
            ConstantPoolInfo::FieldRef {
                class_index: _,
                name_and_type_index: _,
            } => print!("Fieldref"),
            ConstantPoolInfo::MethodRef {
                class_index: _,
                name_and_type_index: _,
            } => print!("Methodref"),
            ConstantPoolInfo::InterfaceMethodRef {
                class_index: _,
                name_and_type_index: _,
            } => print!("InterfaceMethodref"),
            ConstantPoolInfo::NameAndType {
                name_index: _,
                descriptor_index: _,
            } => print!("NameAndType"),
            ConstantPoolInfo::MethodType {
                descriptor_index: _,
            } => print!("MethodType"),
            ConstantPoolInfo::MethodHandle {
                reference_kind: _,
                reference_index: _,
            } => print!("MethodHandle"),
            ConstantPoolInfo::InvokeDynamic {
                bootstrap_method_attr_index: _,
                name_and_type_index: _,
            } => print!("InvokeDynamic"),
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
