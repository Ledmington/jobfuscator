#![forbid(unsafe_code)]

use binary_writer::{BinaryWriter, Endianness};

use crate::{
    access_flags,
    attributes::{Annotation, AttributeInfo, ElementValue},
    bytecode::{get_instruction_length, write_instruction},
    classfile::ClassFile,
    constant_pool::{ConstantPool, ConstantPoolInfo},
    fields::FieldInfo,
    methods::MethodInfo,
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
            } => {
                w.write_u16(*class_index);
                w.write_u16(*name_and_type_index);
            }
            ConstantPoolInfo::NameAndType {
                name_index,
                descriptor_index,
            } => {
                w.write_u16(*name_index);
                w.write_u16(*descriptor_index);
            }
            ConstantPoolInfo::MethodType { descriptor_index } => {
                w.write_u16(*descriptor_index);
            }
            ConstantPoolInfo::MethodHandle { .. } => todo!(),
            ConstantPoolInfo::InvokeDynamic { .. } => todo!(),
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

/**
 * Computes the total number of bytes required to write the given attribute, excluding the initial six bytes for the name_index field and the attribute_length field.
 */
fn get_attribute_length(attribute: &AttributeInfo) -> u32 {
    match attribute {
        AttributeInfo::Code {
            code,
            exception_table,
            attributes,
            ..
        } => {
            2 + 2
                + 4
                + code.values().map(get_instruction_length).sum::<u32>()
                + 2
                + (2 * 4) * (exception_table.len() as u32)
                + 2
                + attributes.iter().map(get_attribute_length).sum::<u32>()
        }
        AttributeInfo::LineNumberTable {
            line_number_table, ..
        } => 2 + (2 * 2) * (line_number_table.len() as u32),
        AttributeInfo::LocalVariableTable {
            local_variable_table,
            ..
        } => 2 + (2 * 5) * (local_variable_table.len() as u32),
        AttributeInfo::LocalVariableTypeTable { .. } => todo!(),
        AttributeInfo::StackMapTable { .. } => todo!(),
        AttributeInfo::SourceFile { .. } => 2,
        AttributeInfo::BootstrapMethods { .. } => todo!(),
        AttributeInfo::InnerClasses { .. } => todo!(),
        AttributeInfo::MethodParameters { .. } => todo!(),
        AttributeInfo::Record { .. } => todo!(),
        AttributeInfo::Signature { .. } => 2,
        AttributeInfo::NestMembers { classes, .. } => 2 + 2 * (classes.len() as u32),
        AttributeInfo::RuntimeVisibleAnnotations { annotations, .. } => {
            2 + annotations.iter().map(get_annotation_length).sum::<u32>()
        }
        AttributeInfo::ConstantValue { .. } => 2,
    }
}

fn get_annotation_length(annotation: &Annotation) -> u32 {
    2 + 2
        + annotation
            .element_value_pairs
            .iter()
            .map(|evp| 2 + get_element_value_length(&evp.value))
            .sum::<u32>()
}

fn get_element_value_length(value: &ElementValue) -> u32 {
    1 + match value {
        ElementValue::Byte { .. } => 2,
        ElementValue::Char { .. } => 2,
        ElementValue::Double { .. } => 2,
        ElementValue::Float { .. } => 2,
        ElementValue::Int { .. } => 2,
        ElementValue::Long { .. } => 2,
        ElementValue::Short { .. } => 2,
        ElementValue::Boolean { .. } => 2,
        ElementValue::String { .. } => 2,
        ElementValue::Enum { .. } => 4,
        ElementValue::Class { .. } => 2,
        ElementValue::Annotation { value } => get_annotation_length(value),
        ElementValue::Array { values } => {
            2 + values.iter().map(get_element_value_length).sum::<u32>()
        }
    }
}

