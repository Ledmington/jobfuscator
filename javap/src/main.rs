#![forbid(unsafe_code)]

use std::env;
use std::io::Result;

use classfile::attributes::{AttributeInfo, StackMapFrame, VerificationTypeInfo};
use classfile::bytecode::BytecodeInstruction;
use classfile::constant_pool::{self, ConstantPool, ConstantPoolInfo};
use classfile::fields::FieldInfo;
use classfile::methods::MethodInfo;
use classfile::{ClassFile, access_flags, parse_class_file, reference_kind};
use time::OffsetDateTime;

/**
 * The index of the column (on the terminal) where the index of each constant pool entry ends.
 */
const CP_INDEX_WIDTH: usize = 6;

/**
 * The index of the column (on the terminal) where the information of each entry is displayed.
 */
const CP_INFO_START_INDEX: usize = 28;

/**
 * The index of the column (on the terminal) where the comments (the '//') start for the constant pool.
 */
const CP_COMMENT_START_INDEX: usize = 42;

/**
 * The index of the column (on the terminal) where the comments (the '//') start for the bytecode printing.
 */
const BYTECODE_COMMENT_START_INDEX: usize = 46;

/**
 * The maximum length (in characters) of the index of a single bytecode instruction.
 */
const BYTECODE_INDEX_LENGTH: usize = 5;

fn print_class_file(cf: &ClassFile) {
    print_header(cf);
    print_constant_pool(&cf.constant_pool);
    println!("{{");
    print_fields(&cf.constant_pool, &cf.fields);
    print_methods(&cf.constant_pool, cf.this_class, &cf.methods);
    println!("}}");
    print_attributes(&cf.constant_pool, cf.this_class, &cf.attributes);
}

fn print_header(cf: &ClassFile) {
    println!("Classfile {}", cf.absolute_file_path);
    println!(
        "  Last modified {} {}, {}; size {} bytes",
        OffsetDateTime::from(cf.modified_time)
            .month()
            .to_string()
            .chars()
            .take(3)
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(""),
        OffsetDateTime::from(cf.modified_time).day(),
        OffsetDateTime::from(cf.modified_time).year(),
        cf.file_size
    );
    println!(
        "  SHA-256 checksum {}",
        cf.sha256_digest
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect::<Vec<String>>()
            .concat()
    );
    let source_file: String = cf
        .attributes
        .iter()
        .filter(|attr| matches!(attr, AttributeInfo::SourceFile { .. }))
        .map(|attr| match attr {
            AttributeInfo::SourceFile { source_file_index } => {
                cf.constant_pool.get_utf8_content(*source_file_index)
            }
            _ => unreachable!(),
        })
        .next()
        .unwrap();
    println!("  Compiled from \"{}\"", source_file);
    println!(
        "{} {}",
        access_flags::modifier_repr_vec(&cf.access_flags),
        cf.constant_pool
            .get_class_name(cf.this_class)
            .replace('/', ".")
    );
    println!("  minor version: {}", cf.minor_version);
    println!("  major version: {}", cf.major_version);
    println!(
        "  flags: (0x{:04x}) {}",
        access_flags::to_u16(&cf.access_flags),
        access_flags::java_repr_vec(&cf.access_flags)
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
        "  interfaces: {}, fields: {}, methods: {}, attributes: {}",
        cf.interfaces.len(),
        cf.fields.len(),
        cf.methods.len(),
        cf.attributes.len()
    );
}

