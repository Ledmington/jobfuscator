use binary_reader::BinaryReader;

use crate::constant_pool::ConstantPool;

pub struct AttributeInfo {}

pub fn parse_attributes(
    reader: &mut BinaryReader,
    cp: &ConstantPool,
    num_attributes: usize,
) -> Vec<AttributeInfo> {
    let mut attributes: Vec<AttributeInfo> = Vec::with_capacity(num_attributes);
    for _ in 0..num_attributes {
        attributes.push(parse_attribute(reader, cp));
    }
    attributes
}

fn parse_attribute(reader: &mut BinaryReader, cp: &ConstantPool) -> AttributeInfo {
    let attribute_name_index: u16 = reader.read_u16().unwrap();
    let attribute_name: String = cp.get_utf8_content(attribute_name_index);
    match attribute_name {
        _ => panic!("Unknown attribute name {}.", attribute_name),
    }
}
