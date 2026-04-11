#![forbid(unsafe_code)]

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::string;
use std::time::SystemTime;

use binary_reader::{BinaryReader, Endianness};
use classfile::access_flags::MethodAccessFlag;
use classfile::attributes::{
    AttributeInfo, AttributeKind, StackMapFrame, VerificationTypeInfo, find_attribute,
};
use classfile::bytecode::BytecodeInstruction;
use classfile::classfile::{ClassFile, parse_class_file};
use classfile::constant_pool::{self, ConstantPool, ConstantPoolInfo};
use classfile::descriptor::MethodDescriptor;
use classfile::fields::FieldInfo;
use classfile::methods::MethodInfo;
use classfile::utils::absolute_no_symlinks;
use classfile::{access_flags, descriptor, reference_kind};
use time::OffsetDateTime;

use crate::line_writer::LineWriter;

/**
 * The index of the column (on the terminal) where the comments (the '//') start for the bytecode printing.
 */
const BYTECODE_COMMENT_START_INDEX: usize = 46;

/**
 * The maximum length (in characters) of the index of a single bytecode instruction.
 */
const BYTECODE_INDEX_LENGTH: usize = 5;

pub(crate) fn print_class_file(filename: String) {
    let mut lw: LineWriter = LineWriter::new();

    let abs_file_path: PathBuf = absolute_no_symlinks(Path::new(&filename)).unwrap();
    let absolute_file_path: String = abs_file_path.to_str().unwrap().to_owned();
    lw.println(&("Classfile ".to_owned() + &absolute_file_path.to_string()));

    let file: File = File::open(&abs_file_path).expect("File does not exist");
    let modified_time: SystemTime = file.metadata().unwrap().modified().unwrap();

    let mut file_reader: BufReader<File> = BufReader::new(file);
    let mut file_bytes: Vec<u8> = Vec::with_capacity(file_reader.capacity());
    file_reader
        .read_to_end(&mut file_bytes)
        .expect("Could not read whole file");
    let file_size: usize = file_bytes.len();

    let digest = sha::sha256(&file_bytes);

    lw.print("  Last modified ")
        .print(
            &OffsetDateTime::from(modified_time)
                .month()
                .to_string()
                .chars()
                .take(3)
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(""),
        )
        .print(" ")
        .print(&OffsetDateTime::from(modified_time).day().to_string())
        .print(", ")
        .print(&OffsetDateTime::from(modified_time).year().to_string())
        .print("; size ")
        .print(&file_size.to_string())
        .println(" bytes");

    lw.print("  SHA-256 checksum ").println(
        &digest
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect::<Vec<String>>()
            .concat(),
    );

    let abs_file_path: PathBuf = absolute_no_symlinks(Path::new(&filename)).unwrap();
    let file: File = File::open(&abs_file_path).expect("File does not exist");
    let mut file_reader: BufReader<File> = BufReader::new(file);
    let mut file_bytes: Vec<u8> = Vec::with_capacity(file_reader.capacity());
    file_reader
        .read_to_end(&mut file_bytes)
        .expect("Could not read whole file");

    let mut reader = BinaryReader::new(&file_bytes, Endianness::Big);
    let cf: ClassFile = parse_class_file(&mut reader);

    print_header(&mut lw, &cf);
    print_constant_pool(&mut lw, &cf.constant_pool);
    lw.println("{");
    print_fields(&cf.constant_pool, &cf.fields);
    print_methods(
        &cf.constant_pool,
        cf.this_class,
        cf.access_flags
            .contains(&access_flags::ClassAccessFlag::Enum),
        &cf.methods,
    );
    lw.println("}");
    print_class_attributes(&cf.constant_pool, &cf.attributes);
}

/**
 * Returns the index of the column (on the terminal) where the index of each constant pool entry ends.
 */
fn get_constant_pool_index_width(cp: &ConstantPool) -> usize {
    4 + num_digits(cp.len())
}

