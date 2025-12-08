pub mod access_flags;
pub mod constant_pool;
pub mod fields;
pub mod reference_kind;

use std::env;
use std::fs::File;
use std::io::{BufReader, Read, Result};
use std::path::{Path, PathBuf};

use binary_reader::{BinaryReader, Endian};
use sha2::{Digest, Sha256};

use crate::access_flags::AccessFlag;
use crate::access_flags::parse_access_flags;
use crate::constant_pool::{ConstantPoolInfo, ConstantPoolTag};
use crate::fields::{FieldInfo, parse_fields};
use crate::reference_kind::ReferenceKind;

/**
 * Specification available at <https://docs.oracle.com/javase/specs/jvms/se25/html/jvms-4.html>
 */
pub struct ClassFile {
    pub absolute_file_path: String,
    pub sha256_digest: Vec<u8>,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<ConstantPoolInfo>,
    pub access_flags: Vec<AccessFlag>,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttributeInfo>,
}

impl ClassFile {
    pub fn get_class_name(&self, cp_index: u16) -> String {
        let class_entry: &ConstantPoolInfo = &self.constant_pool[(cp_index - 1) as usize];
        match class_entry {
            ConstantPoolInfo::Class { name_index } => self.get_utf8_content(*name_index),
            _ => panic!(
                "Expected entry #{} to be of Class type but it wasn't.",
                cp_index
            ),
        }
    }

    pub fn get_method_ref(&self, cp_index: u16) -> String {
        let method_ref_entry: &ConstantPoolInfo = &self.constant_pool[(cp_index - 1) as usize];
        match method_ref_entry {
            ConstantPoolInfo::MethodRef {
                class_index,
                name_and_type_index,
            } => self.get_method_ref_string(*class_index, *name_and_type_index),
            _ => panic!(
                "Expected entry #{} to be of Methodref type but it wasn't.",
                cp_index
            ),
        }
    }

    pub fn get_method_ref_string(&self, class_index: u16, name_and_type_index: u16) -> String {
        self.get_class_name(class_index) + "." + &self.get_name_and_type(name_and_type_index)
    }

    pub fn get_name_and_type(&self, cp_index: u16) -> String {
        let name_and_type_entry: &ConstantPoolInfo = &self.constant_pool[(cp_index - 1) as usize];
        match name_and_type_entry {
            ConstantPoolInfo::NameAndType {
                name_index,
                descriptor_index,
            } => self.get_name_and_type_string(*name_index, *descriptor_index),
            _ => panic!(
                "Expected entry #{} to be of NameAndType type but it wasn't.",
                cp_index
            ),
        }
    }

    pub fn get_name_and_type_string(&self, name_index: u16, descriptor_index: u16) -> String {
        let name = self.get_utf8_content(name_index);
        if name.starts_with('<') {
            "\"".to_owned() + &name + "\":" + &self.get_utf8_content(descriptor_index)
        } else {
            name + ":" + &self.get_utf8_content(descriptor_index)
        }
    }

    pub fn get_utf8_content(&self, cp_index: u16) -> String {
        let name_entry: &ConstantPoolInfo = &self.constant_pool[(cp_index - 1) as usize];
        match name_entry {
            ConstantPoolInfo::Utf8 { bytes } => {
                let content = convert_utf8(bytes);
                if content.starts_with('[') {
                    "\"".to_owned() + &content + "\""
                } else {
                    content
                }
            }
            _ => panic!(
                "Expected entry #{} to be of Utf8 type but it wasn't.",
                cp_index
            ),
        }
    }
}

pub fn convert_utf8(utf8_bytes: &[u8]) -> String {
    String::from_utf8(utf8_bytes.to_vec())
        .unwrap()
        .replace("\n", "\\n")
        .replace("'", "\\'")
        .replace("\u{0001}", "\\u0001")
}

pub struct MethodInfo {}

pub struct AttributeInfo {}

fn absolute_no_symlinks(p: &Path) -> Result<PathBuf> {
    if p.is_absolute() {
        Ok(p.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(p).canonicalize()?)
    }
}

