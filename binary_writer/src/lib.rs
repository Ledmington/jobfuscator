#![forbid(unsafe_code)]

pub mod byte_writer;
pub mod bit_writer;

pub enum Endianness {
    Little,
    Big,
}
