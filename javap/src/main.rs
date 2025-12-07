use std::env;
use std::io::Result;

use classfile::{ClassFile, parse_class_file};

fn print_class_file(classfile: &ClassFile) {
    println!("Classfile {}", classfile.absolute_file_path);
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
        print!("  #{} = ", i);
        match classfile.constant_pool[i] {
            classfile::ConstantPoolInfo::Utf8 { bytes: _ } => print!("Utf8"),
            classfile::ConstantPoolInfo::Long {
                high_bytes: _,
                low_bytes: _,
            } => print!("Long"),
            classfile::ConstantPoolInfo::String { string_index: _ } => print!("String"),
            classfile::ConstantPoolInfo::Class { name_index: _ } => print!("Class"),
            classfile::ConstantPoolInfo::FieldRef {
                class_index: _,
                name_and_type_index: _,
            } => print!("Fieldref"),
            classfile::ConstantPoolInfo::MethodRef {
                class_index: _,
                name_and_type_index: _,
            } => print!("Methodref"),
            classfile::ConstantPoolInfo::InterfaceMethodRef {
                class_index: _,
                name_and_type_index: _,
            } => print!("InterfaceMethodref"),
            classfile::ConstantPoolInfo::NameAndType {
                name_index: _,
                descriptor_index: _,
            } => print!("NameAndType"),
            classfile::ConstantPoolInfo::MethodType {
                descriptor_index: _,
            } => print!("MethodType"),
            classfile::ConstantPoolInfo::MethodHandle {
                reference_kind: _,
                reference_index: _,
            } => print!("MethodHandle"),
            classfile::ConstantPoolInfo::InvokeDynamic {
                bootstrap_method_attr_index: _,
                name_and_type_index: _,
            } => print!("InvokeDynamic"),
        }
        println!();
    }
}

fn main() -> Result<()> {
    let filename = env::args().nth(1).expect("Usage: program <filename>");

    let classfile: ClassFile = parse_class_file(filename);

    print_class_file(&classfile);

    Ok(())
}
