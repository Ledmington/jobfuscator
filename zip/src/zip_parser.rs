#![forbid(unsafe_code)]

use std::{fs::File, io::Read};

use binary_reader::bit_reader::BitReader;

use crate::ZipFile;

pub fn parse_zip(filename: &str) -> ZipFile {
    let mut file = File::open(filename)
        .unwrap_or_else(|err| panic!("Could not open file '{}' due to: {}.", filename, err));
    let mut file_bytes = Vec::new();
    file.read_to_end(&mut file_bytes)
        .unwrap_or_else(|err| panic!("Could not read file '{}' due to: {}.", filename, err));
    parse_zip_buf(&BitReader::new(&file_bytes))
}

fn parse_zip_buf(reader: &BitReader) -> ZipFile {
    ZipFile {}
}
