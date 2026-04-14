#![forbid(unsafe_code)]

use binary_writer::{BinaryWriter, Endianness};

use crate::{
    access_flags, attributes::AttributeInfo, bytecode::write_instruction, classfile::ClassFile, constant_pool::{ConstantPool, ConstantPoolInfo}, fields::FieldInfo, methods::MethodInfo
};

pub fn write_class_file(cf: &ClassFile) -> Vec<u8> {
    let mut w: BinaryWriter = BinaryWriter::new(Endianness::Big);

    w.write_u32(0xcafebabe);
    w.write_u16(cf.minor_version);
    w.write_u16(cf.major_version);

    w.write_u16((cf.constant_pool.len() + 1).try_into().unwrap());
    write_constant_pool(&mut w, &cf.constant_pool);

    w.write_u16(access_flags::to_u16(&cf.access_flags));

    w.write_u16(cf.this_class);
    w.write_u16(cf.super_class);

    w.write_u16(cf.interfaces.len().try_into().unwrap());
    w.write_u16_vec(&cf.interfaces);

    w.write_u16(cf.fields.len().try_into().unwrap());
    write_fields(&mut w, &cf.fields);

    w.write_u16(cf.methods.len().try_into().unwrap());
    write_methods(&mut w, &cf.methods);

    w.write_u16(cf.attributes.len().try_into().unwrap());
    write_attributes(&mut w, &cf.attributes);

    w.array()
}

fn write_constant_pool(w: &mut BinaryWriter, cp: &ConstantPool) {
    for entry in cp.entries.iter() {
        if matches!(entry, ConstantPoolInfo::Null {}) {
            continue;
        }

        w.write_u8(entry.tag().into());
        match entry {
            ConstantPoolInfo::Null {} => {}
            ConstantPoolInfo::Utf8 { bytes } => {
                w.write_u16(bytes.len().try_into().unwrap());
                w.write_u8_vec(bytes);
            }
            ConstantPoolInfo::Integer { bytes } => {
                w.write_u32(*bytes);
            }
            ConstantPoolInfo::Float { bytes } => {
                w.write_u32(*bytes);
            }
            ConstantPoolInfo::Long {
                high_bytes,
                low_bytes,
            } => {
                w.write_u32(*high_bytes);
                w.write_u32(*low_bytes);
            }
            ConstantPoolInfo::Double {
                high_bytes,
                low_bytes,
            } => {
                w.write_u32(*high_bytes);
                w.write_u32(*low_bytes);
            }
            ConstantPoolInfo::String { string_index } => {
                w.write_u16(*string_index);
            }
            ConstantPoolInfo::Class { name_index } => {
                w.write_u16(*name_index);
            }
            ConstantPoolInfo::FieldRef {
                class_index,
                name_and_type_index,
            } => {
                w.write_u16(*class_index);
                w.write_u16(*name_and_type_index);
            }
            ConstantPoolInfo::MethodRef {
                class_index,
                name_and_type_index,
            } => {
                w.write_u16(*class_index);
                w.write_u16(*name_and_type_index);
            }
            ConstantPoolInfo::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => todo!(),
            ConstantPoolInfo::NameAndType {
                name_index,
                descriptor_index,
            } => {
                w.write_u16(*name_index);
                w.write_u16(*descriptor_index);
            }
            ConstantPoolInfo::MethodType { descriptor_index } => todo!(),
            ConstantPoolInfo::MethodHandle {
                reference_kind,
                reference_index,
            } => todo!(),
            ConstantPoolInfo::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => todo!(),
        }
    }
}

fn write_fields(w: &mut BinaryWriter, fields: &[FieldInfo]) {
    for field in fields.iter() {
        w.write_u16(access_flags::to_u16(&field.access_flags));
        w.write_u16(field.name_index);
        w.write_u16(field.descriptor_index);
        w.write_u16(field.attributes.len().try_into().unwrap());
        write_attributes(w, &field.attributes);
    }
}

fn write_methods(w: &mut BinaryWriter, methods: &[MethodInfo]) {
    for method in methods.iter() {
        w.write_u16(access_flags::to_u16(&method.access_flags));
        w.write_u16(method.name_index);
        w.write_u16(method.descriptor_index);
        w.write_u16(method.attributes.len().try_into().unwrap());
        write_attributes(w, &method.attributes);
    }
}

fn write_attributes(w: &mut BinaryWriter, attributes: &[AttributeInfo]) {
    for attribute in attributes.iter() {
        match attribute {
            AttributeInfo::Code {
                max_stack,
                max_locals,
                code,
                exception_table,
                attributes,
            } => {
                w.write_u16(*max_stack);
                w.write_u16(*max_locals);
                w.write_u32(code.len().try_into().unwrap());
                for (index, instruction) in code.iter() {
                    write_instruction(w, instruction);
                }
                w.write_u16(exception_table.len().try_into().unwrap());
                for exception in exception_table.iter() {
                    w.write_u16(exception.start_pc);
                    w.write_u16(exception.end_pc);
                    w.write_u16(exception.handler_pc);
                    w.write_u16(exception.catch_type);
                }
                w.write_u16(attributes.len().try_into().unwrap());
                write_attributes(w, attributes);
            }
            AttributeInfo::LineNumberTable { line_number_table } => todo!(),
            AttributeInfo::LocalVariableTable {
                local_variable_table,
            } => todo!(),
            AttributeInfo::LocalVariableTypeTable {
                local_variable_type_table,
            } => todo!(),
            AttributeInfo::StackMapTable { stack_map_table } => todo!(),
            AttributeInfo::SourceFile { source_file_index } => todo!(),
            AttributeInfo::BootstrapMethods { methods } => todo!(),
            AttributeInfo::InnerClasses { classes } => todo!(),
            AttributeInfo::MethodParameters { parameters } => todo!(),
            AttributeInfo::Record { components } => todo!(),
            AttributeInfo::Signature { signature_index } => todo!(),
            AttributeInfo::NestMembers { classes } => todo!(),
            AttributeInfo::RuntimeVisibleAnnotations { annotations } => todo!(),
            AttributeInfo::ConstantValue {
                constant_value_index,
            } => todo!(),
        }
    }
}
