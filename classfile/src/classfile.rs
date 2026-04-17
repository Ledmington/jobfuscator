#![forbid(unsafe_code)]

use binary_reader::BinaryReader;

use crate::access_flags::{ClassAccessFlag, parse_class_access_flags};
use crate::attributes::{AttributeInfo, parse_class_attributes};
use crate::constant_pool::{
    ConstantPool, ConstantPoolTag, assert_valid_and_type, check_constant_pool, parse_constant_pool,
};
use crate::fields::{FieldInfo, parse_fields};
use crate::methods::{MethodInfo, parse_methods};

/**
 * Specification available at <https://docs.oracle.com/javase/specs/jvms/se25/html/jvms-4.html>
 */
pub struct ClassFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: Vec<ClassAccessFlag>,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttributeInfo>,
}

pub fn parse_class_file(reader: &mut BinaryReader) -> ClassFile {
    let actual_magic_number: u32 = reader.read_u32().unwrap();
    const EXPECTED_MAGIC_NUMBER: u32 = 0xcafebabe;
    assert!(
        actual_magic_number == EXPECTED_MAGIC_NUMBER,
        "Wrong magic number: expected 0x{EXPECTED_MAGIC_NUMBER:08x} but was 0x{actual_magic_number:08x}."
    );

    let minor_version: u16 = reader.read_u16().unwrap();
    let major_version: u16 = reader.read_u16().unwrap();
    {
        // class file format version of java 1.0
        const OLDEST_MAJOR_VERSION: u16 = 45;
        assert!(
            major_version >= OLDEST_MAJOR_VERSION,
            "Invalid class file version {major_version}.{minor_version} (0x{major_version:04x}.{minor_version:04x}) is older than the oldest supported java class file version {OLDEST_MAJOR_VERSION}.0.",
        );

        // class file format version of java 25
        const LATEST_MAJOR_VERSION: u16 = 69;
        assert!(
            major_version <= LATEST_MAJOR_VERSION,
            "Class file version {major_version}.{minor_version} is greater than the latest supported version {LATEST_MAJOR_VERSION}.0.",
        );
    }

    let cp_count: u16 = reader.read_u16().unwrap();
    let constant_pool: ConstantPool = parse_constant_pool(reader, (cp_count - 1).into());
    check_constant_pool(&constant_pool);

    let access_flags: Vec<ClassAccessFlag> = parse_class_access_flags(reader.read_u16().unwrap());

    let this_class: u16 = reader.read_u16().unwrap();
    assert_valid_and_type(&constant_pool, this_class, ConstantPoolTag::Class);

    let super_class: u16 = reader.read_u16().unwrap();
    assert_valid_and_type(&constant_pool, super_class, ConstantPoolTag::Class);

    let interfaces_count: u16 = reader.read_u16().unwrap();
    let interfaces: Vec<u16> = reader.read_u16_vec(interfaces_count.into()).unwrap();
    for interface_idx in interfaces.iter() {
        assert_valid_and_type(&constant_pool, *interface_idx, ConstantPoolTag::Class);
    }

    let fields_count: u16 = reader.read_u16().unwrap();
    let fields: Vec<FieldInfo> = parse_fields(reader, &constant_pool, fields_count.into());

    let methods_count: u16 = reader.read_u16().unwrap();
    let methods: Vec<MethodInfo> = parse_methods(reader, &constant_pool, methods_count.into());

    let attributes_count: u16 = reader.read_u16().unwrap();
    let attributes: Vec<AttributeInfo> =
        parse_class_attributes(reader, &constant_pool, attributes_count.into());

    ClassFile {
        minor_version,
        major_version,
        constant_pool,
        access_flags,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
    }
}
