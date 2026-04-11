#![forbid(unsafe_code)]

mod javap;
mod line_writer;

use std::env;
use std::io::Result;

use crate::javap::print_class_file;

/**
 * Re-implementation of java command line utility javap. Used just for testing. Original source code available <a
 * href="https://github.com/openjdk/jdk/tree/master/src/jdk.jdeps/share/classes/com/sun/tools/javap">here</a>.
 */
fn main() -> Result<()> {
    let filename = env::args().nth(1).expect("Usage: program <filename>");

    print_class_file(filename);

    Ok(())
}
