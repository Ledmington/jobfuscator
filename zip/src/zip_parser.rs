#![forbid(unsafe_code)]

use std::{
    cmp::max,
    fs::File,
    io::Read,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use binary_reader::bit_reader::BitReader;

use crate::{
    CentralDirectoryRecord, CompressionMethod, EndOfCentralDirectoryRecord, ExtraField,
    ExtraFieldType, LocalFileHeader, MsDosDate, MsDosTime, OS, Version, ZipEntry, ZipFile,
};

// Useful reference: https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT

pub fn parse_zip(filename: &str) -> ZipFile {
    let mut file = File::open(filename)
        .unwrap_or_else(|err| panic!("Could not open file '{}' due to: {}.", filename, err));
    let mut file_bytes = Vec::new();
    file.read_to_end(&mut file_bytes)
        .unwrap_or_else(|err| panic!("Could not read file '{}' due to: {}.", filename, err));
    parse_zip_buf(&mut BitReader::new(&file_bytes))
}

fn parse_local_file_header(reader: &mut BitReader) -> LocalFileHeader {
    {
        const EXPECTED_SIGNATURE: u32 = 0x04034b50;
        let signature = reader.read_u32();
        if signature != EXPECTED_SIGNATURE {
            panic!(
                "Wrong Local File Header signature: expected 0x{:08x} but was 0x{:08x}.",
                EXPECTED_SIGNATURE, signature
            );
        }
    }

    let minimum_version = parse_version(reader);

    let bit_flags = reader.read_u16();

    let compression_method = CompressionMethod::try_from(reader.read_u16())
        .unwrap_or_else(|err| panic!("Error during parsing of compression method: {}.", err));

    let last_modification_time = parse_time(reader);
    let last_modification_date = parse_date(reader);
    assert_not_in_the_future(&last_modification_date, &last_modification_time);

    // TODO: check CRC32
    let crc32 = reader.read_u32();
    if crc32 != 0 {
        panic!("Invalid CRC32: 0x{:08x}.", crc32);
    }

    let compressed_size = reader.read_u32();
    if compressed_size != 0 {
        panic!("Invalid compressed size: {} bytes.", compressed_size);
    }

    let uncompressed_size = reader.read_u32();
    if uncompressed_size != 0 {
        panic!("Invalid uncompressed size: {} bytes.", uncompressed_size);
    }

    if matches!(compression_method, CompressionMethod::NONE) && compressed_size != uncompressed_size
    {
        panic!(
            "Compression method was NONE but compressed size ({} bytes) and uncompressed size ({} bytes) were different.",
            compressed_size, uncompressed_size
        );
    }

    let file_name_length = reader.read_u16();

    let extra_field_length = reader.read_u16();

    let mut filename = String::new();
    for _ in 0..file_name_length {
        filename.push(reader.read_u8() as char);
    }

    let extra_fields: Vec<ExtraField> = parse_extra_fields(reader, extra_field_length);

    LocalFileHeader {
        minimum_version,
        bit_flags,
        compression_method,
        last_modification_time,
        last_modification_date,
        compressed_size,
        uncompressed_size,
        filename,
        extra_fields,
    }
}

fn check_local_file_header(cdr: &CentralDirectoryRecord, lfh: &LocalFileHeader) {
    if cdr.minimum_version != lfh.minimum_version {
        panic!(
            "Different minimum versions in CDR ({}) and LFH ({}).",
            cdr.minimum_version, lfh.minimum_version
        );
    }
    if cdr.bit_flags != lfh.bit_flags {
        panic!(
            "Different bit flags in CDR (0x{:04x}) and LFH (0x{:04x}).",
            cdr.bit_flags, lfh.bit_flags
        );
    }
    if cdr.compression_method != lfh.compression_method {
        panic!(
            "Different compression methods in CDR ({}) and LFH ({}).",
            cdr.compression_method, lfh.compression_method
        );
    }
    if cdr.last_modification_time != lfh.last_modification_time {
        panic!(
            "Different last modification times in CDR ({}) and LFH ({}).",
            cdr.last_modification_time, lfh.last_modification_time
        );
    }
    if cdr.last_modification_date != lfh.last_modification_date {
        panic!(
            "Different last modification dates in CDR ({}) and LFH ({}).",
            cdr.last_modification_date, lfh.last_modification_date
        );
    }
    if cdr.filename != lfh.filename {
        panic!(
            "Different filenames in CDR ('{}') and LFH ('{}').",
            cdr.filename, lfh.filename
        );
    }
    if cdr.extra_fields.len() != lfh.extra_fields.len() {
        panic!(
            "Different number of extra fields in CDR ({}) and LFH ({}).",
            cdr.extra_fields.len(),
            lfh.extra_fields.len()
        );
    }
    for i in 0..cdr.extra_fields.len() {
        if cdr.extra_fields[i] != lfh.extra_fields[i] {
            panic!(
                "Different extra field at index {} in CDR ({}) and LFH ({}).",
                i, cdr.extra_fields[i], lfh.extra_fields[i]
            );
        }
    }
}

fn parse_zip_buf(reader: &mut BitReader) -> ZipFile {
    let eocdr = parse_end_of_central_directory_record(reader);

    reader.set_byte_position(eocdr.central_directory_offset as usize);
    let mut central_directory: Vec<CentralDirectoryRecord> =
        Vec::with_capacity(eocdr.total_central_directory_records as usize);
    for _ in 0..eocdr.total_central_directory_records {
        central_directory.push(parse_central_directory_record(reader));
    }

    {
        // check that the central directory size is correct
        let pos: u32 = reader.get_byte_position() as u32;
        let actual_central_directory_size: u32 = pos - eocdr.central_directory_offset;
        if eocdr.central_directory_size != actual_central_directory_size {
            panic!(
                "Wrong Central Directory size: expected {} bytes but was {} bytes.",
                eocdr.central_directory_size, actual_central_directory_size
            );
        }
    }

    let mut entries: Vec<ZipEntry> = Vec::with_capacity(central_directory.len());
    for cdr in central_directory {
        reader.set_byte_position(cdr.local_file_header_offset as usize);
        let lfh = parse_local_file_header(reader);

        check_local_file_header(&cdr, &lfh);

        let compressed = reader.read_u8_vec(cdr.compressed_size as usize);

        entries.push(ZipEntry {
            filename: cdr.filename,
        });
    }

    ZipFile { entries }
}

fn parse_version(reader: &mut BitReader) -> Version {
    let id = reader.read_u16();
    let os = OS::try_from((id >> 8) as u8)
        .unwrap_or_else(|err| panic!("Error during parsing of OS: {}.", err));
    let major: u32 = ((id & 0x00ffu16) / 10) as u32;
    let minor: u32 = ((id & 0x00ffu16) % 10) as u32;
    Version { os, major, minor }
}

fn parse_time(reader: &mut BitReader) -> MsDosTime {
    let time = reader.read_u16();

    // Source: https://www.delorie.com/djgpp/doc/rbinter/it/65/16.html
    let hours: u16 = time >> 11;
    let minutes: u16 = (time & 0x07e0u16) >> 5;
    let seconds: u16 = 2 * (time & 0x001fu16);
    MsDosTime {
        hours,
        minutes,
        seconds,
    }
}

fn parse_date(reader: &mut BitReader) -> MsDosDate {
    let date = reader.read_u16();

    // Source: https://www.delorie.com/djgpp/doc/rbinter/it/66/16.html
    let year: u16 = 1980 + (date >> 9);
    let month: u16 = (date & 0x01e0u16) >> 5;
    let day: u16 = date & 0x001fu16;
    MsDosDate { day, month, year }
}

fn to_system_time(date: &MsDosDate, time: &MsDosTime) -> SystemTime {
    let days = days_from_civil(date.year as i32, date.month as i32, date.day as i32);

    let secs = days * 86400
        + (time.hours as i64) * 3600
        + (time.minutes as i64) * 60
        + (time.seconds as i64);

    UNIX_EPOCH + Duration::from_secs(secs as u64)
}

// same as before
fn days_from_civil(y: i32, m: i32, d: i32) -> i64 {
    let y = y - (m <= 2) as i32;
    let era = (y as i64).div_euclid(400);
    let yoe = (y as i64).rem_euclid(400);
    let doy = (153 * (m as i64 + if m > 2 { -3 } else { 9 }) + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn assert_not_in_the_future(date: &MsDosDate, time: &MsDosTime) {
    let then = to_system_time(date, time);
    let now = SystemTime::now();

    if then > now {
        panic!(
            "Last modification date+time is in the future: {} {}.",
            date, time
        );
    }
}

fn parse_extra_fields(reader: &mut BitReader, num_extra_fields: u16) -> Vec<ExtraField> {
    let mut ef: Vec<ExtraField> = Vec::with_capacity(num_extra_fields as usize);
    let initial_position = reader.get_byte_position();
    // why?
    while reader.get_byte_position() < initial_position + (num_extra_fields as usize) {
        let header_id = reader.read_u16();
        let field_type = ExtraFieldType::try_from(header_id)
            .unwrap_or_else(|err| panic!("Error during parsing of extra field type: {}.", err));

        let data_length = reader.read_u16();
        let data: Vec<u8> = reader.read_u8_vec(data_length as usize);
        ef.push(ExtraField { field_type, data });
    }

    return ef;
}

fn parse_central_directory_record(reader: &mut BitReader) -> CentralDirectoryRecord {
    {
        const EXPECTED_SIGNATURE: u32 = 0x02014b50;
        let signature = reader.read_u32();
        if signature != EXPECTED_SIGNATURE {
            panic!(
                "Wrong Central Directory Record signature: expected 0x{:08x} but was 0x{:08x}.",
                EXPECTED_SIGNATURE, signature
            );
        }
    }

    let version_made_by = parse_version(reader);
    let minimum_version = parse_version(reader);

    let bit_flags = reader.read_u16();

    let compression_method = CompressionMethod::try_from(reader.read_u16())
        .unwrap_or_else(|err| panic!("Error during parsing of compression method id: {}.", err));

    let last_modification_time = parse_time(reader);
    let last_modification_date = parse_date(reader);
    assert_not_in_the_future(&last_modification_date, &last_modification_time);

    // TODO: check CRC32
    let crc32 = reader.read_u32();
    if crc32 != 0 {
        panic!("Invalid CRC32: 0x{:08x}.", crc32);
    }

    let compressed_size = reader.read_u32();
    let uncompressed_size = reader.read_u32();

    if matches!(compression_method, CompressionMethod::NONE) && compressed_size != uncompressed_size
    {
        panic!(
            "Compression method was NONE but compressed size ({} bytes) and uncompressed size ({} bytes) were different.",
            compressed_size, uncompressed_size
        );
    }

    let file_name_length = reader.read_u16();
    let extra_field_length = reader.read_u16();
    let file_comment_length = reader.read_u16();

    let disk_where_file_starts = reader.read_u16();
    if disk_where_file_starts != 0 {
        panic!(
            "Don't know what to do when when file is not on disk 0: was {} (0x{:04x}).",
            disk_where_file_starts, disk_where_file_starts
        );
    }

    let internal_file_attributes = reader.read_u16();
    let external_file_attributes = reader.read_u32();

    let local_file_header_offset = reader.read_u32();
    if (local_file_header_offset as usize) >= reader.size() {
        panic!(
            "Invalid local file header offset: {} bytes (0x{:08x}).",
            local_file_header_offset, local_file_header_offset
        );
    }

    let mut filename = String::new();
    for _ in 0..file_name_length {
        filename.push(reader.read_u8() as char);
    }

    let extra_fields: Vec<ExtraField> = parse_extra_fields(reader, extra_field_length);

    let mut file_comment = String::new();
    for _ in 0..file_comment_length {
        file_comment.push(reader.read_u8() as char);
    }

    CentralDirectoryRecord {
        version_made_by,
        minimum_version,
        bit_flags,
        compression_method,
        last_modification_time,
        last_modification_date,
        compressed_size,
        uncompressed_size,
        internal_file_attributes,
        external_file_attributes,
        local_file_header_offset,
        filename,
        extra_fields,
        file_comment,
    }
}

fn parse_end_of_central_directory_record(reader: &mut BitReader) -> EndOfCentralDirectoryRecord {
    {
        /*
         * We know that the End of Central Directory Record (EOCDR) is always at the end
         * of the ZIP file and can be of any length between 22 and 65557 bytes (both
         * ends included), depending on the length of the comment field which is
         * indicated by a 2-bytes unsigned integer.
         * So, to find the start of EOCD (the signature 0x06054b50) we start at the byte
         * 65536 bytes from the end and scan forward.
         */
        reader.set_byte_position(max(0, reader.size() - 65_536));
        const EXPECTED_SIGNATURE: u32 = 0x06054b50;
        let mut found = false;
        while reader.get_byte_position() < reader.size() {
            let signature = reader.read_u32();
            if signature == EXPECTED_SIGNATURE {
                found = true;
                break;
            }
            reader.set_byte_position(reader.get_byte_position() - 3);
        }

        if !found {
            panic!(
                "EOCDR signature 0x{:08x} not found (maybe this is not a valid ZIP file?).",
                EXPECTED_SIGNATURE
            );
        }
    }

    let disk_number = reader.read_u16();
    if disk_number != 0 {
        panic!(
            "Don't know what to do when disk is not 0: was {} (0x{:04x}).",
            disk_number, disk_number
        );
    }

    let disk_of_central_directory = reader.read_u16();
    if disk_of_central_directory != 0 {
        panic!(
            "Don't know what to do when disk of Central Directory is not 0: was {} (0x{:04x}).",
            disk_of_central_directory, disk_of_central_directory
        );
    }

    let n_central_directory_records_on_this_disk = reader.read_u16();

    let total_central_directory_records = reader.read_u16();

    if n_central_directory_records_on_this_disk != total_central_directory_records {
        panic!(
            "Don't know what to do when number of CDRs on this disk ({}) is different from total number of CDRs ({}).",
            n_central_directory_records_on_this_disk, total_central_directory_records
        );
    }

    let central_directory_size = reader.read_u32();

    let central_directory_offset = reader.read_u32();

    let comment_length = reader.read_u16();
    let mut comment = String::new();
    for _ in 0..comment_length {
        comment.push(reader.read_u8() as char);
    }

    EndOfCentralDirectoryRecord {
        total_central_directory_records,
        central_directory_size,
        central_directory_offset,
        comment,
    }
}
