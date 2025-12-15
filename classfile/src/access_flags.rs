#![forbid(unsafe_code)]

#[repr(u16)]
#[derive(Copy, Clone)]
pub enum AccessFlag {
    Public = 0x0001,
    Private = 0x0002,
    Protected = 0x0004,
    Static = 0x0008,
    Final = 0x0010,
    Super = 0x0020,
    Interface = 0x0200,
    Abstract = 0x0400,
    Synthetic = 0x1000,
    Annotation = 0x2000,
    Enum = 0x4000,
    Module = 0x8000,
}

pub const ALL_CLASS_FLAGS: &[(AccessFlag, u16)] = &[
    (AccessFlag::Public, AccessFlag::Public as u16),
    (AccessFlag::Private, AccessFlag::Private as u16),
    (AccessFlag::Protected, AccessFlag::Protected as u16),
    (AccessFlag::Static, AccessFlag::Static as u16),
    (AccessFlag::Final, AccessFlag::Final as u16),
    (AccessFlag::Super, AccessFlag::Super as u16),
    (AccessFlag::Interface, AccessFlag::Interface as u16),
    (AccessFlag::Abstract, AccessFlag::Abstract as u16),
    (AccessFlag::Synthetic, AccessFlag::Synthetic as u16),
    (AccessFlag::Annotation, AccessFlag::Annotation as u16),
    (AccessFlag::Enum, AccessFlag::Enum as u16),
];

// TODO: Convert into a trait?
pub fn java_repr(flag: AccessFlag) -> String {
    match flag {
        AccessFlag::Public => "ACC_PUBLIC",
        AccessFlag::Private => "ACC_PRIVATE",
        AccessFlag::Protected => "ACC_PROTECTED",
        AccessFlag::Static => "ACC_STATIC",
        AccessFlag::Final => "ACC_FINAL",
        AccessFlag::Super => "ACC_SUPER",
        AccessFlag::Interface => "ACC_INTERFACE",
        AccessFlag::Abstract => "ACC_ABSTRACT",
        AccessFlag::Synthetic => "ACC_SYNTHETIC",
        AccessFlag::Annotation => "ACC_ANNOTATION",
        AccessFlag::Enum => "ACC_ENUM",
        AccessFlag::Module => "ACC_MODULE",
    }
    .to_string()
}

pub fn java_repr_vec(flags: &[AccessFlag]) -> String {
    flags
        .iter()
        .map(|f| java_repr(*f))
        .collect::<Vec<String>>()
        .join(", ")
}

// TODO: Convert into a trait?
pub fn modifier_repr(flag: AccessFlag) -> String {
    match flag {
        AccessFlag::Public => "public",
        AccessFlag::Private => "private",
        AccessFlag::Protected => "protected",
        AccessFlag::Static => "static",
        AccessFlag::Final => "final",
        AccessFlag::Super => "class",
        AccessFlag::Interface => "interface",
        AccessFlag::Abstract => "abstract",
        AccessFlag::Enum => "",
        AccessFlag::Synthetic => "",
        AccessFlag::Annotation => todo!(),
        AccessFlag::Module => todo!(),
    }
    .to_string()
}

pub fn modifier_repr_vec(flags: &[AccessFlag]) -> String {
    let mut result: String = String::new();
    for f in flags {
        let fs: String = modifier_repr(*f);
        if fs.is_empty() {
            continue;
        }
        result = result + " " + &fs;
    }
    result
}

pub fn to_u16(flags: &[AccessFlag]) -> u16 {
    flags
        .iter()
        .map(|f| *f as u16)
        .reduce(|a, b| a | b)
        .unwrap()
}

pub fn parse_access_flags(flags: u16) -> Vec<AccessFlag> {
    let mut result: Vec<AccessFlag> = Vec::new();
    for (f, mask) in ALL_CLASS_FLAGS {
        if (flags & mask) != 0u16 {
            result.push(*f);
        }
    }
    result
}
