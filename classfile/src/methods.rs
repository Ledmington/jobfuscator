use binary_reader::BinaryReader;

use crate::{
    access_flags::{self, AccessFlag},
    attributes::{AttributeInfo, parse_attributes},
    constant_pool::ConstantPool,
};

pub struct MethodInfo {
    pub access_flags: Vec<AccessFlag>,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>,
}

pub fn parse_methods(
    reader: &mut BinaryReader,
    cp: &ConstantPool,
    num_methods: usize,
) -> Vec<MethodInfo> {
    let mut methods: Vec<MethodInfo> = Vec::with_capacity(num_methods);
    for _ in 0..num_methods {
        let access_flags: Vec<AccessFlag> =
            access_flags::parse_access_flags(reader.read_u16().unwrap());
        let name_index: u16 = reader.read_u16().unwrap();
        let descriptor_index: u16 = reader.read_u16().unwrap();
        let attribute_count: u16 = reader.read_u16().unwrap();
        let attributes: Vec<AttributeInfo> = parse_attributes(reader, cp, attribute_count.into());
        methods.push(MethodInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        });
    }
    methods
}
