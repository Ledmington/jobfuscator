#![forbid(unsafe_code)]

pub mod zip_parser;

#[derive(PartialEq)]
#[repr(u8)]
enum OS {
    MsDos = 0,
    AMIGA = 1,
    OpenVms = 2,
    Unix = 3,
    VmCms = 4,
    AtariSt = 5,
    Os2Hpfs = 6,
    Macintosh = 7,
    ZSystem = 8,
    CPM = 9,
    WindowsNtfs = 10,
    MVS = 11,
    VSE = 12,
    AcornRisc = 13,
    Vfat = 14,
    AlternateMvs = 15,
    Beos = 16,
    Tandem = 17,
    OS400 = 18,
    OsxDarwin = 19,
}

impl OS {
    pub fn description(&self) -> &'static str {
        match self {
            OS::MsDos => "MS-DOS and OS/2 (FAT / VFAT / FAT32 file systems)",
            OS::AMIGA => "Amiga",
            OS::OpenVms => "OpenVMS",
            OS::Unix => "UNIX",
            OS::VmCms => "VM/CMS",
            OS::AtariSt => "Atari ST",
            OS::Os2Hpfs => "OS/2 H.P.F.S.",
            OS::Macintosh => "Macintosh",
            OS::ZSystem => "Z-System",
            OS::CPM => "CP/M",
            OS::WindowsNtfs => "Windows NTFS",
            OS::MVS => "MVS (OS/390 - Z/OS)",
            OS::VSE => "VSE",
            OS::AcornRisc => "Acorn Risc",
            OS::Vfat => "VFAT",
            OS::AlternateMvs => "alternate MVS",
            OS::Beos => "BeOS",
            OS::Tandem => "Tandem",
            OS::OS400 => "OS/400",
            OS::OsxDarwin => "OS X (Darwin)",
        }
    }
}

impl TryFrom<u8> for OS {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OS::MsDos),
            1 => Ok(OS::AMIGA),
            2 => Ok(OS::OpenVms),
            3 => Ok(OS::Unix),
            4 => Ok(OS::VmCms),
            5 => Ok(OS::AtariSt),
            6 => Ok(OS::Os2Hpfs),
            7 => Ok(OS::Macintosh),
            8 => Ok(OS::ZSystem),
            9 => Ok(OS::CPM),
            10 => Ok(OS::WindowsNtfs),
            11 => Ok(OS::MVS),
            12 => Ok(OS::VSE),
            13 => Ok(OS::AcornRisc),
            14 => Ok(OS::Vfat),
            15 => Ok(OS::AlternateMvs),
            16 => Ok(OS::Beos),
            17 => Ok(OS::Tandem),
            18 => Ok(OS::OS400),
            19 => Ok(OS::OsxDarwin),
            _ => Err(format!("Unknown/unused OS id: {value} (0x{value:02x}).")),
        }
    }
}

#[derive(PartialEq)]
struct Version {
    os: OS,
    major: u32,
    minor: u32,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}.{}", self.os.description(), self.major, self.minor)
    }
}

#[derive(PartialEq)]
#[repr(u16)]
enum CompressionMethod {
    None = 0x0000,
    Deflate = 0x0008,
}

impl TryFrom<u16> for CompressionMethod {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0000 => Ok(Self::None),
            0x0008 => Ok(Self::Deflate),
            _ => Err(format!("Unknown compression method id: 0x{value:04x}.")),
        }
    }
}

impl std::fmt::Display for CompressionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionMethod::None => write!(f, "None"),
            CompressionMethod::Deflate => write!(f, "Deflate"),
        }
    }
}

#[derive(PartialEq)]
struct MsDosTime {
    hours: u16,
    minutes: u16,
    seconds: u16,
}

impl std::fmt::Display for MsDosTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}/{:02}/{:02}",
            self.hours,
            self.minutes,
            self.seconds * 2
        )
    }
}

#[derive(PartialEq)]
struct MsDosDate {
    year: u16,
    month: u16,
    day: u16,
}

impl std::fmt::Display for MsDosDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}:{}", self.day, self.month, self.year)
    }
}

#[derive(PartialEq)]
struct ExtraField {
    field_type: ExtraFieldType,
    data: Vec<u8>,
}

impl std::fmt::Display for ExtraField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut data_str = String::new();
        for x in self.data.iter() {
            data_str.push_str(&format!("{x:02x}"));
        }
        write!(
            f,
            "ExtraField {{ type : '{}', data : 0x{} }}",
            self.field_type.description(),
            data_str
        )
    }
}