/**
 * Returns the index of the column (on the terminal) where the information of each entry is displayed.
 */
fn get_constant_pool_info_start_index(cp: &ConstantPool) -> usize {
    26 + num_digits(cp.len())
}

/**
 * Returns the index of the column (on the terminal) where the comments (the '//') start for the constant pool.
 */
fn get_constant_pool_comment_start_index(cp: &ConstantPool) -> usize {
    40 + num_digits(cp.len())
}

fn num_digits(n: usize) -> usize {
    (n as f64).log10().floor() as usize
}

fn print_header(lw: &mut LineWriter, cf: &ClassFile) {
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
    lw.print("  Compiled from \"")
        .print(&source_file)
        .println("\"");

    let this_class_name = cf
        .constant_pool
        .get_class_name(cf.this_class)
        .replace('/', ".");
    lw.print(&access_flags::modifier_repr_vec(&cf.access_flags))
        .print(" ")
        .print(&this_class_name);

    let is_enum: bool = cf
        .access_flags
        .contains(&access_flags::ClassAccessFlag::Enum);
    let super_class_name = cf.constant_pool.get_class_name(cf.super_class);
    if is_enum {
        lw.print(" extends java.lang.Enum<")
            .print(&this_class_name)
            .println(">");
    } else if super_class_name != "java/lang/Object" {
        lw.print(" extends ")
            .println(&super_class_name.replace('/', "."));
    } else {
        lw.println("");
    }

    lw.indent(1);

    lw.print("minor version: ")
        .println(&cf.minor_version.to_string())
        .print("major version: ")
        .println(&cf.major_version.to_string())
        .print("flags: (")
        .print(&format!("0x{:04x}", access_flags::to_u16(&cf.access_flags)))
        .print(") ")
        .println(&access_flags::java_repr_vec(&cf.access_flags));
    lw.print("this_class: #")
        .print(&cf.this_class.to_string())
        .tab()
        .print("// ")
        .println(&cf.constant_pool.get_class_name(cf.this_class));
    lw.print("super_class: #")
        .print(&cf.super_class.to_string())
        .tab()
        .print("// ")
        .println(&cf.constant_pool.get_class_name(cf.super_class));
    lw.print("interfaces: ")
        .print(&cf.interfaces.len().to_string())
        .print(", fields: ")
        .print(&cf.fields.len().to_string())
        .print(", methods: ")
        .print(&cf.methods.len().to_string())
        .print(", attributes: ")
        .println(&cf.attributes.len().to_string());

    lw.indent(-1);
}

