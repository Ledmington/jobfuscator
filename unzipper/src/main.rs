#![forbid(unsafe_code)]

use zip::zip_parser::parse_zip;

pub fn main() {
    let filename = std::env::args()
        .nth(1)
        .expect("Error: missing zip file\n\nUsage: unzipper my_file.zip\n");

    let file = parse_zip(&filename);

    for entry in file.entries() {
        println!("{}", entry.name());
    }
}
