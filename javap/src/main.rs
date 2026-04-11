#![forbid(unsafe_code)]

mod javap;
mod line_writer;

use std::env;
use std::io::Result;

use crate::javap::print_class_file;

fn main() -> Result<()> {
    let filename = env::args().nth(1).expect("Usage: program <filename>");

    print_class_file(filename);

    Ok(())
}
