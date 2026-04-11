use std::{env::args, fs::File, io::Read};

use binary_reader::BinaryReader;
use classfile::classfile::{ClassFile, parse_class_file};
use zip::ZipArchive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = args().nth(1).expect("Usage: jobf <jar file>");
    let file = File::open(filename)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        if !entry.name().ends_with(".class") {
            println!(
                "{} ({} bytes) -> not a class file",
                entry.name(),
                entry.size()
            );
            continue;
        }

        let mut file_bytes: Vec<u8> = Vec::with_capacity(entry.size().try_into().unwrap());
        entry.read_to_end(&mut file_bytes)?;
        let mut reader = BinaryReader::new(&file_bytes, binary_reader::Endianness::Big);
        let _cf: ClassFile = parse_class_file(&mut reader);
        println!("{} ({} bytes) OK", entry.name(), entry.size());
    }

    // TODO: write back class files

    Ok(())
}
