#![forbid(unsafe_code)]

pub struct BitReader {
    buf: Vec<u8>,
    bit_position: usize,
}

impl BitReader {
    pub fn new(buf: &Vec<u8>) -> Self {
        BitReader {
            buf: buf.to_vec(),
            bit_position: 0,
        }
    }
}
