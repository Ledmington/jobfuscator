#![forbid(unsafe_code)]

pub mod zip_parser;

#[derive(PartialEq)]
#[repr(u8)]
enum OS {
    MsDos = 0x00,
    AMIGA = 0x01,
    OpenVms = 0x02,
    Unix = 0x03,
    VmCms = 0x04,
    AtariSt = 0x05,
    Os2Hpfs = 0x06,
    Macintosh = 0x07,
    ZSystem = 0x08,
    CPM = 0x09,
    WindowsNtfs = 0x0a,
    MVS = 0x0b,
    VSE = 0x0c,
    AcornRisc = 0x0d,
    Vfat = 0x0e,
    AlternateMvs = 0x0f,
    Beos = 0x10,
    Tandem = 0x11,
    OS400 = 0x12,
    OsxDarwin = 0x13,
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
            0x00 => Ok(OS::MsDos),
            0x01 => Ok(OS::AMIGA),
            0x02 => Ok(OS::OpenVms),
            0x03 => Ok(OS::Unix),
            0x04 => Ok(OS::VmCms),
            0x05 => Ok(OS::AtariSt),
            0x06 => Ok(OS::Os2Hpfs),
            0x07 => Ok(OS::Macintosh),
            0x08 => Ok(OS::ZSystem),
            0x09 => Ok(OS::CPM),
            0x0a => Ok(OS::WindowsNtfs),
            0x0b => Ok(OS::MVS),
            0x0c => Ok(OS::VSE),
            0x0d => Ok(OS::AcornRisc),
            0x0e => Ok(OS::Vfat),
            0x0f => Ok(OS::AlternateMvs),
            0x10 => Ok(OS::Beos),
            0x11 => Ok(OS::Tandem),
            0x12 => Ok(OS::OS400),
            0x13 => Ok(OS::OsxDarwin),
            _ => Err(format!("Unknown OS id: 0x{value:02x}.")),
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
        write!(f, "{:02}:{:02}:{}", self.day, self.month, self.year + 1980)
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

struct LocalFileHeader {
    minimum_version: Version,
    bit_flags: u16,
    compression_method: CompressionMethod,
    last_modification_time: MsDosTime,
    last_modification_date: MsDosDate,
    compressed_size: u32,
    uncompressed_size: u32,
    filename: String,
    extra_fields: Vec<ExtraField>,
}

struct CentralDirectoryRecord {
    version_made_by: Version,
    minimum_version: Version,
    bit_flags: u16,
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
    pub fn entries(&self) -> &Vec<ZipEntry> {
        &self.entries
    }
}

pub struct ZipEntry {
    filename: String,
    compressed_content: Vec<u8>,
}

impl ZipEntry {
    pub fn name(&self) -> &String {
        &self.filename
    }

    pub fn compressed_size(&self) -> usize {
        self.compressed_content.len()
    }
}
