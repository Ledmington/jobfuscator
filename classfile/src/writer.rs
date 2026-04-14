#![forbid(unsafe_code)]

use binary_writer::{BinaryWriter, Endianness};

use crate::classfile::ClassFile;

pub fn write_class_file(cf: &ClassFile) -> Vec<u8> {
    let writer: BinaryWriter = BinaryWriter::new(Endianness::Big);
    writer.array()
}
