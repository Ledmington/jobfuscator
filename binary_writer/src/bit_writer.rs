pub struct BitWriter {
    buf: Vec<u8>,
}

impl BitWriter {
    pub fn new() -> Self {
        BitWriter { buf: Vec::new() }
    }

    pub fn write_u8_vec(&mut self, bytes: &[u8]) {
        self.buf.extend(bytes);
    }

    pub fn array(&self) -> Vec<u8> {
        self.buf.clone()
    }
}