fn print_constant_pool(lw: &mut LineWriter, cp: &ConstantPool) {
    let index_width: usize = get_constant_pool_index_width(cp);
    let info_start_index: usize = get_constant_pool_info_start_index(cp);
    let comment_index: usize = get_constant_pool_comment_start_index(cp);

    lw.println("Constant pool:");
    lw.indent(1);

    let width = cp.len().to_string().len() + 1;

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

        lw.print(&format!(
            "{:>width$}",
            ("#".to_owned() + &(i + 1).to_string())
        ));

        let entry = &cp[i.try_into().unwrap()];

        lw.print(&format!(" = {:<18} ", entry.tag()));

        match entry {
            ConstantPoolInfo::Utf8 { bytes } => {
                let content: String = constant_pool::convert_utf8(bytes).trim_end().to_owned();
                if !content.trim().is_empty() {
                    lw.tab().println(&content);
                }
            }
            ConstantPoolInfo::Integer { bytes } => {
                lw.tab().println(&bytes.to_string());
            }
            ConstantPoolInfo::Float { bytes } => println!(
                "{:<info_start_index$}{:.1}",
                format!("{:>index_width$} = Float", format!("#{}", i + 1),),
                f32::from_bits(*bytes),
            ),
            ConstantPoolInfo::Long {
                high_bytes,
                low_bytes,
            } => println!(
                "{:<info_start_index$}{}l",
                format!("{:>index_width$} = Long", format!("#{}", i + 1),),
                (((*high_bytes as u64) << 32) | (*low_bytes as u64)) as i64,
            ),
            ConstantPoolInfo::Double {
                high_bytes,
                low_bytes,
            } => println!(
                "{:<info_start_index$}{:.1}d",
                format!("{:>index_width$} = Double", format!("#{}", i + 1),),
                f64::from_bits(((*high_bytes as u64) << 32) | (*low_bytes as u64)),
            ),
            ConstantPoolInfo::String { string_index } => {
                lw.print(&format!("#{}", string_index)).tab();
                let string_content: String =
                    cp.get_utf8_content(*string_index).trim_end().to_owned();
                if string_content.trim().is_empty() {
                    lw.println("//");
                } else {
                    lw.print("// ").println(&string_content);
                }
            }
            ConstantPoolInfo::Class { name_index } => {
                lw.print("#")
                    .print(&name_index.to_string())
                    .tab()
                    .print("// ")
                    .println(&cp.get_wrapped_utf8_content(*name_index));
            }
            ConstantPoolInfo::FieldRef {
                class_index,
                name_and_type_index,
            } => {
                lw.print(&format!("#{}.#{}", class_index, name_and_type_index))
                    .tab()
                    .print("// ")
                    .println(&cp.get_field_ref_string(*class_index, *name_and_type_index));
            }
            ConstantPoolInfo::MethodRef {
                class_index,
                name_and_type_index,
            } => {
                lw.print(&format!("#{}.#{}", class_index, name_and_type_index))
                    .tab()
                    .print("// ")
                    .println(&cp.get_method_ref_string(*class_index, *name_and_type_index));
            }
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
            } => {
                lw.print(&format!("#{}:#{}", name_index, descriptor_index))
                    .tab()
                    .print("// ")
                    .println(&cp.get_name_and_type_string(*name_index, *descriptor_index));
            }
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

    lw.indent(-1);
}

fn print_fields(cp: &ConstantPool, fields: &[FieldInfo]) {
    for field in fields.iter() {
        let descriptor: String = cp.get_utf8_content(field.descriptor_index);
        let signature: Option<&AttributeInfo> =
            find_attribute(&field.attributes, AttributeKind::Signature);
        println!(
            "  {} {} {};",
            access_flags::modifier_repr_vec(&field.access_flags),
            match signature {
                Some(AttributeInfo::Signature { signature_index }) =>
                    descriptor::parse_field_descriptor(&cp.get_utf8_content(*signature_index)),
                Some(_) => unreachable!(),
                None => descriptor::parse_field_descriptor(&descriptor),
            },
            cp.get_utf8_content(field.name_index)
        );
        println!("    descriptor: {}", descriptor);
        println!(
            "    flags: (0x{:04x}) {}",
            access_flags::to_u16(&field.access_flags),
            access_flags::java_repr_vec(&field.access_flags)
        );
        print_field_attributes(cp, field);
        println!();
    }
}

