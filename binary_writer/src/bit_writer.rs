pub struct BitWriter {
    buf: Vec<u8>,
}

impl BitWriter {
    pub fn new() -> Self {
        BitWriter { buf: Vec::new() }
    }

    pub fn array(&self) -> Vec<u8> {
        self.buf.clone()
    }
}
