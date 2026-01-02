#![forbid(unsafe_code)]

use std::env;
use std::io::Result;

use classfile::access_flags::MethodAccessFlag;
use classfile::attributes::{AttributeInfo, StackMapFrame, VerificationTypeInfo};
use classfile::bytecode::BytecodeInstruction;
use classfile::constant_pool::{self, ConstantPool, ConstantPoolInfo};
use classfile::descriptor::MethodDescriptor;
use classfile::fields::FieldInfo;
use classfile::methods::MethodInfo;
use classfile::{ClassFile, access_flags, descriptor, parse_class_file, reference_kind};
use time::OffsetDateTime;

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
    print_methods(
        &cf.constant_pool,
        cf.this_class,
        cf.access_flags
            .contains(&access_flags::ClassAccessFlag::Enum),
        &cf.methods,
    );
    println!("}}");
    print_class_attributes(&cf.constant_pool, &cf.attributes);
}

/**
 * Returns the index of the column (on the terminal) where the index of each constant pool entry ends.
 */
fn get_constant_pool_index_width(cp: &ConstantPool) -> usize {
    3 + num_digits(cp.len())
}

/**
 * Returns the index of the column (on the terminal) where the information of each entry is displayed.
 */
fn get_constant_pool_info_start_index(cp: &ConstantPool) -> usize {
    25 + num_digits(cp.len())
}

/**
 * Returns the index of the column (on the terminal) where the comments (the '//') start for the constant pool.
 */
fn get_constant_pool_comment_start_index(cp: &ConstantPool) -> usize {
    39 + num_digits(cp.len())
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

    let this_class_name = cf
        .constant_pool
        .get_class_name(cf.this_class)
        .replace('/', ".");
    print!(
        "{} {}",
        access_flags::modifier_repr_vec(&cf.access_flags),
        this_class_name
    );

    let is_enum: bool = cf
        .access_flags
        .contains(&access_flags::ClassAccessFlag::Enum);
    let super_class_name = cf.constant_pool.get_class_name(cf.super_class);
    if is_enum {
        println!(" extends java.lang.Enum<{}>", this_class_name);
    } else if super_class_name != "java/lang/Object" {
        println!(" extends {}", super_class_name.replace('/', "."));
    } else {
        println!();
    }

    println!("  minor version: {}", cf.minor_version);
    println!("  major version: {}", cf.major_version);
    println!(
        "  flags: (0x{:04x}) {}",
        access_flags::to_u16(&cf.access_flags),
        access_flags::java_repr_vec(&cf.access_flags)
    );

    let comment_index: usize = get_constant_pool_comment_start_index(&cf.constant_pool);
    println!(
        "{:<comment_index$}// {}",
        format!("  this_class: #{}", cf.this_class),
        cf.constant_pool.get_class_name(cf.this_class),
    );
    println!(
        "{:<comment_index$}// {}",
        format!("  super_class: #{}", cf.super_class),
        cf.constant_pool.get_class_name(cf.super_class),
    );
    println!(
        "  interfaces: {}, fields: {}, methods: {}, attributes: {}",
        cf.interfaces.len(),
        cf.fields.len(),
        cf.methods.len(),
        cf.attributes.len()
    );
}

fn num_digits(n: usize) -> usize {
    (n as f64).log10().floor() as usize + 1
}