#[derive(PartialEq)]
#[repr(u16)]
enum ExtraFieldType {
    ZIP64 = 0x0001,
    UnixTimestamp = 0x5455,
    WinZipAes = 0x9901,
    Java = 0xcafe,
}

impl ExtraFieldType {
    pub fn description(&self) -> &'static str {
        match self {
            ExtraFieldType::ZIP64 => "ZIP64 extension",
            ExtraFieldType::UnixTimestamp => "UNIX Timestamp",
            ExtraFieldType::WinZipAes => "WinZIP AES encryption",
            ExtraFieldType::Java => "Java",
        }
    }
}

impl TryFrom<u16> for ExtraFieldType {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0001 => Ok(ExtraFieldType::ZIP64),
            0x5455 => Ok(ExtraFieldType::UnixTimestamp),
            0x9901 => Ok(ExtraFieldType::WinZipAes),
            0xcafe => Ok(ExtraFieldType::Java),
            _ => Err(format!("Unknown extra field type id: 0x{value:04x}.")),
        }
    }
}

#[repr(u16)]
#[derive(Copy, Clone)]
enum BitFlag {
    Encrypted = 0x0001,
    HasDataDescriptor = 0x0008,
}

struct BitFlags(u16);

impl BitFlags {
    pub fn to_u16(&self) -> u16 {
        self.0
    }

    pub fn contains(&self, flag: &BitFlag) -> bool {
        (self.0 & (*flag as u16)) != 0
    }
}

impl TryFrom<u16> for BitFlags {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        const EXPECTED_MASK: u16 = 0b1101_0111_1000_0000u16;
        if (value & EXPECTED_MASK) != 0 {
            Err(format!(
                "Expected bit_flags to respect the mask 0x{:04x} but did not, was 0x{:04x}.",
                EXPECTED_MASK, value
            ))
        } else {
            Ok(BitFlags(value))
        }
    }
}

struct LocalFileHeader {
    minimum_version: Version,
    bit_flags: BitFlags,
    compression_method: CompressionMethod,
    last_modification_time: MsDosTime,
    last_modification_date: MsDosDate,
    compressed_size: u32,
    uncompressed_size: u32,
    filename: String,
    extra_fields: Vec<ExtraField>,
}

struct DataDescriptor {
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
}

struct CentralDirectoryRecord {
    version_made_by: Version,
    minimum_version: Version,
    bit_flags: BitFlags,
    compression_method: CompressionMethod,
    last_modification_time: MsDosTime,
    last_modification_date: MsDosDate,
    compressed_size: u32,
    uncompressed_size: u32,
    internal_file_attributes: u16,
    external_file_attributes: u32,
    local_file_header_offset: u32,
    filename: String,
    extra_fields: Vec<ExtraField>,
    file_comment: String,
}

struct EndOfCentralDirectoryRecord {
    total_central_directory_records: u16,
    central_directory_size: u32,
    central_directory_offset: u32,
    comment: String,
}

pub struct ZipFile {
    entries: Vec<ZipEntry>,
}

impl ZipFile {
    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn entries(&self) -> &Vec<ZipEntry> {
        &self.entries
    }
}

pub struct ZipEntry {
    version_made_by: Version,
    minimum_version: Version,
    bit_flags: BitFlags,
    compression_method: CompressionMethod,
    last_modification_time: MsDosTime,
    last_modification_date: MsDosDate,
    compressed_content: Vec<u8>,
    filename: String,
    comment: String,
}

impl ZipEntry {
    pub fn name(&self) -> &String {
        &self.filename
    }

    pub fn compressed_size(&self) -> usize {
        self.compressed_content.len()
    }

    pub fn version_made_by(&self) -> String {
        self.version_made_by.to_string()
    }

    pub fn minimum_version(&self) -> String {
        self.minimum_version.to_string()
    }

    pub fn bit_flags(&self) -> u16 {
        self.bit_flags.to_u16()
    }

    pub fn compression_method(&self) -> String {
        self.compression_method.to_string()
    }

    pub fn last_modification_date(&self) -> String {
        self.last_modification_date.to_string()
    }

    pub fn last_modification_time(&self) -> String {
        self.last_modification_time.to_string()
    }

    pub fn comment(&self) -> String {
        self.comment.clone()
    }
}