fn print_field_attributes(cp: &ConstantPool, field: &FieldInfo) {
    let comment_index: usize = get_constant_pool_comment_start_index(cp);
    for attribute in field.attributes.iter() {
        match attribute {
            AttributeInfo::Signature { signature_index } => {
                println!(
                    "{:<width$}// {}",
                    format!("    Signature: #{}", signature_index),
                    cp.get_utf8_content(*signature_index),
                    width = comment_index + 2 // why?
                );
            }
            AttributeInfo::ConstantValue {
                constant_value_index,
            } => {
                println!("ConstantValue: #{}", constant_value_index);
            }
            _ => unreachable!("Unknown field attribute {}.", attribute.kind()),
        }
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
            let mut param_types = parsed_descriptor
                .parameter_types
                .iter()
                .map(|t| format!("{}", t))
                .collect::<Vec<String>>()
                .join(", ");

            if method.access_flags.contains(&MethodAccessFlag::Varargs) {
                param_types = param_types[..(param_types.len() - 2)].to_owned() + "...";
            }

            println!(
                "{} {}({});",
                parsed_descriptor.return_type, method_name, param_types
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
        BytecodeInstruction::FConst { constant } => {
            if *constant <= 2.0f32 {
                "fconst_".to_owned() + &constant.to_string()
            } else {
                "fconst    ".to_owned() + &constant.to_string()
            }
        }
        BytecodeInstruction::DConst { constant } => {
            if *constant <= 1.0 {
                "dconst_".to_owned() + &constant.to_string()
            } else {
                "dconst    ".to_owned() + &constant.to_string()
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
        BytecodeInstruction::FLoad {
            local_variable_index,
        } => {
            if *local_variable_index <= 3 {
                "fload_".to_owned() + &local_variable_index.to_string()
            } else {
                "fload         ".to_owned() + &local_variable_index.to_string()
            }
        }
        BytecodeInstruction::FStore {
            local_variable_index,
        } => {
            if *local_variable_index <= 3 {
                "fstore_".to_owned() + &local_variable_index.to_string()
            } else {
                "fstore        ".to_owned() + &local_variable_index.to_string()
            }
        }
        BytecodeInstruction::DLoad {
            local_variable_index,
        } => {
            if *local_variable_index <= 3 {
                "dload_".to_owned() + &local_variable_index.to_string()
            } else {
                "dload         ".to_owned() + &local_variable_index.to_string()
            }
        }
        BytecodeInstruction::DStore {
            local_variable_index,
        } => {
            if *local_variable_index <= 3 {
                "dstore_".to_owned() + &local_variable_index.to_string()
            } else {
                "dstore        ".to_owned() + &local_variable_index.to_string()
            }
        }
        BytecodeInstruction::AaLoad {} => "aaload".to_owned(),
        BytecodeInstruction::BaLoad {} => "baload".to_owned(),
        BytecodeInstruction::AaStore {} => "aastore".to_owned(),
        BytecodeInstruction::BaStore {} => "bastore".to_owned(),
        BytecodeInstruction::CaStore {} => "castore".to_owned(),
        BytecodeInstruction::SaStore {} => "sastore".to_owned(),
        BytecodeInstruction::NewArray { atype } => {
            "newarray       ".to_owned() + &format!("{}", atype)
        }
        BytecodeInstruction::ANewArray {
            constant_pool_index,
        } => "anewarray     #".to_owned() + &constant_pool_index.to_string(),
        BytecodeInstruction::AThrow {} => "athrow".to_owned(),
        BytecodeInstruction::New {
            constant_pool_index,
        } => "new           #".to_owned() + &constant_pool_index.to_string(),
        BytecodeInstruction::BiPush { immediate } => {
            "bipush        ".to_owned() + &(*immediate as i8).to_string()
        }
        BytecodeInstruction::SiPush { immediate } => {
            "sipush        ".to_owned() + &(*immediate as i16).to_string()
        }
        BytecodeInstruction::Pop {} => "pop".to_owned(),
        BytecodeInstruction::Pop2 {} => "pop2".to_owned(),
        BytecodeInstruction::Return {} => "return".to_owned(),
        BytecodeInstruction::IReturn {} => "ireturn".to_owned(),
        BytecodeInstruction::LReturn {} => "lreturn".to_owned(),
        BytecodeInstruction::FReturn {} => "freturn".to_owned(),
        BytecodeInstruction::DReturn {} => "dreturn".to_owned(),
        BytecodeInstruction::AReturn {} => "areturn".to_owned(),
        BytecodeInstruction::ArrayLength {} => "arraylength".to_owned(),
        BytecodeInstruction::LCmp {} => "lcmp".to_owned(),
        BytecodeInstruction::FCmpL {} => "fcmpl".to_owned(),
        BytecodeInstruction::FCmpG {} => "fcmpg".to_owned(),
        BytecodeInstruction::DCmpL {} => "dcmpl".to_owned(),
        BytecodeInstruction::DCmpG {} => "dcmpg".to_owned(),
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
        BytecodeInstruction::Instanceof {
            constant_pool_index,
        } => "instanceof    #".to_owned() + &constant_pool_index.to_string(),

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
        BytecodeInstruction::IfAcmpEq { offset } => {
            "if_acmpeq     ".to_owned() + &add_offset(*position, *offset).to_string()
        }
        BytecodeInstruction::IfAcmpNe { offset } => {
            "if_acmpne     ".to_owned() + &add_offset(*position, *offset).to_string()
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
        BytecodeInstruction::IfNull { offset } => {
            "ifnull        ".to_owned() + &add_offset(*position, *offset).to_string()
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

        BytecodeInstruction::I2L {} => "i2l".to_owned(),
        BytecodeInstruction::I2F {} => "i2f".to_owned(),
        BytecodeInstruction::I2D {} => "i2d".to_owned(),
        BytecodeInstruction::L2I {} => "l2i".to_owned(),
        BytecodeInstruction::L2F {} => "l2f".to_owned(),
        BytecodeInstruction::L2D {} => "l2d".to_owned(),
        BytecodeInstruction::F2I {} => "f2i".to_owned(),
        BytecodeInstruction::F2L {} => "f2l".to_owned(),
        BytecodeInstruction::F2D {} => "f2d".to_owned(),
        BytecodeInstruction::D2I {} => "d2i".to_owned(),
        BytecodeInstruction::D2L {} => "d2l".to_owned(),
        BytecodeInstruction::D2F {} => "d2f".to_owned(),
        BytecodeInstruction::I2B {} => "i2b".to_owned(),
        BytecodeInstruction::I2C {} => "i2c".to_owned(),
        BytecodeInstruction::I2S {} => "i2s".to_owned(),

        BytecodeInstruction::IAdd {} => "iadd".to_owned(),
        BytecodeInstruction::ISub {} => "isub".to_owned(),
        BytecodeInstruction::IMul {} => "imul".to_owned(),
        BytecodeInstruction::IDiv {} => "idiv".to_owned(),
        BytecodeInstruction::IRem {} => "irem".to_owned(),
        BytecodeInstruction::IAnd {} => "iand".to_owned(),
        BytecodeInstruction::IShl {} => "ishl".to_owned(),
        BytecodeInstruction::IShr {} => "ishr".to_owned(),
        BytecodeInstruction::IUshr {} => "iushr".to_owned(),
        BytecodeInstruction::IOr {} => "ior".to_owned(),
        BytecodeInstruction::IXor {} => "ixor".to_owned(),
        BytecodeInstruction::INeg {} => "ineg".to_owned(),

        BytecodeInstruction::LAdd {} => "ladd".to_owned(),
        BytecodeInstruction::LSub {} => "lsub".to_owned(),
        BytecodeInstruction::LMul {} => "lmul".to_owned(),
        BytecodeInstruction::LDiv {} => "ldiv".to_owned(),
        BytecodeInstruction::LRem {} => "lrem".to_owned(),
        BytecodeInstruction::LAnd {} => "land".to_owned(),
        BytecodeInstruction::LShl {} => "lshl".to_owned(),
        BytecodeInstruction::LShr {} => "lshr".to_owned(),
        BytecodeInstruction::LUshr {} => "lushr".to_owned(),
        BytecodeInstruction::LOr {} => "lor".to_owned(),
        BytecodeInstruction::LXor {} => "lxor".to_owned(),
        BytecodeInstruction::LNeg {} => "lneg".to_owned(),

        BytecodeInstruction::FAdd {} => "fadd".to_owned(),
        BytecodeInstruction::FMul {} => "fmul".to_owned(),
        BytecodeInstruction::FNeg {} => "fneg".to_owned(),
        BytecodeInstruction::FDiv {} => "fdiv".to_owned(),
        BytecodeInstruction::FRem {} => "frem".to_owned(),
        BytecodeInstruction::FSub {} => "fsub".to_owned(),

        BytecodeInstruction::DAdd {} => "dadd".to_owned(),
        BytecodeInstruction::DMul {} => "dmul".to_owned(),
        BytecodeInstruction::DNeg {} => "dneg".to_owned(),
        BytecodeInstruction::DDiv {} => "ddiv".to_owned(),
        BytecodeInstruction::DRem {} => "drem".to_owned(),
        BytecodeInstruction::DSub {} => "dsub".to_owned(),
    }
}

fn get_constant_string(cp: &ConstantPool, constant_pool_index: u16) -> String {
    let entry = &cp[constant_pool_index - 1];
    match entry {
        ConstantPoolInfo::String { string_index } => {
            let string_content: String = cp.get_utf8_content(*string_index).trim_end().to_owned();
            if string_content.trim().is_empty() {
                "String".to_owned()
            } else {
                "String ".to_owned() + &string_content
            }
        }
        ConstantPoolInfo::Integer { bytes } => "int ".to_owned() + &(*bytes as i32).to_string(),
        ConstantPoolInfo::Long {
            high_bytes,
            low_bytes,
        } => {
            "long ".to_owned()
                + &((((*high_bytes as u64) << 32) | (*low_bytes as u64)) as i64).to_string()
                + "l"
        }
        ConstantPoolInfo::Float { bytes } => format!("float {:.1}f", &f32::from_bits(*bytes)),
        ConstantPoolInfo::Double {
            high_bytes,
            low_bytes,
        } => {
            format!(
                "double {:.1E}d",
                &f64::from_bits(((*high_bytes as u64) << 32) | (*low_bytes as u64))
            )
        }
        ConstantPoolInfo::Class { name_index } => {
            "class ".to_owned() + &cp.get_utf8_content(*name_index)
        }
        _ => unreachable!(
            "Unknown CP entry to get constant string from: {}.",
            entry.tag()
        ),
    }
}

fn get_method_type(cpe: &ConstantPoolInfo) -> String {
    match cpe {
        ConstantPoolInfo::MethodRef { .. } => "Method",
        ConstantPoolInfo::InterfaceMethodRef { .. } => "InterfaceMethod",
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
        BytecodeInstruction::IConst { .. } => None,
        BytecodeInstruction::LConst { .. } => None,
        BytecodeInstruction::FConst { .. } => None,
        BytecodeInstruction::DConst { .. } => None,
        BytecodeInstruction::Ldc {
            constant_pool_index,
        } => Some(get_constant_string(cp, (*constant_pool_index).into())),
        BytecodeInstruction::LdcW {
            constant_pool_index,
        } => Some(get_constant_string(cp, *constant_pool_index)),
        BytecodeInstruction::Ldc2W {
            constant_pool_index,
        } => Some(get_constant_string(cp, *constant_pool_index)),
        BytecodeInstruction::ALoad { .. } => None,
        BytecodeInstruction::AStore { .. } => None,
        BytecodeInstruction::ILoad { .. } => None,
        BytecodeInstruction::IStore { .. } => None,
        BytecodeInstruction::LLoad { .. } => None,
        BytecodeInstruction::LStore { .. } => None,
        BytecodeInstruction::FLoad { .. } => None,
        BytecodeInstruction::FStore { .. } => None,
        BytecodeInstruction::DLoad { .. } => None,
        BytecodeInstruction::DStore { .. } => None,
        BytecodeInstruction::AaLoad {} => None,
        BytecodeInstruction::BaLoad {} => None,
        BytecodeInstruction::AaStore {} => None,
        BytecodeInstruction::BaStore {} => None,
        BytecodeInstruction::CaStore {} => None,
        BytecodeInstruction::SaStore {} => None,
        BytecodeInstruction::NewArray { .. } => None,
        BytecodeInstruction::ANewArray {
            constant_pool_index,
        } => Some("class ".to_owned() + &cp.get_class_name(*constant_pool_index)),
        BytecodeInstruction::AThrow {} => None,
        BytecodeInstruction::New {
            constant_pool_index,
        } => Some("class ".to_owned() + &cp.get_class_name(*constant_pool_index)),
        BytecodeInstruction::BiPush { .. } => None,
        BytecodeInstruction::SiPush { .. } => None,
        BytecodeInstruction::Pop {} => None,
        BytecodeInstruction::Pop2 {} => None,
        BytecodeInstruction::Return {} => None,
        BytecodeInstruction::IReturn {} => None,
        BytecodeInstruction::LReturn {} => None,
        BytecodeInstruction::FReturn {} => None,
        BytecodeInstruction::DReturn {} => None,
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
        BytecodeInstruction::InvokeDynamic {
            constant_pool_index,
        } => Some("InvokeDynamic ".to_owned() + &cp.get_invoke_dynamic(*constant_pool_index)),
        BytecodeInstruction::InvokeInterface {
            constant_pool_index,
            ..
        } => Some(
            get_method_type(&cp[constant_pool_index - 1])
                + " "
                + &cp.get_method_ref(*constant_pool_index),
        ),
        BytecodeInstruction::ArrayLength {} => None,
        BytecodeInstruction::LCmp {} => None,
        BytecodeInstruction::FCmpL {} => None,
        BytecodeInstruction::FCmpG {} => None,
        BytecodeInstruction::DCmpL {} => None,
        BytecodeInstruction::DCmpG {} => None,
        BytecodeInstruction::IfIcmpEq { .. } => None,
        BytecodeInstruction::IfIcmpNe { .. } => None,
        BytecodeInstruction::IfIcmpLt { .. } => None,
        BytecodeInstruction::IfIcmpGe { .. } => None,
        BytecodeInstruction::IfIcmpGt { .. } => None,
        BytecodeInstruction::IfIcmpLe { .. } => None,
        BytecodeInstruction::IfAcmpEq { .. } => None,
        BytecodeInstruction::IfAcmpNe { .. } => None,
        BytecodeInstruction::IfEq { .. } => None,
        BytecodeInstruction::IfNe { .. } => None,
        BytecodeInstruction::IfLt { .. } => None,
        BytecodeInstruction::IfGe { .. } => None,
        BytecodeInstruction::IfGt { .. } => None,
        BytecodeInstruction::IfLe { .. } => None,
        BytecodeInstruction::IfNull { .. } => None,
        BytecodeInstruction::IfNonNull { .. } => None,
        BytecodeInstruction::GoTo { .. } => None,
        BytecodeInstruction::TableSwitch { .. } => None,
        BytecodeInstruction::LookupSwitch { .. } => None,
        BytecodeInstruction::CheckCast {
            constant_pool_index,
        } => Some("class ".to_owned() + &cp.get_class_name(*constant_pool_index)),
        BytecodeInstruction::Instanceof {
            constant_pool_index,
        } => Some("class ".to_owned() + &cp.get_class_name(*constant_pool_index)),

        BytecodeInstruction::IInc { .. } => None,

        BytecodeInstruction::I2L {} => None,
        BytecodeInstruction::I2F {} => None,
        BytecodeInstruction::I2D {} => None,
        BytecodeInstruction::L2I {} => None,
        BytecodeInstruction::L2F {} => None,
        BytecodeInstruction::L2D {} => None,
        BytecodeInstruction::F2I {} => None,
        BytecodeInstruction::F2L {} => None,
        BytecodeInstruction::F2D {} => None,
        BytecodeInstruction::D2I {} => None,
        BytecodeInstruction::D2L {} => None,
        BytecodeInstruction::D2F {} => None,
        BytecodeInstruction::I2B {} => None,
        BytecodeInstruction::I2C {} => None,
        BytecodeInstruction::I2S {} => None,

        BytecodeInstruction::IAdd {} => None,
        BytecodeInstruction::ISub {} => None,
        BytecodeInstruction::IMul {} => None,
        BytecodeInstruction::IDiv {} => None,
        BytecodeInstruction::IRem {} => None,
        BytecodeInstruction::IAnd {} => None,
        BytecodeInstruction::IShl {} => None,
        BytecodeInstruction::IShr {} => None,
        BytecodeInstruction::IUshr {} => None,
        BytecodeInstruction::IOr {} => None,
        BytecodeInstruction::IXor {} => None,
        BytecodeInstruction::INeg {} => None,

        BytecodeInstruction::LAdd {} => None,
        BytecodeInstruction::LSub {} => None,
        BytecodeInstruction::LMul {} => None,
        BytecodeInstruction::LDiv {} => None,
        BytecodeInstruction::LRem {} => None,
        BytecodeInstruction::LAnd {} => None,
        BytecodeInstruction::LShl {} => None,
        BytecodeInstruction::LShr {} => None,
        BytecodeInstruction::LUshr {} => None,
        BytecodeInstruction::LOr {} => None,
        BytecodeInstruction::LXor {} => None,
        BytecodeInstruction::LNeg {} => None,

        BytecodeInstruction::FAdd {} => None,
        BytecodeInstruction::FMul {} => None,
        BytecodeInstruction::FNeg {} => None,
        BytecodeInstruction::FDiv {} => None,
        BytecodeInstruction::FRem {} => None,
        BytecodeInstruction::FSub {} => None,

        BytecodeInstruction::DAdd {} => None,
        BytecodeInstruction::DMul {} => None,
        BytecodeInstruction::DNeg {} => None,
        BytecodeInstruction::DDiv {} => None,
        BytecodeInstruction::DRem {} => None,
        BytecodeInstruction::DSub {} => None,
    }
}

fn get_verification_type_info_string(cp: &ConstantPool, vti: &VerificationTypeInfo) -> String {
    match vti {
        VerificationTypeInfo::TopVariable => "top".to_owned(),
        VerificationTypeInfo::IntegerVariable => "int".to_owned(),
        VerificationTypeInfo::FloatVariable => "float".to_owned(),
        VerificationTypeInfo::LongVariable => "long".to_owned(),
        VerificationTypeInfo::DoubleVariable => "double".to_owned(),
        VerificationTypeInfo::NullVariable => "null".to_owned(),
        VerificationTypeInfo::UninitializedThisVariable => todo!(),
        VerificationTypeInfo::ObjectVariable {
            constant_pool_index,
        } => "class ".to_owned() + &cp.get_class_name(*constant_pool_index),
        VerificationTypeInfo::UninitializedVariable { .. } => todo!(),
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
            AttributeInfo::RuntimeVisibleAnnotations { annotations } => {
                println!("RuntimeVisibleAnnotations:");
                for annotation in annotations {
                    let annotation_type = cp.get_utf8_content(annotation.type_index);
                    println!(
                        "{} {}",
                        annotation_type,
                        annotation.element_value_pairs.len()
                    );
                }
            }
            _ => unreachable!("Unknown method attribute {}.", attribute.kind()),
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
            AttributeInfo::LocalVariableTypeTable {
                local_variable_type_table,
            } => {
                println!("      LocalVariableTypeTable:");
                println!("        Start  Length  Slot  Name   Signature");
                for entry in local_variable_type_table.iter() {
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
            AttributeInfo::NestMembers { classes } => {
                println!("NestMembers:");
                for class_index in classes {
                    println!("  {}", cp.get_class_name(*class_index));
                }
            }
            _ => unreachable!(),
        }
    }
}
