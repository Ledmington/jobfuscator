use binary_reader::BinaryReader;

use crate::{
    AttributeInfo,
    access_flags::{self, AccessFlag},
    attributes::parse_attributes,
    constant_pool::ConstantPool,
};

pub struct FieldInfo {
    access_flags: Vec<AccessFlag>,
    name_index: u16,
    descriptor_index: u16,
    attributes: Vec<AttributeInfo>,
}

pub fn parse_fields(
    reader: &mut BinaryReader,
    cp: &ConstantPool,
    num_fields: usize,
) -> Vec<FieldInfo> {
    let mut fields: Vec<FieldInfo> = Vec::with_capacity(num_fields);
    for _ in 0..num_fields {
        let access_flags = access_flags::parse_access_flags(reader.read_u16().unwrap());
        let name_index = reader.read_u16().unwrap();
        let descriptor_index = reader.read_u16().unwrap();
        let attributes_count: u16 = reader.read_u16().unwrap();
        let attributes: Vec<AttributeInfo> = parse_attributes(reader, cp, attributes_count.into());
        fields.push(FieldInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        });
    }
    return fields;
}