pub fn parse_class_file(filename: String) -> ClassFile {
    let abs_file_path = absolute_no_symlinks(Path::new(&filename)).unwrap();
    let file = File::open(&abs_file_path).expect("File does not exist");
    let mut file_reader = BufReader::new(file);
    let mut file_bytes: Vec<u8> = Vec::with_capacity(file_reader.capacity());
    file_reader
        .read_to_end(&mut file_bytes)
        .expect("Could not read whole file");

    let digest = Sha256::digest(&file_bytes);

    let mut reader = BinaryReader::new(&file_bytes, Endian::Big);

    let actual_magic_number: u32 = reader.read_u32().unwrap();
    const EXPECTED_MAGIC_NUMBER: u32 = 0xcafebabe;
    if actual_magic_number != EXPECTED_MAGIC_NUMBER {
        panic!(
            "Wrong magic number: expected 0x{:08x} but was 0x{:08x}.",
            EXPECTED_MAGIC_NUMBER, actual_magic_number
        );
    }

    let minor_version: u16 = reader.read_u16().unwrap();
    let major_version: u16 = reader.read_u16().unwrap();

    let cp_count: u16 = reader.read_u16().unwrap();
    let constant_pool: Vec<ConstantPoolInfo> =
        parse_constant_pool(&mut reader, (cp_count - 1).into());

    let access_flags: Vec<AccessFlag> = parse_access_flags(reader.read_u16().unwrap());

    let this_class: u16 = reader.read_u16().unwrap();
    let super_class: u16 = reader.read_u16().unwrap();

    let interfaces_count: u16 = reader.read_u16().unwrap();
    let interfaces: Vec<u16> = reader.read_u16_vec(interfaces_count.into()).unwrap();

    let fields_count: u16 = reader.read_u16().unwrap();
    let fields: Vec<FieldInfo> = parse_fields(&mut reader, fields_count.into());

    let methods_count: u16 = reader.read_u16().unwrap();
    let methods: Vec<MethodInfo> = Vec::with_capacity(methods_count.into());

    let attributes_count: u16 = reader.read_u16().unwrap();
    let attributes: Vec<AttributeInfo> = Vec::with_capacity(attributes_count.into());

    ClassFile {
        absolute_file_path: abs_file_path.to_str().unwrap().to_string(),
        sha256_digest: digest.to_vec(),
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

fn parse_constant_pool(reader: &mut BinaryReader, cp_count: usize) -> Vec<ConstantPoolInfo> {
    let mut cp: Vec<ConstantPoolInfo> = Vec::with_capacity(cp_count);
    let mut i = 0;
    while i < cp_count {
        let tag = ConstantPoolTag::from(reader.read_u8().unwrap());
        cp.push(parse_constant_pool_info(reader, tag.clone()));
        if matches!(tag, ConstantPoolTag::Long) || matches!(tag, ConstantPoolTag::Double) {
            cp.push(ConstantPoolInfo::Null {});
            i += 1;
        }
        i += 1;
    }
    cp
}

fn parse_constant_pool_info(reader: &mut BinaryReader, tag: ConstantPoolTag) -> ConstantPoolInfo {
    match tag {
        ConstantPoolTag::Utf8 => {
            let length: u16 = reader.read_u16().unwrap();
            ConstantPoolInfo::Utf8 {
                bytes: reader.read_u8_vec(length.into()).unwrap(),
            }
        }
        ConstantPoolTag::Long => ConstantPoolInfo::Long {
            high_bytes: reader.read_u32().unwrap(),
            low_bytes: reader.read_u32().unwrap(),
        },
        ConstantPoolTag::String => ConstantPoolInfo::String {
            string_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::Class => ConstantPoolInfo::Class {
            name_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::Fieldref => ConstantPoolInfo::FieldRef {
            class_index: reader.read_u16().unwrap(),
            name_and_type_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::Methodref => ConstantPoolInfo::MethodRef {
            class_index: reader.read_u16().unwrap(),
            name_and_type_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::InterfaceMethodref => ConstantPoolInfo::InterfaceMethodRef {
            class_index: reader.read_u16().unwrap(),
            name_and_type_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::NameAndType => ConstantPoolInfo::NameAndType {
            name_index: reader.read_u16().unwrap(),
            descriptor_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::MethodHandle => ConstantPoolInfo::MethodHandle {
            reference_kind: ReferenceKind::from(reader.read_u8().unwrap()),
            reference_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::MethodType => ConstantPoolInfo::MethodType {
            descriptor_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::InvokeDynamic => ConstantPoolInfo::InvokeDynamic {
            bootstrap_method_attr_index: reader.read_u16().unwrap(),
            name_and_type_index: reader.read_u16().unwrap(),
        },
        _ => panic!("Unknown constant pool tag {:?}.", tag),
    }
}

fn parse_attributes(reader: &mut BinaryReader, num_attributes: usize) -> Vec<AttributeInfo> {
    return Vec::new();
}
