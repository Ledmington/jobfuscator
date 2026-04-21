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

    pub fn size(&self) -> usize {
        self.buf.len()
    }

    pub fn get_byte_position(&self) -> usize {
        self.bit_position / 8
    }

    pub fn set_byte_position(&mut self, new_pos: usize) {
        self.bit_position = new_pos * 8
    }

    fn read_bit(&mut self) -> bool {
        let byte_pos = self.bit_position / 8;
        let bit_pos = self.bit_position % 8;
        self.bit_position += 1;
        (self.buf[byte_pos] & (1 << 7 - bit_pos)) != 0
    }

    pub fn read_u8(&mut self) -> u8 {
        let mut x: u8 = 0;
        for i in 0..8 {
            if self.read_bit() {
                x |= 1u8 << (7 - i);
            }
        }
        x
    }

    pub fn read_u8_vec(&mut self, n: usize) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::with_capacity(n);
        for _ in 0..n {
            v.push(self.read_u8());
        }
        v
    }

    pub fn read_u16(&mut self) -> u16 {
        let mut x: u16 = 0;
        for i in 0..16 {
            if self.read_bit() {
                x |= 1u16 << (15 - i);
            }
        }
        x
    }

    pub fn read_u32(&mut self) -> u32 {
        let mut x: u32 = 0;
        for i in 0..32 {
            if self.read_bit() {
                x |= 1u32 << (31 - i);
            }
        }
        x
    }
}