fn print_constant_pool(cp: &ConstantPool) {
    println!("Constant pool:");
    for i in 0..cp.len() {
        if i > 1
            && (matches!(
                cp[(i - 1).try_into().unwrap()],
                ConstantPoolInfo::Long { .. }
            ) || matches!(
                cp[(i - 1).try_into().unwrap()],
                ConstantPoolInfo::Double { .. }
            ))
        {
            continue;
        }

        match &cp[i.try_into().unwrap()] {
            ConstantPoolInfo::Utf8 { bytes } => {
                let content = constant_pool::convert_utf8(bytes);
                if content.trim().is_empty() {
                    println!(
                        "{:>width$} = Utf8",
                        format!("#{}", i + 1),
                        width = CP_INDEX_WIDTH
                    );
                } else {
                    println!(
                        "{:<width$}{}",
                        format!(
                            "{:>width$} = Utf8",
                            format!("#{}", i + 1),
                            width = CP_INDEX_WIDTH
                        ),
                        content,
                        width = CP_INFO_START_INDEX
                    )
                }
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
            ConstantPoolInfo::String { string_index } => {
                print!(
                    "{:<width$}",
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
                    width = CP_COMMENT_START_INDEX
                );
                let string_content = cp.get_utf8_content(*string_index);
                if string_content.trim().is_empty() {
                    println!("//");
                } else {
                    println!("// {}", string_content);
                }
            }
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
                cp.get_wrapped_utf8_content(*name_index),
                width = CP_COMMENT_START_INDEX
            ),
            ConstantPoolInfo::FieldRef {
                class_index,
                name_and_type_index,
            } => println!(
                "{:<width$}// {}",
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
                cp.get_field_ref_string(*class_index, *name_and_type_index),
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
                cp.get_method_ref_string(*class_index, *name_and_type_index),
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
                cp.get_method_ref_string(*class_index, *name_and_type_index),
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
                cp.get_name_and_type_string(*name_index, *descriptor_index),
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
                cp.get_utf8_content(*descriptor_index),
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
                reference_kind::java_repr(*reference_kind),
                cp.get_method_ref(*reference_index),
                width = CP_COMMENT_START_INDEX
            ),
            ConstantPoolInfo::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => println!(
                "{:<width$}// {}",
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
                cp.get_invoke_dynamic_string(*bootstrap_method_attr_index, *name_and_type_index),
                width = CP_COMMENT_START_INDEX
            ),
            ConstantPoolInfo::Null {} => unreachable!(),
        }
    }
}

fn print_fields(cp: &ConstantPool, fields: &[FieldInfo]) {
    for field in fields.iter() {
        let descriptor: String = cp.get_utf8_content(field.descriptor_index);
        println!(
            "  {} {} {};",
            access_flags::modifier_repr_vec(&field.access_flags),
            classfile::convert_descriptor(&descriptor),
            cp.get_utf8_content(field.name_index)
        );
        println!("    descriptor: {}", descriptor);
        println!(
            "    flags: (0x{:04x}) {}",
            access_flags::to_u16(&field.access_flags),
            access_flags::java_repr_vec(&field.access_flags)
        );
        println!();
    }
}

fn print_methods(cp: &ConstantPool, this_class: u16, methods: &[MethodInfo]) {
    for (i, method) in methods.iter().enumerate() {
        let descriptor: String = cp.get_utf8_content(method.descriptor_index);
        let method_name = cp.get_utf8_content(method.name_index);
        if i > 0 {
            println!();
        }
        print!(
            "  {} ",
            access_flags::modifier_repr_vec(&method.access_flags)
        );
        if method_name == "<clinit>" {
            println!("{{}};");
        } else if method_name == "<init>" {
            println!("{}();", cp.get_class_name(this_class).replace("/", "."));
        } else {
            println!(
                "{} {}{};",
                classfile::get_return_type(&descriptor),
                method_name,
                classfile::convert_descriptor(&descriptor)
            );
        }
        println!("    descriptor: {}", descriptor);
        println!(
            "    flags: (0x{:04x}) {}",
            access_flags::to_u16(&method.access_flags),
            access_flags::java_repr_vec(&method.access_flags)
        );

        print_attributes(cp, this_class, &method.attributes);
    }
}

fn add_offset<T>(position: u32, offset: T) -> u32
where
    T: Into<i64>,
{
    let offset = offset.into();
    if offset >= 0 {
        position
            .checked_add(offset as u32)
            .expect("Negative final position")
    } else {
        position
            .checked_sub((-offset) as u32)
            .expect("Negative final position")
    }
}

