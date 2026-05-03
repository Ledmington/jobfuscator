use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use binary_reader::{BinaryReader, Endianness};
use classfile::access_flags::{ClassAccessFlag, MethodAccessFlag};
use classfile::attributes::ElementValue;
use classfile::attributes::{
    AttributeInfo, AttributeKind, StackMapFrame, VerificationTypeInfo, find_attribute,
};
use classfile::bytecode::BytecodeInstruction;
use classfile::classfile::{ClassFile, parse_class_file};
use classfile::constant_pool::{self, ConstantPool, ConstantPoolInfo};
use classfile::descriptor::{ClassSignature, decode_class_signature, decode_type};
use classfile::fields::FieldInfo;
use classfile::methods::MethodInfo;
use classfile::reference_kind;
use classfile::utils::absolute_no_symlinks;
use date::Date;

use crate::line_writer::LineWriter;

/**
 * The maximum length (in characters) of the index of a single bytecode instruction.
 */
const BYTECODE_INDEX_LENGTH: usize = 4;

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

    let date: Date = Date::from(modified_time);

    lw.print("  Last modified ")
        .print(&date.month())
        .print(" ")
        .print(&date.day())
        .print(", ")
        .print(&date.year())
        .print("; size ")
        .print(&file_size.to_string())
        .println(" bytes");

    lw.print("  SHA-256 checksum ").println(
        &digest
            .iter()
            .map(|x| format!("{x:02x}"))
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
    lw.indent(1);
    print_fields(&mut lw, &cf.constant_pool, &cf.fields);
    print_methods(&mut lw, &cf.constant_pool, &cf, cf.this_class, &cf.methods);
    lw.indent(-1);
    lw.println("}");
    print_class_attributes(&mut lw, &cf.constant_pool, &cf.attributes);
}

