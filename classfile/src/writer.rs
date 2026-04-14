#![forbid(unsafe_code)]

use binary_writer::{BinaryWriter, Endianness};

use crate::{
    access_flags,
    attributes::AttributeInfo,
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

fn write_fields(w: &mut BinaryWriter, fields: &Vec<FieldInfo>) {}

fn write_methods(w: &mut BinaryWriter, methods: &Vec<MethodInfo>) {}

fn write_attributes(w: &mut BinaryWriter, attributes: &Vec<AttributeInfo>) {}
