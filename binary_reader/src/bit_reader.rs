#![forbid(unsafe_code)]

use std::io::Result;

/// A binary reader which allows to read single bits.
/// Big endian.
pub struct BitReader<'a> {
    buf: &'a [u8],
    bit_position: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        BitReader {
            buf,
            bit_position: 0,
        }
    }

    pub fn size(&self) -> usize {
        self.buf.len()
    }

    pub fn get_byte_position(&self) -> usize {
        self.bit_position / 8
    }

    pub fn set_byte_position(&mut self, new_pos: usize) {
        self.bit_position = new_pos * 8
    }

    pub fn read_bit(&mut self) -> Result<bool> {
        let byte_pos = self.bit_position / 8;
        let bit_pos = self.bit_position % 8;
        self.bit_position += 1;
        Ok((self.buf[byte_pos] & (1 << (7 - bit_pos))) != 0)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let mut x: u8 = 0;
        for i in 0..8 {
            if self.read_bit()? {
                x |= 1u8 << (7 - i);
            }
        }
        Ok(x)
    }

    pub fn read_u8_vec(&mut self, n: usize) -> Result<Vec<u8>> {
        let mut v: Vec<u8> = Vec::with_capacity(n);
        for _ in 0..n {
            v.push(self.read_u8()?);
        }
        Ok(v)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let mut x: u16 = 0;
        for i in 0..16 {
            if self.read_bit()? {
                x |= 1u16 << (15 - i);
            }
        }
        Ok(x)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let mut x: u32 = 0;
        for i in 0..32 {
            if self.read_bit()? {
                x |= 1u32 << (31 - i);
            }
        }
        Ok(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reading_be_bits() {
        let buffer: [u8; 1] = [0b01101001];
        let mut reader: BitReader = BitReader::new(&buffer);

        assert!(!reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(!reader.read_bit().unwrap());
        assert!(reader.read_bit().unwrap());
    }

    #[test]
    fn reading_bytes() {
        let buffer: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let mut reader: BitReader = BitReader::new(&buffer);
        for x in buffer {
            assert_eq!(x, reader.read_u8().unwrap());
        }
    }

    #[test]
    fn reading_u16() {
        let buffer: [u8; 2] = [1, 2];
        let mut reader: BitReader = BitReader::new(&buffer);
        assert_eq!(0x0102u16, reader.read_u16().unwrap());
    }

    #[test]
    fn reading_u32() {
        let buffer: [u8; 4] = [1, 2, 3, 4];
        let mut reader: BitReader = BitReader::new(&buffer);
        assert_eq!(0x01020304u32, reader.read_u32().unwrap());
    }
}
