use binary_reader::byte_reader::ByteReader;

use crate::{
    access_flags::MethodAccessFlags,
    assert_valid_and_type,
    attributes::{AttributeInfo, parse_method_attributes},
    constant_pool::{ConstantPool, ConstantPoolTag},
};

#[derive(Clone)]
pub struct MethodInfo {
    pub access_flags: MethodAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16, // TODO: maybe refactor into list of parsed arguments?
    pub attributes: Vec<AttributeInfo>,
}

pub fn parse_methods(
    reader: &mut ByteReader,
    cp: &ConstantPool,
    num_methods: usize,
) -> Vec<MethodInfo> {
    let mut methods: Vec<MethodInfo> = Vec::with_capacity(num_methods);
    for _ in 0..num_methods {
        let access_flags: MethodAccessFlags = MethodAccessFlags::from(reader.read_u16().unwrap());
        let name_index: u16 = reader.read_u16().unwrap();
        assert_valid_and_type!(cp, name_index, ConstantPoolTag::Utf8);
        let descriptor_index: u16 = reader.read_u16().unwrap();
        assert_valid_and_type!(cp, descriptor_index, ConstantPoolTag::Utf8);
        let attribute_count: u16 = reader.read_u16().unwrap();
        let attributes: Vec<AttributeInfo> =
            parse_method_attributes(reader, cp, attribute_count.into());
        methods.push(MethodInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        });
    }
    methods
}