fn print_header(lw: &mut LineWriter, cf: &ClassFile) {
    let source_file: String = cf
        .attributes
        .iter()
        .filter(|attr| matches!(attr, AttributeInfo::SourceFile { .. }))
        .map(|attr| match attr {
            AttributeInfo::SourceFile {
                source_file_index, ..
            } => cf.constant_pool.get_utf8_content(*source_file_index),
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

    lw.print(&cf.access_flags.modifier_repr())
        .print(" ")
        .print(&this_class_name);

    let this_class_signature = find_attribute(&cf.attributes, AttributeKind::Signature);
    if let Some(AttributeInfo::Signature {
        signature_index, ..
    }) = this_class_signature
    {
        let is_interface: bool = cf.access_flags.contains(ClassAccessFlag::Interface);
        let decoded: ClassSignature =
            decode_class_signature(&cf.constant_pool.get_utf8_content(*signature_index));

        let actual_super_class: String = decoded.super_class_name.clone();

        if !decoded.generic_type_bounds.is_empty() {
            lw.print(&format!(
                "<{}>",
                decoded
                    .generic_type_bounds
                    .iter()
                    .map(|gtb| format!("{} extends {}", gtb.type_name, gtb.type_bounds.join(", ")))
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
        }

        if is_interface {
            lw.print(&format!(" extends {}", decoded.interfaces.join(", ")));
        } else {
            lw.print(&format!(" extends {actual_super_class}"));
        }
    }

    lw.println("");

    lw.indent(1);

    lw.print("minor version: ")
        .println(&cf.minor_version.to_string())
        .print("major version: ")
        .println(&cf.major_version.to_string())
        .print("flags: (")
        .print(&format!("0x{:04x}", cf.access_flags.to_u16()))
        .print(") ")
        .println(&cf.access_flags.java_repr());
    lw.print("this_class: #")
        .print(&cf.this_class.to_string())
        .tab()
        .print("// ")
        .println(&cf.constant_pool.get_class_name(cf.this_class));
    lw.print("super_class: #")
        .print(&cf.super_class.to_string());
    if cf.super_class != 0 {
        lw.tab()
            .print("// ")
            .print(&cf.constant_pool.get_class_name(cf.super_class));
    }
    lw.println("");
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
            && (matches!(cp[i.try_into().unwrap()], ConstantPoolInfo::Long { .. })
                || matches!(cp[i.try_into().unwrap()], ConstantPoolInfo::Double { .. }))
        {
            continue;
        }

        lw.print(&format!(
            "{:>width$}",
            ("#".to_owned() + &(i + 1).to_string())
        ));

        let entry = &cp[(i + 1).try_into().unwrap()];

        lw.print(&format!(" = {:<18} ", entry.tag().to_string()));

        match entry {
            ConstantPoolInfo::Utf8 { bytes } => {
                let content: String = constant_pool::convert_utf8(bytes).trim_end().to_owned();
                if !content.trim().is_empty() {
                    lw.print(&content);
                }
                lw.println("");
            }
            ConstantPoolInfo::Integer { bytes } => {
                lw.println(&(*bytes as i32).to_string());
            }
            ConstantPoolInfo::Float { bytes } => {
                lw.println(&format!("{}f", java_format_float(f32::from_bits(*bytes))));
            }
            ConstantPoolInfo::Long {
                high_bytes,
                low_bytes,
            } => {
                lw.println(&format!("{}l", get_long_value(*high_bytes, *low_bytes)));
            }
            ConstantPoolInfo::Double {
                high_bytes,
                low_bytes,
            } => {
                lw.println(&format!(
                    "{}d",
                    java_format_double(get_double_value(*high_bytes, *low_bytes))
                ));
            }
            ConstantPoolInfo::String { string_index } => {
                lw.print(&format!("#{string_index}")).tab();
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
                lw.print(&format!("#{class_index}.#{name_and_type_index}"))
                    .tab()
                    .print("// ")
                    .println(&cp.get_field_ref_string(*class_index, *name_and_type_index));
            }
            ConstantPoolInfo::MethodRef {
                class_index,
                name_and_type_index,
            } => {
                lw.print(&format!("#{class_index}.#{name_and_type_index}"))
                    .tab()
                    .print("// ")
                    .println(&cp.get_method_ref_string(*class_index, *name_and_type_index));
            }
            ConstantPoolInfo::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => {
                lw.print(&format!("#{class_index}.#{name_and_type_index}"))
                    .tab()
                    .print("// ")
                    .println(&cp.get_method_ref_string(*class_index, *name_and_type_index));
            }
            ConstantPoolInfo::NameAndType {
                name_index,
                descriptor_index,
            } => {
                lw.print(&format!("#{name_index}:#{descriptor_index}"))
                    .tab()
                    .print("// ")
                    .println(&cp.get_name_and_type_string(*name_index, *descriptor_index));
            }
            ConstantPoolInfo::MethodType { descriptor_index } => {
                lw.print(&format!("#{descriptor_index}"))
                    .tab()
                    .print("//  ")
                    .println(&cp.get_utf8_content(*descriptor_index));
            }
            ConstantPoolInfo::MethodHandle {
                reference_kind,
                reference_index,
            } => {
                let ref_kind: u8 = *reference_kind as u8;
                lw.print(&format!("{ref_kind}:#{reference_index}"))
                    .tab()
                    .print("// ")
                    .println(&format!(
                        "{} {}",
                        reference_kind::java_repr(*reference_kind),
                        cp.get_method_ref(*reference_index)
                    ));
            }
            ConstantPoolInfo::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => {
                lw.print(&format!(
                    "#{bootstrap_method_attr_index}:#{name_and_type_index}"
                ))
                .tab()
                .print("// ")
                .println(
                    &cp.get_invoke_dynamic_string(
                        *bootstrap_method_attr_index,
                        *name_and_type_index,
                    ),
                );
            }
            ConstantPoolInfo::Null {} => unreachable!(),
        }
    }

    lw.indent(-1);
}

fn print_fields(lw: &mut LineWriter, cp: &ConstantPool, fields: &[FieldInfo]) {
    for field in fields.iter() {
        let descriptor: String = cp.get_utf8_content(field.descriptor_index);
        let signature: Option<&AttributeInfo> =
            find_attribute(&field.attributes, AttributeKind::Signature);
        lw.println(&format!(
            "{} {} {};",
            field.access_flags.modifier_repr(),
            match signature {
                Some(AttributeInfo::Signature {
                    signature_index, ..
                }) => decode_type(&cp.get_utf8_content(*signature_index)),
                Some(_) => unreachable!(),
                None => decode_type(&descriptor),
            },
            cp.get_utf8_content(field.name_index)
        ));

        lw.indent(1);

        lw.println(&format!("descriptor: {descriptor}"));
        lw.println(&format!(
            "flags: (0x{:04x}) {}",
            field.access_flags.to_u16(),
            field.access_flags.java_repr()
        ));
        print_field_attributes(lw, cp, field);

        lw.indent(-1);

        lw.println("");
    }
}

fn print_field_attributes(lw: &mut LineWriter, cp: &ConstantPool, field: &FieldInfo) {
    for attribute in field.attributes.iter() {
        match attribute {
            AttributeInfo::Signature {
                signature_index, ..
            } => {
                lw.print(&format!("Signature: #{signature_index}"))
                    .tab()
                    .println(&format!("// {}", cp.get_utf8_content(*signature_index)));
            }
            AttributeInfo::ConstantValue {
                constant_value_index,
                ..
            } => {
                lw.println(&format!(
                    "ConstantValue: {}",
                    get_constant_string(cp, *constant_value_index)
                ));
            }
            _ => unreachable!("Unknown field attribute {}.", attribute.kind()),
        }
    }
}

fn print_methods(
    lw: &mut LineWriter,
    cp: &ConstantPool,
    cf: &ClassFile,
    this_class: u16,
    methods: &[MethodInfo],
) {
    for (i, method) in methods.iter().enumerate() {
        let method_name: String = cp.get_utf8_content(method.name_index);
        let raw_descriptor: String = cp.get_utf8_content(method.descriptor_index);

        let signature = find_attribute(&method.attributes, AttributeKind::Signature);

        let parsed_descriptor: String = match signature {
            Some(AttributeInfo::Signature {
                signature_index, ..
            }) => decode_type(&cp.get_utf8_content(*signature_index)),
            _ => decode_type(&raw_descriptor),
        };

        if i > 0 {
            lw.println("");
        }

        if method.access_flags.to_u16() != 0x00u16 {
            lw.print(&format!("{} ", method.access_flags.modifier_repr()));
        }

        let is_class_initializer: bool = method_name == "<clinit>";

        // This obscure condition has been copied from the original javap source code
        // https://github.com/openjdk/jdk/blob/08b25611f688ae85c05242afc4cee5b538db4f67/src/jdk.jdeps/share/classes/com/sun/tools/javap/ClassWriter.java#L493
        if cf.access_flags.contains(ClassAccessFlag::Interface)
            && !method.access_flags.contains(MethodAccessFlag::Abstract)
            && !is_class_initializer
            && !method.access_flags.contains(MethodAccessFlag::Static)
            && !method.access_flags.contains(MethodAccessFlag::Private)
        {
            lw.print("default ");
        }

        let is_constructor: bool = method_name == "<init>";

        if is_class_initializer {
            // this is the 'static {}' block of the class
            lw.print("{}");
        } else {
            let first_bracket_index = parsed_descriptor.find('(').unwrap();
            let return_type: String = parsed_descriptor[0..first_bracket_index].to_owned();
            let mut arguments_string: String =
                parsed_descriptor[first_bracket_index..parsed_descriptor.len()].to_owned();

            if method.access_flags.contains(MethodAccessFlag::Varargs) {
                // replace last '[]' with '...'
                arguments_string =
                    arguments_string[..arguments_string.len() - 3].to_owned() + "...)";
            }

            if is_constructor {
                // this is a constructor of the class
                let this_class_name: String = cp.get_class_name(this_class).replace('/', ".");
                lw.print(&format!("{this_class_name}{arguments_string}"));
            } else {
                lw.print(&format!("{return_type} {method_name}{arguments_string}"));
            }
        }

        if let Some(AttributeInfo::Exceptions {
            exception_indices, ..
        }) = find_attribute(&method.attributes, AttributeKind::Exceptions)
        {
            lw.print(&format!(
                " throws {}",
                exception_indices
                    .iter()
                    .map(|exc_idx| cp.get_class_name(*exc_idx).replace('/', "."))
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
        }

        lw.println(";");

        lw.indent(1);

        lw.println(&format!("descriptor: {raw_descriptor}"));
        lw.println(&format!(
            "flags: (0x{:04x}) {}",
            method.access_flags.to_u16(),
            method.access_flags.java_repr()
        ));

        print_method_attributes(lw, cp, this_class, method);

        lw.indent(-1);
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
        BytecodeInstruction::Dup2 {} => "dup2".to_owned(),
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
        BytecodeInstruction::IaLoad {} => "iaload".to_owned(),
        BytecodeInstruction::LaLoad {} => "laload".to_owned(),
        BytecodeInstruction::FaLoad {} => "faload".to_owned(),
        BytecodeInstruction::DaLoad {} => "daload".to_owned(),
        BytecodeInstruction::AaLoad {} => "aaload".to_owned(),
        BytecodeInstruction::BaLoad {} => "baload".to_owned(),
        BytecodeInstruction::CaLoad {} => "caload".to_owned(),
        BytecodeInstruction::SaLoad {} => "saload".to_owned(),
        BytecodeInstruction::IaStore {} => "iastore".to_owned(),
        BytecodeInstruction::LaStore {} => "lastore".to_owned(),
        BytecodeInstruction::FaStore {} => "fastore".to_owned(),
        BytecodeInstruction::DaStore {} => "dastore".to_owned(),
        BytecodeInstruction::AaStore {} => "aastore".to_owned(),
        BytecodeInstruction::BaStore {} => "bastore".to_owned(),
        BytecodeInstruction::CaStore {} => "castore".to_owned(),
        BytecodeInstruction::SaStore {} => "sastore".to_owned(),
        BytecodeInstruction::NewArray { atype } => {
            "newarray       ".to_owned() + &format!("{atype}")
        }
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
        BytecodeInstruction::SiPush { immediate } => {
            "sipush        ".to_owned() + &immediate.to_string()
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
            ..
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
                        format!(
                            "       {:>11}: {}",
                            (*low as usize) + i,
                            add_offset(*position, *offset)
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
                + "\n           default: "
                + &add_offset(*position, *default).to_string()
                + "\n      }"
        }
        BytecodeInstruction::LookupSwitch { default, pairs, .. } => {
            "lookupswitch  { // ".to_owned()
                + &pairs.len().to_string()
                + "\n"
                + &pairs
                    .iter()
                    .map(|p| {
                        format!(
                            "       {:>11}: {}",
                            p.match_value,
                            add_offset(*position, p.offset)
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
                + "\n           default: "
                + &add_offset(*position, *default).to_string()
                + "\n      }"
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

fn get_long_value(high_bytes: u32, low_bytes: u32) -> i64 {
    (((high_bytes as u64) << 32) | (low_bytes as u64)) as i64
}

fn get_double_value(high_bytes: u32, low_bytes: u32) -> f64 {
    f64::from_bits(((high_bytes as u64) << 32) | (low_bytes as u64))
}

fn get_constant_string(cp: &ConstantPool, constant_pool_index: u16) -> String {
    let entry = &cp[constant_pool_index];
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
        } => "long ".to_owned() + &get_long_value(*high_bytes, *low_bytes).to_string() + "l",
        ConstantPoolInfo::Float { bytes } => {
            format!("float {}f", java_format_float(f32::from_bits(*bytes)))
        }
        ConstantPoolInfo::Double {
            high_bytes,
            low_bytes,
        } => {
            format!(
                "double {}d",
                java_format_double(get_double_value(*high_bytes, *low_bytes))
            )
        }
        ConstantPoolInfo::Class { name_index } => {
            "class ".to_owned() + &cp.get_wrapped_utf8_content(*name_index)
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
        BytecodeInstruction::Dup {}
        | BytecodeInstruction::Dup2 {}
        | BytecodeInstruction::AConstNull {}
        | BytecodeInstruction::IConst { .. }
        | BytecodeInstruction::LConst { .. }
        | BytecodeInstruction::FConst { .. }
        | BytecodeInstruction::DConst { .. }
        | BytecodeInstruction::ALoad { .. }
        | BytecodeInstruction::AStore { .. }
        | BytecodeInstruction::ILoad { .. }
        | BytecodeInstruction::IStore { .. }
        | BytecodeInstruction::LLoad { .. }
        | BytecodeInstruction::LStore { .. }
        | BytecodeInstruction::FLoad { .. }
        | BytecodeInstruction::FStore { .. }
        | BytecodeInstruction::DLoad { .. }
        | BytecodeInstruction::DStore { .. }
        | BytecodeInstruction::IaLoad {}
        | BytecodeInstruction::LaLoad {}
        | BytecodeInstruction::FaLoad {}
        | BytecodeInstruction::DaLoad {}
        | BytecodeInstruction::AaLoad {}
        | BytecodeInstruction::BaLoad {}
        | BytecodeInstruction::CaLoad {}
        | BytecodeInstruction::SaLoad {}
        | BytecodeInstruction::IaStore {}
        | BytecodeInstruction::LaStore {}
        | BytecodeInstruction::FaStore {}
        | BytecodeInstruction::DaStore {}
        | BytecodeInstruction::AaStore {}
        | BytecodeInstruction::BaStore {}
        | BytecodeInstruction::CaStore {}
        | BytecodeInstruction::SaStore {}
        | BytecodeInstruction::NewArray { .. }
        | BytecodeInstruction::AThrow {}
        | BytecodeInstruction::BiPush { .. }
        | BytecodeInstruction::SiPush { .. }
        | BytecodeInstruction::Pop {}
        | BytecodeInstruction::Pop2 {}
        | BytecodeInstruction::Return {}
        | BytecodeInstruction::IReturn {}
        | BytecodeInstruction::LReturn {}
        | BytecodeInstruction::FReturn {}
        | BytecodeInstruction::DReturn {}
        | BytecodeInstruction::AReturn {}
        | BytecodeInstruction::ArrayLength {}
        | BytecodeInstruction::LCmp {}
        | BytecodeInstruction::FCmpL {}
        | BytecodeInstruction::FCmpG {}
        | BytecodeInstruction::DCmpL {}
        | BytecodeInstruction::DCmpG {}
        | BytecodeInstruction::IfIcmpEq { .. }
        | BytecodeInstruction::IfIcmpNe { .. }
        | BytecodeInstruction::IfIcmpLt { .. }
        | BytecodeInstruction::IfIcmpGe { .. }
        | BytecodeInstruction::IfIcmpGt { .. }
        | BytecodeInstruction::IfIcmpLe { .. }
        | BytecodeInstruction::IfAcmpEq { .. }
        | BytecodeInstruction::IfAcmpNe { .. }
        | BytecodeInstruction::IfEq { .. }
        | BytecodeInstruction::IfNe { .. }
        | BytecodeInstruction::IfLt { .. }
        | BytecodeInstruction::IfGe { .. }
        | BytecodeInstruction::IfGt { .. }
        | BytecodeInstruction::IfLe { .. }
        | BytecodeInstruction::IfNull { .. }
        | BytecodeInstruction::IfNonNull { .. }
        | BytecodeInstruction::GoTo { .. }
        | BytecodeInstruction::TableSwitch { .. }
        | BytecodeInstruction::LookupSwitch { .. }
        | BytecodeInstruction::IInc { .. }
        | BytecodeInstruction::I2L {}
        | BytecodeInstruction::I2F {}
        | BytecodeInstruction::I2D {}
        | BytecodeInstruction::L2I {}
        | BytecodeInstruction::L2F {}
        | BytecodeInstruction::L2D {}
        | BytecodeInstruction::F2I {}
        | BytecodeInstruction::F2L {}
        | BytecodeInstruction::F2D {}
        | BytecodeInstruction::D2I {}
        | BytecodeInstruction::D2L {}
        | BytecodeInstruction::D2F {}
        | BytecodeInstruction::I2B {}
        | BytecodeInstruction::I2C {}
        | BytecodeInstruction::I2S {}
        | BytecodeInstruction::IAdd {}
        | BytecodeInstruction::ISub {}
        | BytecodeInstruction::IMul {}
        | BytecodeInstruction::IDiv {}
        | BytecodeInstruction::IRem {}
        | BytecodeInstruction::IAnd {}
        | BytecodeInstruction::IShl {}
        | BytecodeInstruction::IShr {}
        | BytecodeInstruction::IUshr {}
        | BytecodeInstruction::IOr {}
        | BytecodeInstruction::IXor {}
        | BytecodeInstruction::INeg {}
        | BytecodeInstruction::LAdd {}
        | BytecodeInstruction::LSub {}
        | BytecodeInstruction::LMul {}
        | BytecodeInstruction::LDiv {}
        | BytecodeInstruction::LRem {}
        | BytecodeInstruction::LAnd {}
        | BytecodeInstruction::LShl {}
        | BytecodeInstruction::LShr {}
        | BytecodeInstruction::LUshr {}
        | BytecodeInstruction::LOr {}
        | BytecodeInstruction::LXor {}
        | BytecodeInstruction::LNeg {}
        | BytecodeInstruction::FAdd {}
        | BytecodeInstruction::FMul {}
        | BytecodeInstruction::FNeg {}
        | BytecodeInstruction::FDiv {}
        | BytecodeInstruction::FRem {}
        | BytecodeInstruction::FSub {}
        | BytecodeInstruction::DAdd {}
        | BytecodeInstruction::DMul {}
        | BytecodeInstruction::DNeg {}
        | BytecodeInstruction::DDiv {}
        | BytecodeInstruction::DRem {}
        | BytecodeInstruction::DSub {} => None,

        BytecodeInstruction::Ldc {
            constant_pool_index,
        } => Some(get_constant_string(cp, (*constant_pool_index).into())),
        BytecodeInstruction::LdcW {
            constant_pool_index,
        } => Some(get_constant_string(cp, *constant_pool_index)),
        BytecodeInstruction::Ldc2W {
            constant_pool_index,
        } => Some(get_constant_string(cp, *constant_pool_index)),

        BytecodeInstruction::ANewArray {
            constant_pool_index,
        } => Some("class ".to_owned() + &cp.get_class_name(*constant_pool_index)),
        BytecodeInstruction::New {
            constant_pool_index,
        } => Some("class ".to_owned() + &cp.get_class_name(*constant_pool_index)),
        BytecodeInstruction::CheckCast {
            constant_pool_index,
        } => Some("class ".to_owned() + &cp.get_class_name(*constant_pool_index)),
        BytecodeInstruction::Instanceof {
            constant_pool_index,
        } => Some("class ".to_owned() + &cp.get_class_name(*constant_pool_index)),

        BytecodeInstruction::GetStatic { field_ref_index } => Some(
            "Field ".to_owned()
                + &match cp[*field_ref_index] {
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
                + &match cp[*field_ref_index] {
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
                + &match cp[*field_ref_index] {
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
                + &match cp[*field_ref_index] {
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
                + &match cp[*method_ref_index] {
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
            let method_entry = &cp[*method_ref_index];
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
            let method_entry = &cp[*method_ref_index];
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
        } => {
            let method_entry = &cp[*constant_pool_index];
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
                                cp.get_method_ref(*constant_pool_index)
                            }
                        }
                        ConstantPoolInfo::InterfaceMethodRef {
                            class_index,
                            name_and_type_index,
                        } => {
                            if *class_index == this_class {
                                cp.get_name_and_type(*name_and_type_index)
                            } else {
                                cp.get_method_ref(*constant_pool_index)
                            }
                        }
                        _ => unreachable!(),
                    },
            )
        }
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
        } => format!("class {}", cp.get_class_name(*constant_pool_index)),
        VerificationTypeInfo::UninitializedVariable { offset } => {
            format!("uninitialized {offset}")
        }
    }
}

fn get_number_of_arguments(cp: &ConstantPool, method: &MethodInfo) -> u8 {
    let descriptor: String = decode_type(&cp.get_utf8_content(method.descriptor_index));
    let arguments: String = descriptor
        .chars()
        .skip(descriptor.find('(').unwrap())
        .collect();

    let mut args: u8 = 1;
    if arguments == "()" {
        args = 0;
    } else {
        let mut generics = 0;
        for ch in arguments.chars() {
            if ch == '<' {
                generics += 1;
            } else if ch == '>' {
                generics -= 1;
                if generics < 0 {
                    panic!("Invalid generics syntax in method descriptor: '{arguments}'.",);
                }
            } else if ch == ',' && generics == 0 {
                args += 1;
            }
        }
    }

    if !method.access_flags.contains(MethodAccessFlag::Static) {
        // if the method is not static, there is the implicit 'this' argument
        args += 1;
    }

    args
}

fn print_method_attributes(
    lw: &mut LineWriter,
    cp: &ConstantPool,
    this_class: u16,
    method: &MethodInfo,
) {
    for attribute in method.attributes.iter() {
        match attribute {
            AttributeInfo::Code {
                max_stack,
                max_locals,
                code,
                exception_table,
                attributes,
                ..
            } => {
                lw.println("Code:");
                lw.indent(1);
                let args_size = get_number_of_arguments(cp, method);
                lw.println(&format!(
                    "stack={max_stack}, locals={max_locals}, args_size={args_size}"
                ));
                for (position, instruction) in code.iter() {
                    let opcode_and_arguments: String =
                        get_opcode_and_arguments_string(position, instruction);
                    let comment: Option<String> = get_comment(cp, this_class, instruction);
                    match comment {
                        Some(content) => {
                            lw.print(&format!(
                                "{position:>BYTECODE_INDEX_LENGTH$}: {opcode_and_arguments}"
                            ))
                            .tab()
                            .println(&format!("// {content}"));
                        }
                        None => {
                            lw.println(&format!(
                                "{position:>BYTECODE_INDEX_LENGTH$}: {opcode_and_arguments}",
                            ));
                        }
                    }
                }
                if !exception_table.is_empty() {
                    lw.println("Exception table:");
                    lw.indent(1);
                    lw.println(" from    to  target type");
                    for exception in exception_table.iter() {
                        lw.println(&format!(
                            " {:5} {:5} {:5}   Class {}",
                            exception.start_pc,
                            exception.end_pc,
                            exception.handler_pc,
                            cp.get_class_name(exception.catch_type)
                        ));
                    }
                    lw.indent(-1);
                }
                print_code_attributes(lw, cp, attributes);

                lw.indent(-1);
            }
            AttributeInfo::MethodParameters { parameters, .. } => {
                lw.println("MethodParameters:");
                lw.indent(1);
                lw.println("Name                           Flags");
                for param in parameters.iter() {
                    let name = if param.name_index == 0 {
                        "<no name>"
                    } else {
                        &cp.get_utf8_content(param.name_index)
                    };
                    lw.print(name);
                    if param.access_flags.to_u16() != 0 {
                        lw.print(&format!(
                            "                      {}",
                            param.access_flags.modifier_repr(),
                        ));
                    }
                    lw.println("");
                }
                lw.indent(-1);
            }
            AttributeInfo::Signature {
                signature_index, ..
            } => {
                lw.print(&format!("Signature: #{signature_index}"))
                    .tab()
                    .println(&format!("// {}", cp.get_utf8_content(*signature_index)));
            }
            AttributeInfo::RuntimeVisibleAnnotations { annotations, .. } => {
                lw.println("RuntimeVisibleAnnotations:");
                lw.indent(1);
                for (i, annotation) in annotations.iter().enumerate() {
                    let mut annotation_type = cp.get_utf8_content(annotation.type_index);
                    annotation_type =
                        annotation_type[1..annotation_type.len() - 1].replace('/', ".");
                    let has_values = !annotation.element_value_pairs.is_empty();
                    lw.print(&format!("{}: #{}(", i, annotation.type_index));
                    if has_values {
                        for (i, ev) in annotation.element_value_pairs.iter().enumerate() {
                            if i > 0 {
                                lw.print(",");
                            }
                            lw.print(&format!(
                                "#{}={}#{}",
                                ev.element_name_index,
                                ev.value.tag(),
                                match &ev.value {
                                    ElementValue::Byte { .. } => todo!(),
                                    ElementValue::Char { .. } => todo!(),
                                    ElementValue::Double { .. } => todo!(),
                                    ElementValue::Float { .. } => todo!(),
                                    ElementValue::Int { .. } => todo!(),
                                    ElementValue::Long { .. } => todo!(),
                                    ElementValue::Short { .. } => todo!(),
                                    ElementValue::Boolean { const_value_index } =>
                                        const_value_index,
                                    ElementValue::String { const_value_index } => const_value_index,
                                    ElementValue::Enum { .. } => todo!(),
                                    ElementValue::Class { .. } => todo!(),
                                    ElementValue::Annotation { .. } => todo!(),
                                    ElementValue::Array { .. } => todo!(),
                                }
                            ));
                        }
                    }
                    lw.println(")");
                    lw.indent(1);
                    lw.print(&annotation_type);
                    if has_values {
                        lw.println("(");
                        lw.indent(1);
                        for ev in annotation.element_value_pairs.iter() {
                            lw.println(&format!(
                                "{}={}",
                                cp.get_utf8_content(ev.element_name_index),
                                match &ev.value {
                                    ElementValue::Byte { .. } => todo!(),
                                    ElementValue::Char { .. } => todo!(),
                                    ElementValue::Double { .. } => todo!(),
                                    ElementValue::Float { .. } => todo!(),
                                    ElementValue::Int { .. } => todo!(),
                                    ElementValue::Long { .. } => todo!(),
                                    ElementValue::Short { .. } => todo!(),
                                    ElementValue::Boolean { const_value_index } =>
                                        (if cp.get_integer(*const_value_index) == 0 {
                                            "false"
                                        } else {
                                            "true"
                                        })
                                        .to_owned(),
                                    ElementValue::String { const_value_index } =>
                                        "\"".to_owned()
                                            + &cp.get_utf8_content(*const_value_index)
                                            + "\"",
                                    ElementValue::Enum { .. } => todo!(),
                                    ElementValue::Class { .. } => todo!(),
                                    ElementValue::Annotation { .. } => todo!(),
                                    ElementValue::Array { .. } => todo!(),
                                }
                            ));
                        }
                        lw.indent(-1);
                        lw.print(")");
                    }
                    lw.println("");
                    lw.indent(-1);
                }
                lw.indent(-1);
            }
            AttributeInfo::Exceptions {
                exception_indices, ..
            } => {
                lw.println("Exceptions:");
                lw.indent(1);
                for exception_index in exception_indices {
                    lw.println(&format!(
                        "throws {}",
                        cp.get_class_name(*exception_index).replace('/', ".")
                    ));
                }
                lw.indent(-1);
            }
            AttributeInfo::Deprecated { .. } => {
                lw.println("Deprecated: true");
            }
            _ => unreachable!("Unknown method attribute {}.", attribute.kind()),
        }
    }
}

fn print_code_attributes(lw: &mut LineWriter, cp: &ConstantPool, attributes: &[AttributeInfo]) {
    for attribute in attributes.iter() {
        match attribute {
            AttributeInfo::LineNumberTable {
                line_number_table, ..
            } => {
                lw.println("LineNumberTable:");
                lw.indent(1);
                for entry in line_number_table.iter() {
                    lw.println(&format!("line {}: {}", entry.line_number, entry.start_pc));
                }
                lw.indent(-1);
            }
            AttributeInfo::LocalVariableTable {
                local_variable_table,
                ..
            } => {
                lw.println("LocalVariableTable:");
                lw.indent(1);
                lw.println("Start  Length  Slot  Name   Signature");
                for entry in local_variable_table.iter() {
                    lw.println(&format!(
                        " {:4}    {:4}    {:2} {:>5}   {}",
                        entry.start_pc,
                        entry.length,
                        entry.index,
                        cp.get_utf8_content(entry.name_index),
                        cp.get_utf8_content(entry.descriptor_index)
                    ));
                }
                lw.indent(-1);
            }
            AttributeInfo::LocalVariableTypeTable {
                local_variable_type_table,
                ..
            } => {
                lw.println("LocalVariableTypeTable:");
                lw.indent(1);
                lw.println("Start  Length  Slot  Name   Signature");
                for entry in local_variable_type_table.iter() {
                    lw.println(&format!(
                        " {:4}    {:4}    {:2} {:>5}   {}",
                        entry.start_pc,
                        entry.length,
                        entry.index,
                        cp.get_utf8_content(entry.name_index),
                        cp.get_utf8_content(entry.descriptor_index)
                    ));
                }
                lw.indent(-1);
            }
            AttributeInfo::StackMapTable {
                stack_map_table, ..
            } => {
                lw.println(&format!(
                    "StackMapTable: number_of_entries = {}",
                    stack_map_table.len(),
                ));
                lw.indent(1);
                for frame in stack_map_table.iter() {
                    match frame {
                        StackMapFrame::SameFrame { frame_type } => {
                            lw.println(&format!("frame_type = {frame_type} /* same */"));
                        }
                        StackMapFrame::SameLocals1StackItemFrame { frame_type, stack } => {
                            lw.println(&format!(
                                "frame_type = {frame_type} /* same_locals_1_stack_item */"
                            ));
                            lw.indent(1);
                            lw.println(&format!(
                                "stack = [ {} ]",
                                get_verification_type_info_string(cp, stack)
                            ));
                            lw.indent(-1);
                        }
                        StackMapFrame::SameLocals1StackItemFrameExtended {
                            offset_delta,
                            stack,
                        } => {
                            lw.println(
                                "frame_type = 247 /* same_locals_1_stack_item_frame_extended */",
                            );
                            lw.indent(1);
                            lw.println(&format!("offset_delta = {offset_delta}"));
                            lw.println(&format!(
                                "stack = [ {} ]",
                                get_verification_type_info_string(cp, stack)
                            ));
                            lw.indent(-1);
                        }
                        StackMapFrame::ChopFrame {
                            frame_type,
                            offset_delta,
                        } => {
                            lw.println(&format!("frame_type = {frame_type} /* chop */"));
                            lw.indent(1);
                            lw.println(&format!("offset_delta = {offset_delta}"));
                            lw.indent(-1);
                        }
                        StackMapFrame::SameFrameExtended { offset_delta } => {
                            lw.println("frame_type = 251 /* same_frame_extended */");
                            lw.indent(1);
                            lw.println(&format!("offset_delta = {offset_delta}"));
                            lw.indent(-1);
                        }
                        StackMapFrame::AppendFrame {
                            frame_type,
                            offset_delta,
                            locals,
                        } => {
                            lw.println(&format!("frame_type = {frame_type} /* append */"));
                            lw.indent(1);
                            lw.println(&format!("offset_delta = {offset_delta}"));
                            lw.println(&format!(
                                "locals = {}",
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
                            ));
                            lw.indent(-1);
                        }
                        StackMapFrame::FullFrame {
                            offset_delta,
                            locals,
                            stack,
                        } => {
                            lw.println("frame_type = 255 /* full_frame */");
                            lw.indent(1);
                            lw.println(&format!("offset_delta = {offset_delta}"));
                            lw.println(&format!(
                                "locals = {}",
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
                            ));
                            lw.println(&format!(
                                "stack = {}",
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
                            ));
                            lw.indent(-1);
                        }
                    }
                }
                lw.indent(-1);
            }
            _ => unreachable!(),
        }
    }
}

fn print_class_attributes(lw: &mut LineWriter, cp: &ConstantPool, attributes: &[AttributeInfo]) {
    for attribute in attributes.iter() {
        match attribute {
            AttributeInfo::SourceFile {
                source_file_index, ..
            } => {
                lw.println(&format!(
                    "SourceFile: \"{}\"",
                    cp.get_utf8_content(*source_file_index)
                ));
            }
            AttributeInfo::InnerClasses { classes, .. } => {
                lw.println("InnerClasses:");
                lw.indent(1);
                for class in classes.iter() {
                    let modifiers = class.inner_class_access_flags.modifier_repr();

                    if class.is_anonymous() || class.is_local() {
                        if class.inner_class_access_flags.to_u16() != 0 {
                            lw.print(&format!("{modifiers} "));
                        }
                        lw.print(&format!("#{};", class.inner_class_info_index))
                            .tab()
                            .println(&format!(
                                "// class {}",
                                cp.get_class_name(class.inner_class_info_index)
                            ));
                    } else if class.is_member() {
                        lw.print(&format!(
                            "{} #{}= #{} of #{};",
                            modifiers,
                            class.inner_name_index,
                            class.inner_class_info_index,
                            class.outer_class_info_index
                        ))
                        .tab()
                        .println(&format!(
                            "// {}=class {} of class {}",
                            cp.get_utf8_content(class.inner_name_index),
                            cp.get_class_name(class.inner_class_info_index),
                            cp.get_class_name(class.outer_class_info_index),
                        ));
                    } else {
                        unreachable!();
                    }
                }
                lw.indent(-1);
            }
            AttributeInfo::BootstrapMethods { methods, .. } => {
                lw.println("BootstrapMethods:");
                lw.indent(1);
                for (i, method) in methods.iter().enumerate() {
                    lw.print(&format!("{}: #{} ", i, method.bootstrap_method_ref));

                    // TODO: can we merge this match-case with the one below?
                    match cp[method.bootstrap_method_ref] {
                        ConstantPoolInfo::MethodHandle {
                            reference_kind,
                            reference_index,
                        } => {
                            lw.println(&format!(
                                "{} {}",
                                reference_kind::java_repr(reference_kind),
                                cp.get_method_ref(reference_index)
                            ));
                        }
                        _ => unreachable!(),
                    }
                    lw.indent(1);
                    lw.println("Method arguments:");
                    for arg in method.bootstrap_arguments.iter() {
                        lw.print(&format!("  #{arg} "));
                        match cp[*arg] {
                            ConstantPoolInfo::String { string_index } => {
                                lw.println(&cp.get_utf8_content(string_index));
                            }
                            ConstantPoolInfo::Class { name_index } => {
                                lw.println(&cp.get_utf8_content(name_index));
                            }
                            ConstantPoolInfo::MethodType { descriptor_index } => {
                                lw.println(&cp.get_utf8_content(descriptor_index));
                            }
                            ConstantPoolInfo::MethodHandle {
                                reference_kind,
                                reference_index,
                            } => {
                                lw.println(&format!(
                                    "{} {}",
                                    reference_kind::java_repr(reference_kind),
                                    cp.get_method_ref(reference_index)
                                ));
                            }
                            _ => unreachable!(),
                        }
                    }
                    lw.indent(-1);
                }
                lw.indent(-1);
            }
            AttributeInfo::Record { components, .. } => {
                lw.println("Record:");
                lw.indent(1);
                for component in components.iter() {
                    let descriptor = cp.get_utf8_content(component.descriptor_index);
                    lw.println(&format!(
                        "{} {};",
                        decode_type(&descriptor),
                        cp.get_utf8_content(component.name_index)
                    ));
                    lw.println(&format!("  descriptor: {descriptor}"));
                    lw.println("");
                }
                lw.indent(-1);
            }
            AttributeInfo::Signature {
                signature_index, ..
            } => {
                lw.print(&format!("Signature: #{signature_index}"))
                    .tab()
                    .println(&format!("// {}", cp.get_utf8_content(*signature_index)));
            }
            AttributeInfo::NestMembers { classes, .. } => {
                lw.println("NestMembers:");
                for class_index in classes {
                    lw.println(&format!("  {}", cp.get_class_name(*class_index)));
                }
            }
            AttributeInfo::EnclosingMethod {
                class_index,
                method_index,
                ..
            } => {
                lw.print("EnclosingMethod: ")
                    .print(&format!("#{class_index}.#{method_index}"))
                    .tab()
                    .print("// ")
                    .println(&cp.get_class_name(*class_index).replace('/', "."));
            }
            AttributeInfo::NestHost {
                host_class_index, ..
            } => {
                lw.println(&format!(
                    "NestHost: class {}",
                    cp.get_class_name(*host_class_index)
                ));
            }
            _ => unreachable!(),
        }
    }
}

/**
 * Formats the given f64 as Java's default format.
 */
fn java_format_double(val: f64) -> String {
    if val.is_nan() {
        "NaN".to_owned()
    } else if val.is_infinite() {
        if val > 0.0 {
            "Infinity".to_owned()
        } else {
            "-Infinity".to_owned()
        }
    } else if val == 0.0 {
        // both +0.0 and -0.0 compare equal to 0.0, so check the sign bit
        if val.is_sign_negative() {
            "-0.0".to_owned()
        } else {
            "0.0".to_owned()
        }
    } else if val.abs() == 5e-324 {
        // The smallest denormal is hard-coded because formatted incorrectly
        if val > 0.0 {
            "4.9E-324".to_owned()
        } else {
            "-4.9E-324".to_owned()
        }
    } else if val.abs() >= 1.0e-3 && val.abs() < 1.0e7 {
        format!("{val:?}")
    } else {
        format_scientific_f64(val)
    }
}

fn format_scientific_f64(val: f64) -> String {
    let raw = format!("{val:E}");

    let e_pos = raw.find('E').unwrap();
    let mantissa = &raw[..e_pos];
    let exp: i32 = raw[e_pos + 1..].parse().unwrap();

    // Trim trailing zeros but ensure at least one decimal digit
    let trimmed = if mantissa.find('.').is_some() {
        let trimmed = mantissa.trim_end_matches('0');
        if trimmed.ends_with('.') {
            format!("{trimmed}0")
        } else {
            trimmed.to_owned()
        }
    } else {
        // No decimal point at all (e.g. "1E8") — add ".0"
        format!("{mantissa}.0")
    };

    format!("{trimmed}E{exp}")
}

/**
 * Formats the given f32 as Java's default format.
 */
fn java_format_float(val: f32) -> String {
    if val.is_nan() {
        "NaN".to_owned()
    } else if val.is_infinite() {
        if val > 0.0f32 {
            "Infinity".to_owned()
        } else {
            "-Infinity".to_owned()
        }
    } else if val == 0.0f32 {
        // both +0.0 and -0.0 compare equal to 0.0, so check the sign bit
        if val.is_sign_negative() {
            "-0.0".to_owned()
        } else {
            "0.0".to_owned()
        }
    } else if val.abs() == 1.4E-45f32 {
        // The smallest denormal is hard-coded because formatted incorrectly
        if val > 0.0f32 {
            "1.4E-45".to_owned()
        } else {
            "-1.4E-45".to_owned()
        }
    } else {
        let debug = format!("{val:?}");
        if val.abs() >= 1.0e-3f32 && val.abs() < 1.0e7f32 {
            debug
        } else {
            format_scientific_f32(val)
        }
    }
}

fn format_scientific_f32(val: f32) -> String {
    // Format as f32 to get shortest round-trip digits for f32
    let raw = format!("{val:E}");

    let e_pos = raw.find('E').unwrap();
    let mantissa = &raw[..e_pos];
    let exp: i32 = raw[e_pos + 1..].parse().unwrap();

    let trimmed = if mantissa.find('.').is_some() {
        let trimmed = mantissa.trim_end_matches('0');
        if trimmed.ends_with('.') {
            format!("{trimmed}0")
        } else {
            trimmed.to_owned()
        }
    } else {
        format!("{mantissa}.0")
    };

    format!("{trimmed}E{exp}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn double_formatting() {
        let cases = [
            (0.0, "0.0"),
            (-0.0, "-0.0"),
            (f64::INFINITY, "Infinity"),
            (-f64::INFINITY, "-Infinity"),
            (f64::NAN, "NaN"),
            (-f64::NAN, "NaN"),
            (0.017453292519943295, "0.017453292519943295"),
            (4.9E-324, "4.9E-324"),
            (8.98846567431158E307, "8.98846567431158E307"),
            (1.1125369292536007E-308, "1.1125369292536007E-308"),
            (100000000.0, "1.0E8"),
            (10000000.0, "1.0E7"),
            (9999999.999999999, "9999999.999999998"),
            (1000000.0, "1000000.0"),
            (100000.0, "100000.0"),
            (10000.0, "10000.0"),
            (1000.0, "1000.0"),
            (100.0, "100.0"),
            (10.0, "10.0"),
            (1.0, "1.0"),
            (0.1, "0.1"),
            (0.01, "0.01"),
            (0.001, "0.001"),
            (0.0009999999999999999, "9.999999999999998E-4"),
            (0.0001, "1.0E-4"),
            (0.00001, "1.0E-5"),
        ];

        for (input, expected) in cases {
            let actual = java_format_double(input);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn float_formatting() {
        let cases = [
            (0.0f32, "0.0"),
            (-0.0f32, "-0.0"),
            (f32::INFINITY, "Infinity"),
            (-f32::INFINITY, "-Infinity"),
            (f32::NAN, "NaN"),
            (-f32::NAN, "NaN"),
            (1.4E-45f32, "1.4E-45"),
            (100000000.0f32, "1.0E8"),
            (10000000.0f32, "1.0E7"),
            (9999999.0f32, "9999999.0"),
            (1000000.0f32, "1000000.0"),
            (100000.0f32, "100000.0"),
            (10000.0f32, "10000.0"),
            (1000.0f32, "1000.0"),
            (100.0f32, "100.0"),
            (10.0f32, "10.0"),
            (1.0f32, "1.0"),
            (0.1f32, "0.1"),
            (0.01f32, "0.01"),
            (0.001f32, "0.001"),
            (0.0009999999f32, "9.999999E-4"),
            (0.0001f32, "1.0E-4"),
            (0.00001f32, "1.0E-5"),
        ];

        for (input, expected) in cases {
            let actual = java_format_float(input);
            assert_eq!(expected, actual);
        }
    }
}
