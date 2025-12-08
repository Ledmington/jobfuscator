use binary_reader::BinaryReader;

use crate::{
    AttributeInfo,
    access_flags::{self, AccessFlag},
    parse_attributes,
};

pub struct FieldInfo {
    access_flags: Vec<AccessFlag>,
    name_index: u16,
    descriptor_index: u16,
    attributes: Vec<AttributeInfo>,
}

pub fn parse_fields(reader: &mut BinaryReader, num_fields: usize) -> Vec<FieldInfo> {
    let mut fields: Vec<FieldInfo> = Vec::with_capacity(num_fields);
    for i in 0..num_fields {
        let access_flags = access_flags::parse_access_flags(reader.read_u16().unwrap());
        let name_index = reader.read_u16().unwrap();
        let descriptor_index = reader.read_u16().unwrap();
        let attributes_count: u16 = reader.read_u16().unwrap();
        let attributes: Vec<AttributeInfo> = parse_attributes(reader, attributes_count.into());
        fields[i] = FieldInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        };
    }
    return fields;
}
