#![forbid(unsafe_code)]

pub mod access_flags;
pub mod attributes;
pub mod bytecode;
pub mod constant_pool;
pub mod fields;
pub mod methods;
pub mod reference_kind;

use std::env;
use std::fs::File;
use std::io::{BufReader, Read, Result};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use binary_reader::{BinaryReader, Endian};
use sha2::{Digest, Sha256};

use crate::access_flags::AccessFlag;
use crate::access_flags::parse_access_flags;
use crate::attributes::{AttributeInfo, parse_attributes};
use crate::constant_pool::{ConstantPool, parse_constant_pool};
use crate::fields::{FieldInfo, parse_fields};
use crate::methods::{MethodInfo, parse_methods};

/**
 * Specification available at <https://docs.oracle.com/javase/specs/jvms/se25/html/jvms-4.html>
 */
pub struct ClassFile {
    pub absolute_file_path: String,
    pub modified_time: SystemTime,
    pub file_size: usize,
    pub sha256_digest: Vec<u8>,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: Vec<AccessFlag>,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttributeInfo>,
}

fn absolute_no_symlinks(p: &Path) -> Result<PathBuf> {
    if p.is_absolute() {
        Ok(p.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(p).canonicalize()?)
    }
}

pub fn parse_class_file(filename: String) -> ClassFile {
    let abs_file_path = absolute_no_symlinks(Path::new(&filename)).unwrap();
    let absolute_file_path = abs_file_path.to_str().unwrap().to_owned();
    let file = File::open(&abs_file_path).expect("File does not exist");
    let modified_time: SystemTime = file.metadata().unwrap().modified().unwrap();
    let mut file_reader = BufReader::new(file);
    let mut file_bytes: Vec<u8> = Vec::with_capacity(file_reader.capacity());
    file_reader
        .read_to_end(&mut file_bytes)
        .expect("Could not read whole file");
    let file_size: usize = file_bytes.len();

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
    let constant_pool: ConstantPool = parse_constant_pool(&mut reader, (cp_count - 1).into());

    let access_flags: Vec<AccessFlag> = parse_access_flags(reader.read_u16().unwrap());

    let this_class: u16 = reader.read_u16().unwrap();
    let super_class: u16 = reader.read_u16().unwrap();

    let interfaces_count: u16 = reader.read_u16().unwrap();
    let interfaces: Vec<u16> = reader.read_u16_vec(interfaces_count.into()).unwrap();

    let fields_count: u16 = reader.read_u16().unwrap();
    let fields: Vec<FieldInfo> = parse_fields(&mut reader, &constant_pool, fields_count.into());

    let methods_count: u16 = reader.read_u16().unwrap();
    let methods: Vec<MethodInfo> = parse_methods(&mut reader, &constant_pool, methods_count.into());

    let attributes_count: u16 = reader.read_u16().unwrap();
    let attributes: Vec<AttributeInfo> =
        parse_attributes(&mut reader, &constant_pool, attributes_count.into());

    ClassFile {
        absolute_file_path,
        modified_time,
        file_size,
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

pub fn get_return_type(descriptor: &String) -> String {
    convert_descriptor(&descriptor.split(')').last().unwrap().to_string())
}

pub fn convert_descriptor(descriptor: &String) -> String {
    match descriptor.chars().nth(0).unwrap().to_string().as_str() {
        "V" => "void".to_owned(),
        "J" => "long".to_owned(),
        "L" => descriptor[1..(descriptor.len() - 1)].replace('/', "."),
        "(" => {
            let args: String = descriptor[1..].split(")").nth(0).unwrap().to_string();
            "(".to_owned() + &convert_descriptor(&args) + ")"
        }
        "[" => convert_descriptor(&descriptor[1..].to_string()) + "[]",
        _ => descriptor.to_string(),
    }
}
