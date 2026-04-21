#![forbid(unsafe_code)]

mod make_everything_public;
mod pipeline;
mod shuffle_fields;
mod shuffle_methods;
mod transformation;

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use binary_reader::byte_reader::ByteReader;
use classfile::{
    classfile::{ClassFile, parse_class_file},
    writer::write_class_file,
};
use cli_parser::{CommandLineOption, CommandLineParser, CommandLineType};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::FileOptions};

use crate::{
    make_everything_public::MakeEverythingPublic, pipeline::TransformationPipeline,
    shuffle_fields::ShuffleFields, shuffle_methods::ShuffleMethods,
};

fn is_class_file(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xCA, 0xFE, 0xBA, 0xBE])
}

fn is_zip_file(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04])
}

fn parse_and_rewrite(reader: &mut ByteReader, pipeline: &TransformationPipeline) -> Vec<u8> {
    let in_cf: ClassFile = parse_class_file(reader);
    let out_cf = pipeline.execute(&in_cf);
    write_class_file(&out_cf)
}

macro_rules! log {
    ($quiet:expr, $($arg:tt)*) => {
        if !$quiet {
            println!($($arg)*);
        }
    };
}

macro_rules! die {
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
        std::process::exit(1);
    }};
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = CommandLineParser::new(
        "jobf",
        Some("The java class file obfuscator.".to_owned()),
        vec![
            CommandLineOption::new(
                Some("i".to_owned()),
                Some("input".to_owned()),
                "The file to read from.".to_owned(),
                CommandLineType::String {
                    default_value: None,
                },
            ),
            CommandLineOption::new(
                Some("o".to_owned()),
                Some("output".to_owned()),
                "The file to write to.".to_owned(),
                CommandLineType::String {
                    default_value: None,
                },
            ),
            CommandLineOption::new(
                Some("f".to_owned()),
                Some("force".to_owned()),
                "When enabled, overwrites the output file if it already exists.".to_owned(),
                CommandLineType::Boolean {
                    default_value: Some(false),
                },
            ),
            CommandLineOption::new(
                Some("q".to_owned()),
                Some("quiet".to_owned()),
                "Avoids printing on stdout.".to_owned(),
                CommandLineType::Boolean {
                    default_value: Some(false),
                },
            ),
            CommandLineOption::new(
                Some("s".to_owned()),
                Some("seed".to_owned()),
                "64-bit seed for RNG-based transformations (accepts hexadecimal and decimal)."
                    .to_owned(),
                CommandLineType::U64 {
                    default_value: Some(42u64),
                },
            ),
            CommandLineOption::new(
                None,
                Some("make-everything-public".to_owned()),
                "Converts all classes, fields and methods to public.".to_owned(),
                CommandLineType::Boolean {
                    default_value: Some(false),
                },
            ),
            CommandLineOption::new(
                None,
                Some("shuffle-fields".to_owned()),
                "Shuffles the fields inside a class.".to_owned(),
                CommandLineType::Boolean {
                    default_value: Some(false),
                },
            ),
            CommandLineOption::new(
                None,
                Some("shuffle-methods".to_owned()),
                "Shuffles the methods inside a class.".to_owned(),
                CommandLineType::Boolean {
                    default_value: Some(false),
                },
            ),
        ],
    );

    let args = parser.parse(std::env::args());

    let input_filename = args.get("input").unwrap().as_str();
    let output_filename = args.get("output").unwrap().as_str();
    let force = args.get("force").unwrap().as_bool();
    let quiet = args.get("quiet").unwrap().as_bool();
    let make_everything_public = args.get("make-everything-public").unwrap().as_bool();
    let shuffle_fields = args.get("shuffle-fields").unwrap().as_bool();
    let shuffle_methods = args.get("shuffle-methods").unwrap().as_bool();
    let seed: u64 = args.get("seed").unwrap().as_u64();

    let mut pipeline: TransformationPipeline = TransformationPipeline::new();

    if make_everything_public {
        pipeline.add(Box::new(MakeEverythingPublic {}));
    }
    if shuffle_fields {
        pipeline.add(Box::new(ShuffleFields::new(seed)));
    }
    if shuffle_methods {
        pipeline.add(Box::new(ShuffleMethods::new(seed)));
    }

    let mut file = File::open(&input_filename)
        .unwrap_or_else(|err| die!("Could not open file '{}' due to: {}.", input_filename, err));

    // Read first few bytes to detect file type and rewind
    let mut header = [0u8; 4];
    file.read_exact(&mut header)?;
    file.seek(SeekFrom::Start(0))?;

    let is_class_file = is_class_file(&header);
    let is_zip_file = is_zip_file(&header);

    if !is_class_file && !is_zip_file {
        die!("Unknown input file type (not .class or .jar)");
    }

    if Path::new(&output_filename).exists() && !force {
        die!(
            "Output file '{}' already exists. To overwrite it, re-run with '--force'.",
            output_filename
        );
    }

    if is_class_file {
        let mut file_bytes = Vec::new();
        file.read_to_end(&mut file_bytes)?;

        let mut reader = ByteReader::new(&file_bytes, binary_reader::Endianness::Big);
        let out_bytes = parse_and_rewrite(&mut reader, &pipeline);
        log!(
            quiet,
            "{} -> valid class file ({} bytes)",
            &input_filename,
            file_bytes.len()
        );

        let mut out_file = File::create(&output_filename).unwrap_or_else(|err| {
            die!(
                "Could not create file '{}' due to: {}.",
                output_filename,
                err
            )
        });
        out_file.write_all(&out_bytes)?;

        log!(quiet, "Wrote output to {}", &output_filename);
    } else if is_zip_file {
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

            let mut reader = ByteReader::new(&file_bytes, binary_reader::Endianness::Big);
            let out_bytes = parse_and_rewrite(&mut reader, &pipeline);
            log!(quiet, "{} ({} bytes) OK", name, file_bytes.len());

            zip_writer.start_file(name, options)?;
            zip_writer.write_all(&out_bytes)?;
        }

        zip_writer.finish()?;
        log!(quiet, "Wrote output jar to {}", &output_filename);
    }

    Ok(())
}
