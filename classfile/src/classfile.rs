#![forbid(unsafe_code)]

use binary_reader::BinaryReader;

use crate::access_flags::{ClassAccessFlag, parse_class_access_flags};
use crate::attributes::{AttributeInfo, parse_class_attributes};
use crate::constant_pool::{
    ConstantPool, ConstantPoolTag, assert_valid_and_type, parse_constant_pool,
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
        "Wrong magic number: expected 0x{:08x} but was 0x{:08x}.",
        EXPECTED_MAGIC_NUMBER,
        actual_magic_number
    );

    let minor_version: u16 = reader.read_u16().unwrap();
    let major_version: u16 = reader.read_u16().unwrap();
    {
        // class file format version of java 1.0
        const OLDEST_MAJOR_VERSION: u16 = 44;
        assert!(
            major_version >= OLDEST_MAJOR_VERSION,
            "Invalid class file version {},{} (0x{:04x}.{:04x})",
            major_version,
            minor_version,
            major_version,
            minor_version
        );

        // class file format version of java 25
        const LATEST_MAJOR_VERSION: u16 = 69;
        assert!(
            major_version <= LATEST_MAJOR_VERSION,
            "Class file version {}.{} is greater than {}.0",
            major_version,
            minor_version,
            LATEST_MAJOR_VERSION
        );
    }

    let cp_count: u16 = reader.read_u16().unwrap();
    let constant_pool: ConstantPool = parse_constant_pool(reader, (cp_count - 1).into());

    let access_flags: Vec<ClassAccessFlag> = parse_class_access_flags(reader.read_u16().unwrap());

    let this_class: u16 = reader.read_u16().unwrap();
    assert_valid_and_type(&constant_pool, this_class, ConstantPoolTag::Class);

    let super_class: u16 = reader.read_u16().unwrap();
    assert_valid_and_type(&constant_pool, super_class, ConstantPoolTag::Class);

    let interfaces_count: u16 = reader.read_u16().unwrap();
    let interfaces: Vec<u16> = reader.read_u16_vec(interfaces_count.into()).unwrap();

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
