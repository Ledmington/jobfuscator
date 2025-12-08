pub mod access_flags;

use std::env;
use std::fs::File;
use std::io::{BufReader, Read, Result};
use std::path::{Path, PathBuf};

use binary_reader::{BinaryReader, Endian};
use sha2::{Digest, Sha256};

use crate::access_flags::AccessFlag;
use crate::access_flags::parse_access_flags;

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
            ConstantPoolInfo::Utf8 { bytes } => convert_utf8(bytes),
            _ => panic!(
                "Expected entry #{} to be of Utf8 type but it wasn't.",
                cp_index
            ),
        }
    }
}

pub fn convert_utf8(utf8_bytes: &Vec<u8>) -> String {
    String::from_utf8(utf8_bytes.to_vec())
        .unwrap()
        .replace("\n", "\\n")
}

pub enum ConstantPoolInfo {
    /**
     * The type of constant pool entry which can be found right after a Long or Double one.
     */
    Null {},
    Utf8 {
        bytes: Vec<u8>,
    },
    Long {
        high_bytes: u32,
        low_bytes: u32,
    },
    Double {
        high_bytes: u32,
        low_bytes: u32,
    },
    String {
        string_index: u16,
    },
    Class {
        name_index: u16,
    },
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    InterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    MethodType {
        descriptor_index: u16,
    },
    MethodHandle {
        reference_kind: ReferenceKind,
        reference_index: u16,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
}

#[derive(Debug, Clone)]
pub enum ConstantPoolTag {
    Utf8,
    Integer,
    Float,
    Long,
    Double,
    String,
    Class,
    Fieldref,
    Methodref,
    InterfaceMethodref,
    NameAndType,
    MethodHandle,
    MethodType,
    Dynamic,
    InvokeDynamic,
    Module,
    Package,
}

impl From<u8> for ConstantPoolTag {
    fn from(value: u8) -> Self {
        match value {
            1 => ConstantPoolTag::Utf8,
            3 => ConstantPoolTag::Integer,
            4 => ConstantPoolTag::Float,
            5 => ConstantPoolTag::Long,
            6 => ConstantPoolTag::Double,
            8 => ConstantPoolTag::String,
            7 => ConstantPoolTag::Class,
            9 => ConstantPoolTag::Fieldref,
            10 => ConstantPoolTag::Methodref,
            11 => ConstantPoolTag::InterfaceMethodref,
            12 => ConstantPoolTag::NameAndType,
            15 => ConstantPoolTag::MethodHandle,
            16 => ConstantPoolTag::MethodType,
            17 => ConstantPoolTag::Dynamic,
            18 => ConstantPoolTag::InvokeDynamic,
            19 => ConstantPoolTag::Module,
            20 => ConstantPoolTag::Package,
            _ => panic!("Unknown constant pool tag value {}.", value),
        }
    }
}

pub enum ReferenceKind {
    GetField,
    GetStatic,
    PutField,
    PutStatic,
    InvokeVirtual,
    InvokeStatic,
    InvokeSpecial,
    NewInvokeSpecial,
    InvokeInterface,
}

impl From<u8> for ReferenceKind {
    fn from(value: u8) -> Self {
        match value {
            1 => ReferenceKind::GetField,
            2 => ReferenceKind::GetStatic,
            3 => ReferenceKind::PutField,
            4 => ReferenceKind::PutStatic,
            5 => ReferenceKind::InvokeVirtual,
            6 => ReferenceKind::InvokeStatic,
            7 => ReferenceKind::InvokeSpecial,
            8 => ReferenceKind::NewInvokeSpecial,
            9 => ReferenceKind::InvokeInterface,
            _ => panic!("Unknwon reference_kind value {}.", value),
        }
    }
}

pub struct FieldInfo {}

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
    let cp: Vec<ConstantPoolInfo> = parse_constant_pool(&mut reader, (cp_count - 1).into());

    let access_flags: Vec<AccessFlag> = parse_access_flags(reader.read_u16().unwrap());

    let this_class: u16 = reader.read_u16().unwrap();
    let super_class: u16 = reader.read_u16().unwrap();

    let interfaces_count: u16 = reader.read_u16().unwrap();
    let interfaces: Vec<u16> = reader.read_u16_vec(interfaces_count.into()).unwrap();

    let fields_count: u16 = reader.read_u16().unwrap();
    let fields: Vec<FieldInfo> = Vec::with_capacity(fields_count.into());

    let methods_count: u16 = reader.read_u16().unwrap();
    let methods: Vec<MethodInfo> = Vec::with_capacity(methods_count.into());

    let attributes_count: u16 = reader.read_u16().unwrap();
    let attributes: Vec<AttributeInfo> = Vec::with_capacity(attributes_count.into());

    ClassFile {
        absolute_file_path: abs_file_path.to_str().unwrap().to_string(),
        sha256_digest: digest.to_vec(),
        minor_version,
        major_version,
        constant_pool: cp,
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
