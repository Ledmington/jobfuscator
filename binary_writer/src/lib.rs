#![forbid(unsafe_code)]

pub enum Endianness {
    Little,
    Big,
}

pub struct BinaryWriter {
    buf: Vec<u8>,
    pos: usize,
    endianness: Endianness,
}

impl BinaryWriter {
    pub fn new(endianness: Endianness) -> Self {
        Self {
            buf: Vec::new(),
            pos: 0,
            endianness,
        }
    }

    pub fn write_u8(&mut self, x: u8) {
        self.buf.push(x);
        self.pos += 1;
    }

    pub fn write_u8_vec(&mut self, x: &[u8]) {
        for v in x.iter() {
            self.buf.push(*v);
        }
        self.pos += x.len();
    }

    pub fn write_i8(&mut self, x: i8) {
        self.buf.push(x as u8);
        self.pos += 1;
    }

    pub fn write_u16(&mut self, x: u16) {
        for v in match self.endianness {
            Endianness::Big => u16::to_be_bytes(x),
            Endianness::Little => u16::to_le_bytes(x),
        } {
            self.buf.push(v);
        }
        self.pos += 2;
    }

    pub fn write_i16(&mut self, x: i16) {
        for v in match self.endianness {
            Endianness::Big => i16::to_be_bytes(x),
            Endianness::Little => i16::to_le_bytes(x),
        } {
            self.buf.push(v);
        }
        self.pos += 2;
    }

    pub fn write_u16_vec(&mut self, x: &Vec<u16>) {
        for v in x {
            self.write_u16(*v);
        }
    }

    pub fn write_u32(&mut self, x: u32) {
        for v in match self.endianness {
            Endianness::Big => u32::to_be_bytes(x),
            Endianness::Little => u32::to_le_bytes(x),
        } {
            self.buf.push(v);
        }
        self.pos += 4;
    }

    pub fn write_i32(&mut self, x: i32) {
        for v in match self.endianness {
            Endianness::Big => i32::to_be_bytes(x),
            Endianness::Little => i32::to_le_bytes(x),
        } {
            self.buf.push(v);
        }
        self.pos += 4;
    }

    pub fn write_i32_vec(&mut self, x: &Vec<i32>) {
        for v in x {
            self.write_i32(*v);
        }
    }

    pub fn array(&self) -> Vec<u8> {
        self.buf.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_be_bytes() {
        let mut writer: BinaryWriter = BinaryWriter::new(Endianness::Big);
        writer.write_u8(1);
        writer.write_u8(2);
        writer.write_u8(3);
        writer.write_u8(4);
        assert_eq!(vec![1u8, 2u8, 3u8, 4u8], writer.array());
    }

    #[test]
    fn write_le_bytes() {
        let mut writer: BinaryWriter = BinaryWriter::new(Endianness::Little);
        writer.write_u8(1);
        writer.write_u8(2);
        writer.write_u8(3);
        writer.write_u8(4);
        assert_eq!(vec![1u8, 2u8, 3u8, 4u8], writer.array());
    }

    #[test]
    fn write_be_u16() {
        let mut writer: BinaryWriter = BinaryWriter::new(Endianness::Big);
        writer.write_u16(0x0102u16);
        assert_eq!(vec![1u8, 2u8], writer.array());
    }

    #[test]
    fn write_le_u16() {
        let mut writer: BinaryWriter = BinaryWriter::new(Endianness::Little);
        writer.write_u16(0x0102u16);
        assert_eq!(vec![2u8, 1u8], writer.array());
    }

    #[test]
    fn write_be_i16() {
        let mut writer: BinaryWriter = BinaryWriter::new(Endianness::Big);
        writer.write_i16(0x0102i16);
        assert_eq!(vec![1u8, 2u8], writer.array());
    }

    #[test]
    fn write_le_i16() {
        let mut writer: BinaryWriter = BinaryWriter::new(Endianness::Little);
        writer.write_i16(0x0102i16);
        assert_eq!(vec![2u8, 1u8], writer.array());
    }

    #[test]
    fn write_be_u32() {
        let mut writer: BinaryWriter = BinaryWriter::new(Endianness::Big);
        writer.write_u32(0x01020304u32);
        assert_eq!(vec![1u8, 2u8, 3u8, 4u8], writer.array());
    }

    #[test]
    fn write_le_u32() {
        let mut writer: BinaryWriter = BinaryWriter::new(Endianness::Little);
        writer.write_u32(0x01020304u32);
        assert_eq!(vec![4u8, 3u8, 2u8, 1u8], writer.array());
    }

    #[test]
    fn write_be_i32() {
        let mut writer: BinaryWriter = BinaryWriter::new(Endianness::Big);
        writer.write_i32(0x01020304i32);
        assert_eq!(vec![1u8, 2u8, 3u8, 4u8], writer.array());
    }

    #[test]
    fn write_le_i32() {
        let mut writer: BinaryWriter = BinaryWriter::new(Endianness::Little);
        writer.write_i32(0x01020304i32);
        assert_eq!(vec![4u8, 3u8, 2u8, 1u8], writer.array());
    }
}
