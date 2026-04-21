#![forbid(unsafe_code)]

pub mod bit_reader;
pub mod byte_reader;

pub enum Endianness {
    Little,
    Big,
}