fn get_opcode_and_arguments_string(position: &u32, instruction: &BytecodeInstruction) -> String {
    match instruction {
        BytecodeInstruction::Dup {} => "dup".to_owned(),
        BytecodeInstruction::AConstNull {} => "aconst_null".to_owned(),
        BytecodeInstruction::IConst { constant } => {
            if *constant == -1 {
                "iconst_m1".to_owned()
            } else if *constant <= 5 {
                "iconst_".to_owned() + &constant.to_string()
            } else {
                "iconst ".to_owned() + &constant.to_string()
            }
        }
        BytecodeInstruction::LConst { constant } => {
            if *constant <= 1 {
                "lconst_".to_owned() + &constant.to_string()
            } else {
                "lconst    ".to_owned() + &constant.to_string()
            }
        }
        BytecodeInstruction::Ldc {
            constant_pool_index,
        } => "ldc           #".to_owned() + &constant_pool_index.to_string(),
        BytecodeInstruction::LdcW {
            constant_pool_index,
        } => "ldc_w         #".to_owned() + &constant_pool_index.to_string(),
        BytecodeInstruction::Ldc2W {
            constant_pool_index,
        } => "ldc2_w        #".to_owned() + &constant_pool_index.to_string(),
        BytecodeInstruction::ALoad {
            local_variable_index,
        } => {
            if *local_variable_index <= 3 {
                "aload_".to_owned() + &local_variable_index.to_string()
            } else {
                "aload         ".to_owned() + &local_variable_index.to_string()
            }
        }
        BytecodeInstruction::AStore {
            local_variable_index,
        } => {
            if *local_variable_index <= 3 {
                "astore_".to_owned() + &local_variable_index.to_string()
            } else {
                "astore        ".to_owned() + &local_variable_index.to_string()
            }
        }
        BytecodeInstruction::ILoad {
            local_variable_index,
        } => {
            if *local_variable_index <= 3 {
                "iload_".to_owned() + &local_variable_index.to_string()
            } else {
                "iload         ".to_owned() + &local_variable_index.to_string()
            }
        }
        BytecodeInstruction::IStore {
            local_variable_index,
        } => {
            if *local_variable_index <= 3 {
                "istore_".to_owned() + &local_variable_index.to_string()
            } else {
                "istore        ".to_owned() + &local_variable_index.to_string()
            }
        }
        BytecodeInstruction::LLoad {
            local_variable_index,
        } => {
            if *local_variable_index <= 3 {
                "lload_".to_owned() + &local_variable_index.to_string()
            } else {
                "lload         ".to_owned() + &local_variable_index.to_string()
            }
        }
        BytecodeInstruction::LStore {
            local_variable_index,
        } => {
            if *local_variable_index <= 3 {
                "lstore_".to_owned() + &local_variable_index.to_string()
            } else {
                "lstore        ".to_owned() + &local_variable_index.to_string()
            }
        }
        BytecodeInstruction::AaLoad {} => "aaload".to_owned(),
        BytecodeInstruction::AaStore {} => "aastore".to_owned(),
        BytecodeInstruction::ANewArray {
            constant_pool_index,
        } => "anewarray     #".to_owned() + &constant_pool_index.to_string(),
        BytecodeInstruction::AThrow {} => "athrow".to_owned(),
        BytecodeInstruction::New {
            constant_pool_index,
        } => "new           #".to_owned() + &constant_pool_index.to_string(),
        BytecodeInstruction::BiPush { immediate } => {
            "bipush        ".to_owned() + &immediate.to_string()
        }
        BytecodeInstruction::Return {} => "return".to_owned(),
        BytecodeInstruction::LReturn {} => "lreturn".to_owned(),
        BytecodeInstruction::AReturn {} => "areturn".to_owned(),
        BytecodeInstruction::ArrayLength {} => "arraylength".to_owned(),
        BytecodeInstruction::GetStatic { field_ref_index } => {
            "getstatic     #".to_owned() + &field_ref_index.to_string()
        }
        BytecodeInstruction::PutStatic { field_ref_index } => {
            "putstatic     #".to_owned() + &field_ref_index.to_string()
        }
        BytecodeInstruction::CheckCast {
            constant_pool_index,
        } => "checkcast     #".to_owned() + &constant_pool_index.to_string(),

        // Invocation instructions
        BytecodeInstruction::InvokeSpecial { method_ref_index } => {
            "invokespecial #".to_owned() + &method_ref_index.to_string()
        }
        BytecodeInstruction::InvokeStatic { method_ref_index } => {
            "invokestatic  #".to_owned() + &method_ref_index.to_string()
        }
        BytecodeInstruction::InvokeVirtual { method_ref_index } => {
            "invokevirtual #".to_owned() + &method_ref_index.to_string()
        }
        BytecodeInstruction::InvokeDynamic {
            constant_pool_index,
        } => "invokedynamic #".to_owned() + &constant_pool_index.to_string() + ",  0",
        BytecodeInstruction::InvokeInterface {
            constant_pool_index,
            count,
        } => {
            "invokeinterface #".to_owned()
                + &constant_pool_index.to_string()
                + ",  "
                + &count.to_string()
        }

        // Conditional instructions
        BytecodeInstruction::IfIcmpEq { offset } => {
            "if_icmpeq     ".to_owned() + &add_offset(*position, *offset).to_string()
        }
        BytecodeInstruction::IfIcmpNe { offset } => {
            "if_icmpne     ".to_owned() + &add_offset(*position, *offset).to_string()
        }
        BytecodeInstruction::IfIcmpLt { offset } => {
            "if_icmplt     ".to_owned() + &add_offset(*position, *offset).to_string()
        }
        BytecodeInstruction::IfIcmpGe { offset } => {
            "if_icmpge     ".to_owned() + &add_offset(*position, *offset).to_string()
        }
        BytecodeInstruction::IfIcmpGt { offset } => {
            "if_icmpgt     ".to_owned() + &add_offset(*position, *offset).to_string()
        }
        BytecodeInstruction::IfIcmpLe { offset } => {
            "if_icmple     ".to_owned() + &add_offset(*position, *offset).to_string()
        }
        BytecodeInstruction::IfEq { offset } => {
            "ifeq          ".to_owned() + &add_offset(*position, *offset).to_string()
        }
        BytecodeInstruction::IfNe { offset } => {
            "ifne          ".to_owned() + &add_offset(*position, *offset).to_string()
        }
        BytecodeInstruction::IfLt { offset } => {
            "iflt          ".to_owned() + &add_offset(*position, *offset).to_string()
        }
        BytecodeInstruction::IfGe { offset } => {
            "ifge          ".to_owned() + &add_offset(*position, *offset).to_string()
        }
        BytecodeInstruction::IfGt { offset } => {
            "ifgt          ".to_owned() + &add_offset(*position, *offset).to_string()
        }
        BytecodeInstruction::IfLe { offset } => {
            "ifle          ".to_owned() + &add_offset(*position, *offset).to_string()
        }
        BytecodeInstruction::IfNonNull { offset } => {
            "ifnonnull     ".to_owned() + &add_offset(*position, *offset).to_string()
        }
        BytecodeInstruction::GoTo { offset } => {
            "goto          ".to_owned() + &add_offset(*position, *offset).to_string()
        }

        // Switches
        BytecodeInstruction::TableSwitch {
            default,
            low,
            offsets,
        } => {
            "tableswitch   { // ".to_owned()
                + &low.to_string()
                + " to "
                + &(low + (offsets.len() as i32) - 1).to_string()
                + "\n"
                + &offsets
                    .iter()
                    .enumerate()
                    .map(|(i, offset)| {
                        format!("             {:>11}: {}", i, add_offset(*position, *offset))
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
                + "\n                 default: "
                + &add_offset(*position, *default).to_string()
                + "\n            }"
        }
        BytecodeInstruction::LookupSwitch { default, pairs } => {
            "lookupswitch  { // ".to_owned()
                + &pairs.len().to_string()
                + "\n"
                + &pairs
                    .iter()
                    .map(|p| {
                        format!(
                            "             {:>11}: {}",
                            p.match_value,
                            add_offset(*position, p.offset)
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
                + "\n                 default: "
                + &add_offset(*position, *default).to_string()
                + "\n            }"
        }

        // Arithmetic instructions
        BytecodeInstruction::IInc { index, constant } => {
            "iinc          ".to_owned() + &index.to_string() + ", " + &constant.to_string()
        }
        BytecodeInstruction::LDiv {} => "ldiv".to_owned(),
        BytecodeInstruction::IAdd {} => "iadd".to_owned(),
        BytecodeInstruction::ISub {} => "isub".to_owned(),
        BytecodeInstruction::I2L {} => "i2l".to_owned(),
        BytecodeInstruction::LAdd {} => "ladd".to_owned(),
        BytecodeInstruction::LMul {} => "lmul".to_owned(),
    }
}

fn get_constant_string(cp: &ConstantPool, constant_pool_index: u16) -> String {
    match cp[constant_pool_index - 1] {
        ConstantPoolInfo::String { string_index } => {
            let string_content = cp.get_utf8_content(string_index);
            if string_content.trim().is_empty() {
                "String".to_owned()
            } else {
                "String ".to_owned() + &cp.get_utf8_content(string_index)
            }
        }
        ConstantPoolInfo::Long {
            high_bytes,
            low_bytes,
        } => {
            "long ".to_owned()
                + &(((high_bytes as u64) << 32) | (low_bytes as u64)).to_string()
                + "l"
        }
        _ => unreachable!(),
    }
}

fn get_method_type(cpe: &ConstantPoolInfo) -> String {
    match cpe {
        ConstantPoolInfo::MethodRef {
            class_index: _,
            name_and_type_index: _,
        } => "Method",
        ConstantPoolInfo::InterfaceMethodRef {
            class_index: _,
            name_and_type_index: _,
        } => "InterfaceMethod",
        _ => unreachable!(),
    }
    .to_owned()
}

fn get_comment(
    cp: &ConstantPool,
    this_class: u16,
    instruction: &BytecodeInstruction,
) -> Option<String> {
    match instruction {
        BytecodeInstruction::Dup {} => None,
        BytecodeInstruction::AConstNull {} => None,
        BytecodeInstruction::IConst { constant: _ } => None,
        BytecodeInstruction::LConst { constant: _ } => None,
        BytecodeInstruction::Ldc {
            constant_pool_index,
        } => Some(get_constant_string(cp, (*constant_pool_index).into())),
        BytecodeInstruction::LdcW {
            constant_pool_index,
        } => Some(get_constant_string(cp, *constant_pool_index)),
        BytecodeInstruction::Ldc2W {
            constant_pool_index,
        } => Some(get_constant_string(cp, *constant_pool_index)),
        BytecodeInstruction::ALoad {
            local_variable_index: _,
        } => None,
        BytecodeInstruction::AStore {
            local_variable_index: _,
        } => None,
        BytecodeInstruction::ILoad {
            local_variable_index: _,
        } => None,
        BytecodeInstruction::IStore {
            local_variable_index: _,
        } => None,
        BytecodeInstruction::LLoad {
            local_variable_index: _,
        } => None,
        BytecodeInstruction::LStore {
            local_variable_index: _,
        } => None,
        BytecodeInstruction::AaLoad {} => None,
        BytecodeInstruction::AaStore {} => None,
        BytecodeInstruction::ANewArray {
            constant_pool_index,
        } => Some("class ".to_owned() + &cp.get_class_name(*constant_pool_index)),
        BytecodeInstruction::AThrow {} => None,
        BytecodeInstruction::New {
            constant_pool_index,
        } => Some("class ".to_owned() + &cp.get_class_name(*constant_pool_index)),
        BytecodeInstruction::BiPush { immediate: _ } => None,
        BytecodeInstruction::Return {} => None,
        BytecodeInstruction::LReturn {} => None,
        BytecodeInstruction::AReturn {} => None,
        BytecodeInstruction::GetStatic { field_ref_index } => Some(
            "Field ".to_owned()
                + &match cp[field_ref_index - 1] {
                    ConstantPoolInfo::FieldRef {
                        class_index,
                        name_and_type_index,
                    } => {
                        if class_index == this_class {
                            cp.get_name_and_type(name_and_type_index)
                        } else {
                            cp.get_field_ref(*field_ref_index)
                        }
                    }
                    _ => unreachable!(),
                },
        ),
        BytecodeInstruction::PutStatic { field_ref_index } => Some(
            "Field ".to_owned()
                + &match cp[field_ref_index - 1] {
                    ConstantPoolInfo::FieldRef {
                        class_index,
                        name_and_type_index,
                    } => {
                        if class_index == this_class {
                            cp.get_name_and_type(name_and_type_index)
                        } else {
                            cp.get_field_ref(*field_ref_index)
                        }
                    }
                    _ => unreachable!(),
                },
        ),
        BytecodeInstruction::InvokeSpecial { method_ref_index } => {
            Some("Method ".to_owned() + &cp.get_method_ref(*method_ref_index))
        }
        BytecodeInstruction::InvokeStatic { method_ref_index } => {
            let method_entry = &cp[method_ref_index - 1];
            Some(
                get_method_type(method_entry)
                    + " "
                    + &match method_entry {
                        ConstantPoolInfo::MethodRef {
                            class_index,
                            name_and_type_index,
                        } => {
                            if *class_index == this_class {
                                cp.get_name_and_type(*name_and_type_index)
                            } else {
                                cp.get_method_ref(*method_ref_index)
                            }
                        }
                        ConstantPoolInfo::InterfaceMethodRef {
                            class_index,
                            name_and_type_index,
                        } => {
                            if *class_index == this_class {
                                cp.get_name_and_type(*name_and_type_index)
                            } else {
                                cp.get_method_ref(*method_ref_index)
                            }
                        }
                        _ => unreachable!(),
                    },
            )
        }
        BytecodeInstruction::InvokeVirtual { method_ref_index } => {
            Some("Method ".to_owned() + &cp.get_method_ref(*method_ref_index))
        }
        BytecodeInstruction::InvokeDynamic {
            constant_pool_index,
        } => Some("InvokeDynamic ".to_owned() + &cp.get_invoke_dynamic(*constant_pool_index)),
        BytecodeInstruction::InvokeInterface {
            constant_pool_index,
            count: _,
        } => Some(
            get_method_type(&cp[constant_pool_index - 1])
                + " "
                + &cp.get_method_ref(*constant_pool_index),
        ),
        BytecodeInstruction::ArrayLength {} => None,
        BytecodeInstruction::IfIcmpEq { offset: _ } => None,
        BytecodeInstruction::IfIcmpNe { offset: _ } => None,
        BytecodeInstruction::IfIcmpLt { offset: _ } => None,
        BytecodeInstruction::IfIcmpGe { offset: _ } => None,
        BytecodeInstruction::IfIcmpGt { offset: _ } => None,
        BytecodeInstruction::IfIcmpLe { offset: _ } => None,
        BytecodeInstruction::IfEq { offset: _ } => None,
        BytecodeInstruction::IfNe { offset: _ } => None,
        BytecodeInstruction::IfLt { offset: _ } => None,
        BytecodeInstruction::IfGe { offset: _ } => None,
        BytecodeInstruction::IfGt { offset: _ } => None,
        BytecodeInstruction::IfLe { offset: _ } => None,
        BytecodeInstruction::IfNonNull { offset: _ } => None,
        BytecodeInstruction::GoTo { offset: _ } => None,
        BytecodeInstruction::TableSwitch {
            default: _,
            low: _,
            offsets: _,
        } => None,
        BytecodeInstruction::LookupSwitch {
            default: _,
            pairs: _,
        } => None,
        BytecodeInstruction::CheckCast {
            constant_pool_index,
        } => Some("class ".to_owned() + &cp.get_class_name(*constant_pool_index)),
        BytecodeInstruction::IInc {
            index: _,
            constant: _,
        } => None,
        BytecodeInstruction::LDiv {} => None,
        BytecodeInstruction::IAdd {} => None,
        BytecodeInstruction::ISub {} => None,
        BytecodeInstruction::I2L {} => None,
        BytecodeInstruction::LAdd {} => None,
        BytecodeInstruction::LMul {} => None,
    }
}

fn get_verification_type_info_string(cp: &ConstantPool, vti: &VerificationTypeInfo) -> String {
    match vti {
        VerificationTypeInfo::TopVariable => todo!(),
        VerificationTypeInfo::IntegerVariable => "int".to_owned(),
        VerificationTypeInfo::FloatVariable => "float".to_owned(),
        VerificationTypeInfo::LongVariable => "long".to_owned(),
        VerificationTypeInfo::DoubleVariable => "double".to_owned(),
        VerificationTypeInfo::NullVariable => "null".to_owned(),
        VerificationTypeInfo::UninitializedThisVariable => todo!(),
        VerificationTypeInfo::ObjectVariable {
            constant_pool_index,
        } => "class ".to_owned() + &cp.get_class_name(*constant_pool_index),
        VerificationTypeInfo::UninitializedVariable { offset: _ } => todo!(),
    }
}

fn print_attributes(cp: &ConstantPool, this_class: u16, attributes: &[AttributeInfo]) {
    for attribute in attributes.iter() {
        match attribute {
            AttributeInfo::Code {
                max_stack,
                max_locals,
                code,
                exception_table,
                attributes,
            } => {
                println!("    Code:");
                println!(
                    "      stack={}, locals={}, args_size={}",
                    max_stack, max_locals, 0
                );
                for (position, instruction) in code.iter() {
                    let opcode_and_arguments: String =
                        get_opcode_and_arguments_string(position, instruction);
                    let comment: Option<String> = get_comment(cp, this_class, instruction);
                    match comment {
                        Some(content) => {
                            println!(
                                "{:<width$}// {}",
                                format!(
                                    "     {:>width$}: {}",
                                    position,
                                    opcode_and_arguments,
                                    width = BYTECODE_INDEX_LENGTH
                                ),
                                content,
                                width = BYTECODE_COMMENT_START_INDEX
                            )
                        }
                        None => println!(
                            "     {:>width$}: {}",
                            position,
                            opcode_and_arguments,
                            width = BYTECODE_INDEX_LENGTH
                        ),
                    }
                }
                if !exception_table.is_empty() {
                    println!("      Exception table:");
                    println!("         from    to  target type");
                    for exception in exception_table.iter() {
                        println!(
                            "          {}  {}  {}   Class {}",
                            exception.start_pc,
                            exception.end_pc,
                            exception.handler_pc,
                            cp.get_class_name(exception.catch_type)
                        );
                    }
                }
                print_attributes(cp, this_class, attributes);
            }
            AttributeInfo::LineNumberTable { line_number_table } => {
                println!("      LineNumberTable:");
                for entry in line_number_table.iter() {
                    println!("        line {}: {}", entry.line_number, entry.start_pc);
                }
            }
            AttributeInfo::LocalVariableTable {
                local_variable_table,
            } => {
                println!("      LocalVariableTable:");
                println!("        Start  Length  Slot  Name   Signature");
                for entry in local_variable_table.iter() {
                    println!(
                        "         {:4}    {:4}    {:2} {:>5}   {}",
                        entry.start_pc,
                        entry.length,
                        entry.index,
                        cp.get_utf8_content(entry.name_index),
                        cp.get_utf8_content(entry.descriptor_index)
                    );
                }
            }
            AttributeInfo::StackMapTable { stack_map_table } => {
                println!(
                    "      StackMapTable: number_of_entries = {}",
                    stack_map_table.len()
                );
                for frame in stack_map_table.iter() {
                    match frame {
                        StackMapFrame::SameFrame { frame_type } => {
                            println!("        frame_type = {} /* same */", frame_type)
                        }
                        StackMapFrame::SameLocals1StackItemFrame { frame_type, stack } => {
                            println!(
                                "        frame_type = {} /* same_locals_1_stack_item */",
                                frame_type
                            );
                            println!(
                                "          stack = [ {} ]",
                                get_verification_type_info_string(cp, stack)
                            );
                        }
                        StackMapFrame::SameLocals1StackItemFrameExtended {
                            offset_delta,
                            stack,
                        } => {
                            println!(
                                "        frame_type = 247 /* same_locals_1_stack_item_frame_extended */"
                            );
                            println!("          offset_delta = {}", offset_delta);
                            println!(
                                "          stack = [ {} ]",
                                get_verification_type_info_string(cp, stack)
                            );
                        }
                        StackMapFrame::ChopFrame {
                            frame_type,
                            offset_delta,
                        } => {
                            println!("        frame_type = {} /* chop */", frame_type);
                            println!("          offset_delta = {}", offset_delta);
                        }
                        StackMapFrame::SameFrameExtended { offset_delta } => {
                            println!("        frame_type = 251 /* same_frame_extended */");
                            println!("          offset_delta = {}", offset_delta);
                        }
                        StackMapFrame::AppendFrame {
                            frame_type,
                            offset_delta,
                            locals,
                        } => {
                            println!("        frame_type = {} /* append */", frame_type);
                            println!("          offset_delta = {}", offset_delta);
                            println!(
                                "          locals = {}",
                                if locals.is_empty() {
                                    "[]".to_owned()
                                } else {
                                    "[ ".to_owned()
                                        + &locals
                                            .iter()
                                            .map(|x| get_verification_type_info_string(cp, x))
                                            .collect::<Vec<String>>()
                                            .join(", ")
                                        + " ]"
                                }
                            );
                        }
                        StackMapFrame::FullFrame {
                            offset_delta,
                            locals,
                            stack,
                        } => {
                            println!("        frame_type = 255 /* full_frame */");
                            println!("          offset_delta = {}", offset_delta);
                            println!(
                                "          locals = {}",
                                if locals.is_empty() {
                                    "[]".to_owned()
                                } else {
                                    "[ ".to_owned()
                                        + &locals
                                            .iter()
                                            .map(|x| get_verification_type_info_string(cp, x))
                                            .collect::<Vec<String>>()
                                            .join(", ")
                                        + " ]"
                                }
                            );
                            println!(
                                "          stack = {}",
                                if stack.is_empty() {
                                    "[]".to_owned()
                                } else {
                                    "[ ".to_owned()
                                        + &stack
                                            .iter()
                                            .map(|x| get_verification_type_info_string(cp, x))
                                            .collect::<Vec<String>>()
                                            .join(", ")
                                        + " ]"
                                }
                            );
                        }
                    }
                }
            }
            AttributeInfo::SourceFile { source_file_index } => {
                println!(
                    "SourceFile: \"{}\"",
                    cp.get_utf8_content(*source_file_index)
                )
            }
            AttributeInfo::BootstrapMethods { methods } => {
                println!("BootstrapMethods:");
                for (i, method) in methods.iter().enumerate() {
                    print!("  {}: #{} ", i, method.bootstrap_method_ref);

                    // TODO: can we merge this match-case with the one below?
                    match cp[method.bootstrap_method_ref - 1] {
                        ConstantPoolInfo::MethodHandle {
                            reference_kind,
                            reference_index,
                        } => println!(
                            "{} {}",
                            reference_kind::java_repr(reference_kind),
                            cp.get_method_ref(reference_index)
                        ),
                        _ => unreachable!(),
                    }
                    println!("    Method arguments:");
                    for arg in method.bootstrap_arguments.iter() {
                        print!("      #{} ", arg);
                        match cp[arg - 1] {
                            ConstantPoolInfo::String { string_index } => {
                                println!("{}", cp.get_utf8_content(string_index))
                            }
                            ConstantPoolInfo::MethodType { descriptor_index } => {
                                println!("{}", cp.get_utf8_content(descriptor_index))
                            }
                            ConstantPoolInfo::MethodHandle {
                                reference_kind,
                                reference_index,
                            } => println!(
                                "{} {}",
                                reference_kind::java_repr(reference_kind),
                                cp.get_method_ref(reference_index)
                            ),
                            _ => unreachable!(),
                        }
                    }
                }
            }
            AttributeInfo::InnerClasses { classes } => {
                println!("InnerClasses:");
                for class in classes.iter() {
                    println!(
                        "{:<width$}// {}=class {} of class {}",
                        format!(
                            "  {} #{}= #{} of #{};",
                            access_flags::modifier_repr_vec(&class.inner_class_access_flags),
                            class.inner_name_index,
                            class.inner_class_info_index,
                            class.outer_class_info_index
                        ),
                        cp.get_utf8_content(class.inner_name_index),
                        cp.get_class_name(class.inner_class_info_index),
                        cp.get_class_name(class.outer_class_info_index),
                        width = CP_COMMENT_START_INDEX
                    );
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let filename = env::args().nth(1).expect("Usage: program <filename>");

    let classfile: ClassFile = parse_class_file(filename);

    print_class_file(&classfile);

    Ok(())
}
