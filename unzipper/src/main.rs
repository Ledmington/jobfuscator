#![forbid(unsafe_code)]

use zip::{ZipFile, zip_parser::parse_zip};

pub fn main() {
    let filename = std::env::args()
        .nth(1)
        .expect("Error: missing zip file\n\nUsage: unzipper my_file.zip\n");

    let file: ZipFile = parse_zip(&filename);

    let num_entries = file.num_entries();
    for (i, entry) in file.entries().iter().enumerate() {
        println!("Entry n. {} / {}", i + 1, num_entries);
        println!("  Name: '{}'", entry.name());
        println!("  Version made by: {}", entry.version_made_by());
        println!("  Minimum version: {}", entry.minimum_version());
        println!("  Flags: 0x{:04x}", entry.bit_flags());
        println!("  Compression: {}", entry.compression_method());
        println!(
            "  Last modification: {} {}",
            entry.last_modification_date(),
            entry.last_modification_time()
        );
        println!("  C. Size: {} bytes", entry.compressed_size());
        println!(
            "  Comment: '{}'{}",
            entry.comment(),
            if entry.comment().is_empty() {
                " (empty)"
            } else {
                ""
            }
        );
        println!();
    }
}
