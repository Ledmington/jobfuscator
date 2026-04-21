#![forbid(unsafe_code)]

use std::io::{Error, ErrorKind, Result};

use crate::Endianness;

pub struct ByteReader<'a> {
    buf: &'a [u8],
    pos: usize,
    endianness: Endianness,
}

impl<'a> ByteReader<'a> {
    pub fn new(buf: &'a [u8], endianness: Endianness) -> Self {
        Self {
            buf,
            pos: 0,
            endianness,
        }
    }

    fn read_bytes(&mut self, count: usize) -> Result<&'a [u8]> {
        if self.pos + count > self.buf.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Not enough bytes"));
        }
        let slice = &self.buf[self.pos..self.pos + count];
        self.pos += count;
        Ok(slice)
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        Ok(self.read_bytes(N)?.try_into().unwrap()) // try_into on a &[u8] of known size N cannot fail
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_bytes(1)?[0])
    }

    pub fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let bytes = self.read_array::<2>()?;
        Ok(match self.endianness {
            Endianness::Little => u16::from_le_bytes(bytes),
            Endianness::Big => u16::from_be_bytes(bytes),
        })
    }

    pub fn read_i16(&mut self) -> Result<i16> {
        let bytes = self.read_array::<2>()?;
        Ok(match self.endianness {
            Endianness::Little => i16::from_le_bytes(bytes),
            Endianness::Big => i16::from_be_bytes(bytes),
        })
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let bytes = self.read_array::<4>()?;
        Ok(match self.endianness {
            Endianness::Little => u32::from_le_bytes(bytes),
            Endianness::Big => u32::from_be_bytes(bytes),
        })
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        let bytes = self.read_array::<4>()?;
        Ok(match self.endianness {
            Endianness::Little => i32::from_le_bytes(bytes),
            Endianness::Big => i32::from_be_bytes(bytes),
        })
    }

    pub fn read_u8_vec(&mut self, count: usize) -> Result<Vec<u8>> {
        Ok(self.read_bytes(count)?.to_vec())
    }

    pub fn read_u16_vec(&mut self, count: usize) -> Result<Vec<u16>> {
        (0..count).map(|_| self.read_u16()).collect()
    }

    pub fn read_i32_vec(&mut self, count: usize) -> Result<Vec<i32>> {
        (0..count).map(|_| self.read_i32()).collect()
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reading_be_bytes() {
        let buffer: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let mut reader: ByteReader = ByteReader::new(&buffer, Endianness::Big);
        for x in buffer {
            assert_eq!(x, reader.read_u8().unwrap());
        }
    }

    #[test]
    fn reading_le_bytes() {
        let buffer: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let mut reader: ByteReader = ByteReader::new(&buffer, Endianness::Little);
        for x in buffer {
            assert_eq!(x, reader.read_u8().unwrap());
        }
    }

    #[test]
    fn reading_be_u16() {
        let buffer: [u8; 2] = [1, 2];
        let mut reader: ByteReader = ByteReader::new(&buffer, Endianness::Big);
        assert_eq!(0x0102u16, reader.read_u16().unwrap());
    }

    #[test]
    fn reading_le_u16() {
        let buffer: [u8; 2] = [1, 2];
        let mut reader: ByteReader = ByteReader::new(&buffer, Endianness::Little);
        assert_eq!(0x0201u16, reader.read_u16().unwrap());
    }

    #[test]
    fn reading_be_u32() {
        let buffer: [u8; 4] = [1, 2, 3, 4];
        let mut reader: ByteReader = ByteReader::new(&buffer, Endianness::Big);
        assert_eq!(0x01020304u32, reader.read_u32().unwrap());
    }

    #[test]
    fn reading_le_u32() {
        let buffer: [u8; 4] = [1, 2, 3, 4];
        let mut reader: ByteReader = ByteReader::new(&buffer, Endianness::Little);
        assert_eq!(0x04030201u32, reader.read_u32().unwrap());
    }
}