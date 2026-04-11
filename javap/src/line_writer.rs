#![forbid(unsafe_code)]

const TAB_COLUMN: usize = 40;

const INDENT_WIDTH: usize = 2;

pub(crate) struct LineWriter {
    buffer: String,
    indent_count: usize,
    pending_newline: bool,
    pending_spaces: usize,
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

    fn print(&mut self, s: &str) {
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
                        self.indent();
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
    }

    fn do_println(&mut self) {
        self.pending_spaces = 0;
        println!("{}", self.buffer);
        self.buffer.clear();
    }

    fn indent_delta(&mut self, delta: isize) {
        self.indent_count = (self.indent_count as isize + delta) as usize;
    }

    fn tab(&mut self) {
        let col = self.indent_count * INDENT_WIDTH + TAB_COLUMN;
        self.pending_spaces += if col <= self.buffer.len() {
            1
        } else {
            col - self.buffer.len()
        };
    }

    fn indent(&mut self) {
        self.pending_spaces += self.indent_count * INDENT_WIDTH;
    }
}

macro_rules! println {
    ($self:expr) => {
        $self.print("\n")
    };
    ($self:expr, $fmt:literal $(, $args:expr)*) => {
        $self.print(&format!($fmt $(, $args)*))
    };
}
