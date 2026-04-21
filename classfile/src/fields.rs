
use binary_reader::byte_reader::ByteReader;

use crate::{
    access_flags::FieldAccessFlags,
    assert_valid_and_type,
    attributes::{AttributeInfo, parse_field_attributes},
    constant_pool::{ConstantPool, ConstantPoolTag},
};

#[derive(Clone)]
pub struct FieldInfo {
    pub access_flags: FieldAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>,
}

pub fn parse_fields(
    reader: &mut ByteReader,
    cp: &ConstantPool,
    num_fields: usize,
) -> Vec<FieldInfo> {
    let mut fields: Vec<FieldInfo> = Vec::with_capacity(num_fields);
    for _ in 0..num_fields {
        let access_flags: FieldAccessFlags = FieldAccessFlags::from(reader.read_u16().unwrap());
        let name_index: u16 = reader.read_u16().unwrap();
        assert_valid_and_type!(cp, name_index, ConstantPoolTag::Utf8);
        let descriptor_index: u16 = reader.read_u16().unwrap();
        assert_valid_and_type!(cp, descriptor_index, ConstantPoolTag::Utf8);
        let attributes_count: u16 = reader.read_u16().unwrap();
        let attributes: Vec<AttributeInfo> =
            parse_field_attributes(reader, cp, attributes_count.into());

        /*let constant_value = find_attribute(&attributes, AttributeKind::ConstantValue);
        if let Some(AttributeInfo::ConstantValue {
            constant_value_index,
            ..
        }) = constant_value
        {
            let field_descriptor = parse_field_descriptor(&cp.get_utf8_content(descriptor_index));
            match field_descriptor.field_type {
                Type::Boolean | Type::Char | Type::Byte | Type::Short | Type::Int => {
                    assert_valid_and_type!(cp, *constant_value_index, ConstantPoolTag::Integer)
                }
                Type::Long => {
                    assert_valid_and_type!(cp, *constant_value_index, ConstantPoolTag::Long)
                }
                Type::Float => {
                    assert_valid_and_type!(cp, *constant_value_index, ConstantPoolTag::Float)
                }
                Type::Double => {
                    assert_valid_and_type!(cp, *constant_value_index, ConstantPoolTag::Double)
                }
                Type::Object { class_name } if class_name == "java/lang/String" => {
                    assert_valid_and_type!(cp, *constant_value_index, ConstantPoolTag::String);
                }
                _ => {}
            }
        }*/

        fields.push(FieldInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        });
    }
    fields
}
