const TAB_COLUMN: isize = 40;

const INDENT_WIDTH: isize = 2;

pub(crate) struct LineWriter {
    buffer: String,
    indent_count: isize,
    pending_newline: bool,
    pending_spaces: isize,
}

impl LineWriter {
    pub(crate) fn new() -> Self {
        Self {
            buffer: String::new(),
            indent_count: 0,
            pending_newline: false,
            pending_spaces: 0,
        }
    }

    pub(crate) fn println(&mut self, s: &str) -> &mut Self {
        self.print(s).print("\n");
        self
    }

    pub(crate) fn print(&mut self, s: &str) -> &mut Self {
        if self.pending_newline {
            self.do_println();
            self.pending_newline = false;
        }

        for c in s.chars() {
            match c {
                ' ' => self.pending_spaces += 1,
                '\n' => self.do_println(),
                _ => {
                    if self.buffer.is_empty() {
                        self.do_indent();
                    }
                    if self.pending_spaces > 0 {
                        for _ in 0..self.pending_spaces {
                            self.buffer.push(' ');
                        }
                        self.pending_spaces = 0;
                    }
                    self.buffer.push(c);
                }
            }
        }

        self
    }

    fn do_println(&mut self) {
        self.pending_spaces = 0;
        println!("{}", self.buffer);
        self.buffer.clear();
    }

    pub(crate) fn tab(&mut self) -> &mut Self {
        let col = self.indent_count * INDENT_WIDTH + TAB_COLUMN;
        let buf_len = self.buffer.len().try_into().unwrap();
        self.pending_spaces += if col <= buf_len { 1 } else { col - buf_len };
        self
    }

    fn do_indent(&mut self) {
        self.pending_spaces += self.indent_count * INDENT_WIDTH;
    }

    pub(crate) fn indent(&mut self, delta: isize) -> &mut Self {
        self.indent_count += delta;
        self
    }
}
