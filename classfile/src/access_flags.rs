#[repr(u16)]
#[derive(Copy, Clone)]
pub enum AccessFlag {
    Public = 0x0001,
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

// TODO: Convert into a trait?
pub fn modifier_repr(flag: AccessFlag) -> String {
    match flag {
        AccessFlag::Public => "public",
        AccessFlag::Final => "final",
        AccessFlag::Super => "class",
        AccessFlag::Interface => "interface",
        AccessFlag::Abstract => "abstract",
        AccessFlag::Enum => "enum",
        _ => "unknown",
    }
    .to_string()
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