fn print_constant_pool(cp: &ConstantPool) {
    let index_width: usize = get_constant_pool_index_width(cp);
    let info_start_index: usize = get_constant_pool_info_start_index(cp);
    let comment_index: usize = get_constant_pool_comment_start_index(cp);

    println!("Constant pool:");
    for i in 0..cp.len() {
        /*
            We skip entries right after Long and Double. Why?
            > In retrospect, making 8-byte constants take two constant pool entries was a poor choice.
            Source: <https://docs.oracle.com/javase/specs/jvms/se25/html/jvms-4.html#jvms-4.4.5>
        */
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
                    println!("{:>index_width$} = Utf8", format!("#{}", i + 1),);
                } else {
                    println!(
                        "{:<info_start_index$}{}",
                        format!("{:>index_width$} = Utf8", format!("#{}", i + 1),),
                        content,
                    )
                }
            }
            ConstantPoolInfo::Long {
                high_bytes,
                low_bytes,
            } => println!(
                "{:<info_start_index$}{}l",
                format!("{:>index_width$} = Long", format!("#{}", i + 1),),
                ((*high_bytes as u64) << 32) | (*low_bytes as u64),
            ),
            ConstantPoolInfo::Double {
                high_bytes: _,
                low_bytes: _,
            } => print!("Double"),
            ConstantPoolInfo::String { string_index } => {
                print!(
                    "{:<comment_index$}",
                    format!(
                        "{:<info_start_index$}#{}",
                        format!("{:>index_width$} = String", format!("#{}", i + 1),),
                        string_index,
                    ),
                );
                let string_content = cp.get_utf8_content(*string_index);
                if string_content.trim().is_empty() {
                    println!("//");
                } else {
                    println!("// {}", string_content);
                }
            }
            ConstantPoolInfo::Class { name_index } => println!(
                "{:<comment_index$}// {}",
                format!(
                    "{:<info_start_index$}#{}",
                    format!("{:>index_width$} = Class", format!("#{}", i + 1),),
                    name_index,
                ),
                cp.get_wrapped_utf8_content(*name_index),
            ),
            ConstantPoolInfo::FieldRef {
                class_index,
                name_and_type_index,
            } => println!(
                "{:<comment_index$}// {}",
                format!(
                    "{:<info_start_index$}#{}.#{}",
                    format!("{:>index_width$} = Fieldref", format!("#{}", i + 1),),
                    class_index,
                    name_and_type_index,
                ),
                cp.get_field_ref_string(*class_index, *name_and_type_index),
            ),
            ConstantPoolInfo::MethodRef {
                class_index,
                name_and_type_index,
            } => println!(
                "{:<comment_index$}// {}",
                format!(
                    "{:<info_start_index$}#{}.#{}",
                    format!("{:>index_width$} = Methodref", format!("#{}", i + 1),),
                    class_index,
                    name_and_type_index,
                ),
                cp.get_method_ref_string(*class_index, *name_and_type_index),
            ),
            ConstantPoolInfo::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => println!(
                "{:<comment_index$}// {}",
                format!(
                    "{:<info_start_index$}#{}.#{}",
                    format!(
                        "{:>index_width$} = InterfaceMethodref",
                        format!("#{}", i + 1),
                    ),
                    class_index,
                    name_and_type_index,
                ),
                cp.get_method_ref_string(*class_index, *name_and_type_index),
            ),
            ConstantPoolInfo::NameAndType {
                name_index,
                descriptor_index,
            } => println!(
                "{:<comment_index$}// {}",
                format!(
                    "{:<info_start_index$}#{}:#{}",
                    format!("{:>index_width$} = NameAndType", format!("#{}", i + 1),),
                    name_index,
                    descriptor_index,
                ),
                cp.get_name_and_type_string(*name_index, *descriptor_index),
            ),
            ConstantPoolInfo::MethodType { descriptor_index } => println!(
                "{:<comment_index$}//  {}",
                format!(
                    "{:<info_start_index$}#{}",
                    format!("{:>index_width$} = MethodType", format!("#{}", i + 1),),
                    descriptor_index,
                ),
                cp.get_utf8_content(*descriptor_index),
            ),
            ConstantPoolInfo::MethodHandle {
                reference_kind,
                reference_index,
            } => println!(
                "{:<comment_index$}// {} {}",
                format!(
                    "{:<info_start_index$}{}:#{}",
                    format!("{:>index_width$} = MethodHandle", format!("#{}", i + 1),),
                    *reference_kind as u8,
                    reference_index,
                ),
                reference_kind::java_repr(*reference_kind),
                cp.get_method_ref(*reference_index),
            ),
            ConstantPoolInfo::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => println!(
                "{:<comment_index$}// {}",
                format!(
                    "{:<info_start_index$}#{}:#{}",
                    format!("{:>index_width$} = InvokeDynamic", format!("#{}", i + 1),),
                    bootstrap_method_attr_index,
                    name_and_type_index,
                ),
                cp.get_invoke_dynamic_string(*bootstrap_method_attr_index, *name_and_type_index),
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
            descriptor::parse_field_descriptor(&descriptor),
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

fn print_methods(cp: &ConstantPool, this_class: u16, is_enum: bool, methods: &[MethodInfo]) {
    for (i, method) in methods.iter().enumerate() {
        let method_name: String = cp.get_utf8_content(method.name_index);
        let raw_descriptor: String = cp.get_utf8_content(method.descriptor_index);
        let parsed_descriptor: MethodDescriptor =
            descriptor::parse_method_descriptor(&raw_descriptor);
        if i > 0 {
            println!();
        }
        print!(
            "  {} ",
            access_flags::modifier_repr_vec(&method.access_flags)
        );
        if method_name == "<clinit>" {
            // this is the 'static {}' block of the class
            println!("{{}};");
        } else if method_name == "<init>" {
            // this is a constructor of the class

            let param_types = if is_enum {
                parsed_descriptor
                    .parameter_types
                    .iter()
                    .skip(2) // if this is an enum's constructor, we omit the first two parameter which are always the name and the ordinal, implicitly added by the compiler
                    .map(|t| format!("{}", t))
                    .collect::<Vec<String>>()
                    .join(", ")
            } else {
                parsed_descriptor
                    .parameter_types
                    .iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<String>>()
                    .join(", ")
            };

            println!(
                "{}({});",
                cp.get_class_name(this_class).replace("/", "."),
                param_types
            );
        } else {
            println!(
                "{} {}({});",
                parsed_descriptor.return_type,
                method_name,
                parsed_descriptor
                    .parameter_types
                    .iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<String>>()
                    .join(", ")
            );
        }
        println!("    descriptor: {}", raw_descriptor);
        println!(
            "    flags: (0x{:04x}) {}",
            access_flags::to_u16(&method.access_flags),
            access_flags::java_repr_vec(&method.access_flags)
        );

        print_method_attributes(cp, this_class, method);
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
        BytecodeInstruction::IReturn {} => "ireturn".to_owned(),
        BytecodeInstruction::LReturn {} => "lreturn".to_owned(),
        BytecodeInstruction::AReturn {} => "areturn".to_owned(),
        BytecodeInstruction::ArrayLength {} => "arraylength".to_owned(),
        BytecodeInstruction::LCmp {} => "lcmp".to_owned(),
        BytecodeInstruction::GetStatic { field_ref_index } => {
            "getstatic     #".to_owned() + &field_ref_index.to_string()
        }
        BytecodeInstruction::PutStatic { field_ref_index } => {
            "putstatic     #".to_owned() + &field_ref_index.to_string()
        }
        BytecodeInstruction::GetField { field_ref_index } => {
            "getfield      #".to_owned() + &field_ref_index.to_string()
        }
        BytecodeInstruction::PutField { field_ref_index } => {
            "putfield      #".to_owned() + &field_ref_index.to_string()
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
        BytecodeInstruction::LSub {} => "lsub".to_owned(),
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
        ConstantPoolInfo::Double {
            high_bytes,
            low_bytes,
        } => {
            "double ".to_owned()
                + &(((high_bytes as u64) << 32) | (low_bytes as u64)).to_string()
                + "l"
        }
        ConstantPoolInfo::Class { name_index } => {
            "class ".to_owned() + &cp.get_utf8_content(name_index)
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
        BytecodeInstruction::IReturn {} => None,
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
        BytecodeInstruction::GetField { field_ref_index } => Some(
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
        BytecodeInstruction::PutField { field_ref_index } => Some(
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
        BytecodeInstruction::InvokeSpecial { method_ref_index } => Some(
            "Method ".to_owned()
                + &match cp[method_ref_index - 1] {
                    ConstantPoolInfo::MethodRef {
                        class_index,
                        name_and_type_index,
                    } => {
                        if class_index == this_class {
                            cp.get_name_and_type(name_and_type_index)
                        } else {
                            cp.get_method_ref(*method_ref_index)
                        }
                    }
                    _ => unreachable!(),
                },
        ),
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
        BytecodeInstruction::LCmp {} => None,
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
        BytecodeInstruction::LSub {} => None,
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

fn get_number_of_arguments(cp: &ConstantPool, method: &MethodInfo) -> u8 {
    let mut num_arguments: u8 =
        descriptor::parse_method_descriptor(&cp.get_utf8_content(method.descriptor_index))
            .parameter_types
            .len()
            .try_into()
            .unwrap();

    if !method.access_flags.contains(&MethodAccessFlag::Static) {
        // if the method is not static, there is the implicit 'this' argument
        num_arguments += 1;
    }

    num_arguments
}

fn print_method_attributes(cp: &ConstantPool, this_class: u16, method: &MethodInfo) {
    let comment_index: usize = get_constant_pool_comment_start_index(cp);
    for attribute in method.attributes.iter() {
        match attribute {
            AttributeInfo::Code {
                max_stack,
                max_locals,
                code,
                exception_table,
                attributes,
            } => {
                println!("    Code:");
                let args_size = get_number_of_arguments(cp, method);
                println!(
                    "      stack={}, locals={}, args_size={}",
                    max_stack, max_locals, args_size
                );
                for (position, instruction) in code.iter() {
                    let opcode_and_arguments: String =
                        get_opcode_and_arguments_string(position, instruction);
                    let comment: Option<String> = get_comment(cp, this_class, instruction);
                    match comment {
                        Some(content) => {
                            println!(
                                "{:<BYTECODE_COMMENT_START_INDEX$}// {}",
                                format!(
                                    "     {:>BYTECODE_INDEX_LENGTH$}: {}",
                                    position, opcode_and_arguments,
                                ),
                                content,
                            )
                        }
                        None => println!(
                            "     {:>BYTECODE_INDEX_LENGTH$}: {}",
                            position, opcode_and_arguments,
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
                print_code_attributes(cp, attributes);
            }
            AttributeInfo::MethodParameters { parameters } => {
                println!("    MethodParameters:");
                println!("      Name                           Flags");
                for param in parameters.iter() {
                    let name = if param.name_index == 0 {
                        "<no name>"
                    } else {
                        &cp.get_utf8_content(param.name_index)
                    };
                    print!("      {}", name);
                    if !param.access_flags.is_empty() {
                        print!(
                            "                      {}",
                            access_flags::modifier_repr_vec(&param.access_flags)
                        );
                    }
                    println!();
                }
            }
            AttributeInfo::Signature { signature_index } => {
                println!(
                    "{:<width$}// {}",
                    format!("    Signature: #{}", signature_index),
                    cp.get_utf8_content(*signature_index),
                    width = comment_index + 2 // why?
                );
            }
            _ => unreachable!(),
        }
    }
}

fn print_code_attributes(cp: &ConstantPool, attributes: &[AttributeInfo]) {
    for attribute in attributes.iter() {
        match attribute {
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
            _ => unreachable!(),
        }
    }
}

fn print_class_attributes(cp: &ConstantPool, attributes: &[AttributeInfo]) {
    let comment_index: usize = get_constant_pool_comment_start_index(cp);
    for attribute in attributes.iter() {
        match attribute {
            AttributeInfo::SourceFile { source_file_index } => {
                println!(
                    "SourceFile: \"{}\"",
                    cp.get_utf8_content(*source_file_index)
                )
            }
            AttributeInfo::InnerClasses { classes } => {
                println!("InnerClasses:");
                for class in classes.iter() {
                    println!(
                        "{:<comment_index$}// {}=class {} of class {}",
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
                    );
                }
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
                            ConstantPoolInfo::Class { name_index } => {
                                println!("{}", cp.get_utf8_content(name_index));
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
            AttributeInfo::Record { components } => {
                println!("Record:");
                for component in components.iter() {
                    let descriptor = cp.get_utf8_content(component.descriptor_index);
                    println!(
                        "  {} {};",
                        descriptor::parse_field_descriptor(&descriptor),
                        cp.get_utf8_content(component.name_index)
                    );
                    println!("    descriptor: {}", descriptor);
                    println!();
                }
            }
            AttributeInfo::Signature { signature_index } => {
                println!(
                    "{:<width$}// {}",
                    format!("Signature: #{}", signature_index),
                    cp.get_utf8_content(*signature_index),
                    width = 40 // why is this 40 used only here?
                );
            }
            _ => unreachable!(),
        }
    }
}

fn main() -> Result<()> {
    let filename = env::args().nth(1).expect("Usage: program <filename>");

    let classfile: ClassFile = parse_class_file(filename);

    print_class_file(&classfile);

    Ok(())
}
