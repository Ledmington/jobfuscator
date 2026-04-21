#![forbid(unsafe_code)]

pub mod zip_parser;

pub struct ZipFile {}

#[repr(u8)]
enum OS {
    MsDos = 0x00,
    AMIGA = 0x01,
    OPENVMS = 0x02,
    UNIX = 0x03,
    VmCms = 0x04,
    AtariSt = 0x05,
    Os2Hpfs = 0x06,
    MACINTOSH = 0x07,
    ZSystem = 0x08,
    CPM = 0x09,
    WindowsNtfs = 0x0a,
    MVS = 0x0b,
    VSE = 0x0c,
    AcornRisc = 0x0d,
    VFAT = 0x0e,
    AlternateMvs = 0x0f,
    BEOS = 0x10,
    TANDEM = 0x11,
    OS400 = 0x12,
    OsxDarwin = 0x13,
}

// impl OS {
//     pub fn description(&self) -> &'static str {
//         match self {
//             OS::MsDos => "MS-DOS and OS/2 (FAT / VFAT / FAT32 file systems)",
//             OS::AMIGA => "Amiga",
//             OS::OPENVMS => "OpenVMS",
//             OS::UNIX => "UNIX",
//             OS::VmCms => "VM/CMS",
//             OS::AtariSt => "Atari ST",
//             OS::Os2Hpfs => "OS/2 H.P.F.S.",
//             OS::MACINTOSH => "Macintosh",
//             OS::ZSystem => "Z-System",
//             OS::CPM => "CP/M",
//             OS::WindowsNtfs => "Windows NTFS",
//             OS::MVS => "MVS (OS/390 - Z/OS)",
//             OS::VSE => "VSE",
//             OS::AcornRisc => "Acorn Risc",
//             OS::VFAT => "VFAT",
//             OS::AlternateMvs => "alternate MVS",
//             OS::BEOS => "BeOS",
//             OS::TANDEM => "Tandem",
//             OS::OS400 => "OS/400",
//             OS::OsxDarwin => "OS X (Darwin)",
//         }
//     }
// }

impl TryFrom<u8> for OS {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(OS::MsDos),
            0x01 => Ok(OS::AMIGA),
            0x02 => Ok(OS::OPENVMS),
            0x03 => Ok(OS::UNIX),
            0x04 => Ok(OS::VmCms),
            0x05 => Ok(OS::AtariSt),
            0x06 => Ok(OS::Os2Hpfs),
            0x07 => Ok(OS::MACINTOSH),
            0x08 => Ok(OS::ZSystem),
            0x09 => Ok(OS::CPM),
            0x0a => Ok(OS::WindowsNtfs),
            0x0b => Ok(OS::MVS),
            0x0c => Ok(OS::VSE),
            0x0d => Ok(OS::AcornRisc),
            0x0e => Ok(OS::VFAT),
            0x0f => Ok(OS::AlternateMvs),
            0x10 => Ok(OS::BEOS),
            0x11 => Ok(OS::TANDEM),
            0x12 => Ok(OS::OS400),
            0x13 => Ok(OS::OsxDarwin),
            _ => Err(format!("Unknown OS id: 0x{:02x}.", value)),
        }
    }
}

struct Version {
    os: OS,
    major: u32,
    minor: u32,
}

#[repr(u16)]
enum CompressionMethod {
    NONE = 0x0000,
    DEFLATE = 0x0008,
}

impl TryFrom<u16> for CompressionMethod {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0000 => Ok(Self::NONE),
            0x0008 => Ok(Self::DEFLATE),
            _ => Err(format!("Unknown compression method id: 0x{:04x}.", value)),
        }
    }
}

struct MsDosTime {
    hours: u16,
    minutes: u16,
    seconds: u16,
}

struct MsDosDate {
    year: u16,
    month: u16,
    day: u16,
}

struct ExtraField {
    field_type: ExtraFieldType,
    data: Vec<u8>,
}

#[repr(u16)]
enum ExtraFieldType {
    ZIP64 = 0x0001,
    UnixTimestamp = 0x5455,
    WinZipAes = 0x9901,
    JAVA = 0xcafe,
}

// impl ExtraFieldType {
//     pub fn description(&self) -> &'static str {
//         match self {
//             ExtraFieldType::ZIP64 => "ZIP64 extension",
//             ExtraFieldType::UnixTimestamp => "UNIX Timestamp",
//             ExtraFieldType::WinZipAes => "WinZIP AES encryption",
//             ExtraFieldType::JAVA => "Java",
//         }
//     }
// }

impl TryFrom<u16> for ExtraFieldType {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0001 => Ok(ExtraFieldType::ZIP64),
            0x5455 => Ok(ExtraFieldType::UnixTimestamp),
            0x9901 => Ok(ExtraFieldType::WinZipAes),
            0xcafe => Ok(ExtraFieldType::JAVA),
            _ => Err(format!("Unknown extra field type id: 0x{:04x}.", value)),
        }
    }
}

struct CentralDirectoryRecord {
    version_made_by: Version,
    minimum_version: Version,
    bit_flags: u16,
    compression_method: CompressionMethod,
    last_modification_time: MsDosTime,
    last_modification_date: MsDosDate,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    disk_where_file_starts: u16,
    internal_file_attributes: u16,
    external_file_attributes: u32,
    local_file_header_offset: u32,
    filename: String,
    extra_fields: Vec<ExtraField>,
    file_comment: String,
}

struct EndOfCentralDirectoryRecord {
    disk_number: u16,
    disk_of_central_directory: u16,
    n_central_directory_records_on_this_disk: u16,
    total_central_directory_records: u16,
    central_directory_size: u32,
    central_directory_offset: u32,
    comment: String,
}
