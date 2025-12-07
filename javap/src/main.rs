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
}

fn main() -> Result<()> {
    let filename = env::args().nth(1).expect("Usage: program <filename>");

    let classfile: ClassFile = parse_class_file(filename);

    print_class_file(&classfile);

    Ok(())
}
