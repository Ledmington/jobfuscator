#![forbid(unsafe_code)]

use std::io::Write;

struct LineWriter<W: Write> {
    out: W,
    buffer: String,
    indent_count: usize,
    indent_width: usize,
    tab_column: usize,
    pending_newline: bool,
    pending_spaces: usize,
}

impl<W: Write> LineWriter<W> {
    fn new(out: W, indent_width: usize, tab_column: usize) -> Self {
        Self {
            out,
            buffer: String::new(),
            indent_count: 0,
            indent_width,
            tab_column,
            pending_newline: false,
            pending_spaces: 0,
        }
    }

    fn print(&mut self, s: &str) {
        if self.pending_newline {
            self.println();
            self.pending_newline = false;
        }

        for c in s.chars() {
            match c {
                ' ' => self.pending_spaces += 1,
                '\n' => self.println(),
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

    fn println(&mut self) {
        self.pending_spaces = 0;
        writeln!(self.out, "{}", self.buffer).expect("failed to write");
        self.buffer.clear();
    }

    fn indent_delta(&mut self, delta: isize) {
        self.indent_count = (self.indent_count as isize + delta) as usize;
    }

    fn tab(&mut self) {
        let col = self.indent_count * self.indent_width + self.tab_column;
        self.pending_spaces += if col <= self.buffer.len() {
            1
        } else {
            col - self.buffer.len()
        };
    }

    fn indent(&mut self) {
        self.pending_spaces += self.indent_count * self.indent_width;
    }
}