fn write_attributes(w: &mut BinaryWriter, attributes: &[AttributeInfo]) {
    for attribute in attributes.iter() {
        match attribute {
            AttributeInfo::Code {
                name_index,
                max_stack,
                max_locals,
                code,
                exception_table,
                attributes,
            } => {
                w.write_u16(*name_index);
                w.write_u32(get_attribute_length(attribute));
                w.write_u16(*max_stack);
                w.write_u16(*max_locals);
                w.write_u32(code.values().map(get_instruction_length).sum());
                for (_, instruction) in code.iter() {
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
            AttributeInfo::LineNumberTable {
                name_index,
                line_number_table,
            } => {
                w.write_u16(*name_index);
                w.write_u32(get_attribute_length(attribute));
                w.write_u16(line_number_table.len().try_into().unwrap());
                for entry in line_number_table.iter() {
                    w.write_u16(entry.start_pc);
                    w.write_u16(entry.line_number);
                }
            }
            AttributeInfo::LocalVariableTable {
                name_index,
                local_variable_table,
            } => {
                w.write_u16(*name_index);
                w.write_u32(get_attribute_length(attribute));
                w.write_u16(local_variable_table.len().try_into().unwrap());
                for entry in local_variable_table.iter() {
                    w.write_u16(entry.start_pc);
                    w.write_u16(entry.length);
                    w.write_u16(entry.name_index);
                    w.write_u16(entry.descriptor_index);
                    w.write_u16(entry.index);
                }
            }
            AttributeInfo::LocalVariableTypeTable { .. } => todo!(),
            AttributeInfo::StackMapTable { .. } => todo!(),
            AttributeInfo::SourceFile {
                name_index,
                source_file_index,
            } => {
                w.write_u16(*name_index);
                w.write_u32(get_attribute_length(attribute));
                w.write_u16(*source_file_index);
            }
            AttributeInfo::BootstrapMethods { .. } => todo!(),
            AttributeInfo::InnerClasses { .. } => todo!(),
            AttributeInfo::MethodParameters { .. } => todo!(),
            AttributeInfo::Record { .. } => todo!(),
            AttributeInfo::Signature {
                name_index,
                signature_index,
            } => {
                w.write_u16(*name_index);
                w.write_u32(get_attribute_length(attribute));
                w.write_u16(*signature_index);
            }
            AttributeInfo::NestMembers {
                name_index,
                classes,
            } => {
                w.write_u16(*name_index);
                w.write_u32(get_attribute_length(attribute));
                w.write_u16(classes.len().try_into().unwrap());
                w.write_u16_vec(classes);
            }
            AttributeInfo::RuntimeVisibleAnnotations {
                name_index,
                annotations,
            } => {
                w.write_u16(*name_index);
                w.write_u32(get_attribute_length(attribute));
                write_annotations(w, annotations);
            }
            AttributeInfo::ConstantValue {
                name_index,
                constant_value_index,
            } => {
                w.write_u16(*name_index);
                w.write_u32(get_attribute_length(attribute));
                w.write_u16(*constant_value_index);
            }
        }
    }
}

fn write_annotations(w: &mut BinaryWriter, annotations: &[Annotation]) {
    w.write_u16(annotations.len().try_into().unwrap());
    for annotation in annotations.iter() {
        write_annotation(w, annotation);
    }
}

fn write_annotation(w: &mut BinaryWriter, annotation: &Annotation) {
    w.write_u16(annotation.type_index);
    w.write_u16(annotation.element_value_pairs.len().try_into().unwrap());
    for evp in annotation.element_value_pairs.iter() {
        w.write_u16(evp.element_name_index);
        write_element_value(w, &evp.value);
    }
}

fn write_element_value(w: &mut BinaryWriter, value: &ElementValue) {
    w.write_u8(value.tag() as u8);
    match &value {
        crate::attributes::ElementValue::Byte { const_value_index } => {
            w.write_u16(*const_value_index)
        }
        crate::attributes::ElementValue::Char { const_value_index } => {
            w.write_u16(*const_value_index)
        }
        crate::attributes::ElementValue::Double { const_value_index } => {
            w.write_u16(*const_value_index)
        }
        crate::attributes::ElementValue::Float { const_value_index } => {
            w.write_u16(*const_value_index)
        }
        crate::attributes::ElementValue::Int { const_value_index } => {
            w.write_u16(*const_value_index)
        }
        crate::attributes::ElementValue::Long { const_value_index } => {
            w.write_u16(*const_value_index)
        }
        crate::attributes::ElementValue::Short { const_value_index } => {
            w.write_u16(*const_value_index)
        }
        crate::attributes::ElementValue::Boolean { const_value_index } => {
            w.write_u16(*const_value_index)
        }
        crate::attributes::ElementValue::String { const_value_index } => {
            w.write_u16(*const_value_index)
        }
        crate::attributes::ElementValue::Enum {
            type_name_index,
            const_name_index,
        } => {
            w.write_u16(*type_name_index);
            w.write_u16(*const_name_index);
        }
        crate::attributes::ElementValue::Class { class_info_index } => {
            w.write_u16(*class_info_index)
        }
        crate::attributes::ElementValue::Annotation { value } => write_annotation(w, value),
        crate::attributes::ElementValue::Array { values } => {
            w.write_u16(values.len().try_into().unwrap());
            for ev in values.iter() {
                write_element_value(w, ev);
            }
        }
    }
}
