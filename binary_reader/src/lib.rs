use std::io::{Error, ErrorKind, Result};

pub enum Endian {
    Little,
    Big,
}

pub struct BinaryReader<'a> {
    buf: &'a [u8],
    pos: usize,
    endian: Endian,
}

impl<'a> BinaryReader<'a> {
    pub fn new(buf: &'a [u8], endian: Endian) -> Self {
        Self {
            buf,
            pos: 0,
            endian,
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

    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_bytes(1)?[0])
    }

    pub fn read_u8_vec(&mut self, count: usize) -> Result<Vec<u8>> {
        Ok(self.read_bytes(count)?.to_vec())
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let bytes: [u8; 2] = self.read_bytes(2).unwrap().try_into().unwrap();
        Ok(match self.endian {
            Endian::Little => u16::from_le_bytes(bytes),
            Endian::Big => u16::from_be_bytes(bytes),
        })
    }

    pub fn read_u16_vec(&mut self, count: usize) -> Result<Vec<u16>> {
        debug_assert!(count > 0);
        let mut res: Vec<u16> = vec![0u16; count];
        for x in res.iter_mut().take(count) {
            *x = self.read_u16().unwrap();
        }
        Ok(res)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let bytes: [u8; 4] = self.read_bytes(4).unwrap().try_into().unwrap();
        Ok(match self.endian {
            Endian::Little => u32::from_le_bytes(bytes),
            Endian::Big => u32::from_be_bytes(bytes),
        })
    }
}
