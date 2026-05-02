use binary_reader::BinaryReader;

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
    reader: &mut BinaryReader,
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

        fields.push(FieldInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        });
    }
    fields
}
