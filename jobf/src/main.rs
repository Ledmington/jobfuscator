#![forbid(unsafe_code)]

use std::{
    env::args,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

use binary_reader::BinaryReader;
use classfile::{
    classfile::{ClassFile, parse_class_file},
    writer::write_class_file,
};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::FileOptions};

fn is_class_file(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xCA, 0xFE, 0xBA, 0xBE])
}

fn is_zip_file(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04])
}

fn transform_class_file(cf: &ClassFile) -> &ClassFile {
    // TODO
    cf
}

fn parse_and_rewrite(reader: &mut BinaryReader) -> Vec<u8> {
    let in_cf: ClassFile = parse_class_file(reader);
    let out_cf = transform_class_file(&in_cf);
    write_class_file(out_cf)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_filename: String = args()
        .nth(1)
        .expect("Usage: jobf <input_file> [output_file]");
    let output_filename: String = args().nth(2).unwrap_or(input_filename.clone() + ".out");

    let mut file = File::open(&input_filename)?;

    // Read first few bytes to detect file type
    let mut header = [0u8; 4];
    file.read_exact(&mut header)?;

    // rewind file
    file.seek(SeekFrom::Start(0))?;

    if is_class_file(&header) {
        let mut file_bytes = Vec::new();
        file.read_to_end(&mut file_bytes)?;

        let mut reader = BinaryReader::new(&file_bytes, binary_reader::Endianness::Big);
        let out_bytes = parse_and_rewrite(&mut reader);
        println!(
            "{} -> valid class file ({} bytes)",
            &input_filename,
            file_bytes.len()
        );

        let mut out_file = File::create(&output_filename)?;
        out_file.write_all(&out_bytes)?;

        println!("Wrote output to {}", &output_filename);
    } else if is_zip_file(&header) {
        let mut archive = ZipArchive::new(file)?;

        let out_file = File::create(&output_filename)?;
        let mut zip_writer = ZipWriter::new(out_file);

        let options = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let name = entry.name().to_string();

            if entry.is_dir() {
                zip_writer.add_directory(name, options)?;
                continue;
            }

            let mut file_bytes = Vec::new();
            entry.read_to_end(&mut file_bytes)?;

            let mut reader = BinaryReader::new(&file_bytes, binary_reader::Endianness::Big);
            let out_bytes = parse_and_rewrite(&mut reader);
            println!("{} ({} bytes) OK", name, file_bytes.len());

            zip_writer.start_file(name, options)?;
            zip_writer.write_all(&out_bytes)?;
        }

        zip_writer.finish()?;
        println!("Wrote output jar to {}", &output_filename);
    } else {
        return Err("Unknown file type (not .class or .jar)".into());
    }

    Ok(())
}
