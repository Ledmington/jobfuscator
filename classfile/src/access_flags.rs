#![forbid(unsafe_code)]

pub trait AccessFlag {
    fn java_repr(&self) -> &'static str;
    fn modifier_repr(&self) -> &'static str;
    fn as_u16(&self) -> u16;
}

#[repr(u16)]
#[derive(Copy, Clone, PartialEq)]
pub enum ClassAccessFlag {
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

impl AccessFlag for ClassAccessFlag {
    fn java_repr(&self) -> &'static str {
        match self {
            ClassAccessFlag::Public => "ACC_PUBLIC",
            ClassAccessFlag::Final => "ACC_FINAL",
            ClassAccessFlag::Super => "ACC_SUPER",
            ClassAccessFlag::Interface => todo!(),
            ClassAccessFlag::Abstract => todo!(),
            ClassAccessFlag::Synthetic => todo!(),
            ClassAccessFlag::Annotation => todo!(),
            ClassAccessFlag::Enum => "ACC_ENUM",
            ClassAccessFlag::Module => todo!(),
        }
    }

    fn modifier_repr(&self) -> &'static str {
        match self {
            ClassAccessFlag::Public => "public",
            ClassAccessFlag::Final => "final",
            ClassAccessFlag::Super => "class",
            ClassAccessFlag::Interface => todo!(),
            ClassAccessFlag::Abstract => todo!(),
            ClassAccessFlag::Synthetic => todo!(),
            ClassAccessFlag::Annotation => todo!(),
            ClassAccessFlag::Enum => "",
            ClassAccessFlag::Module => todo!(),
        }
    }

    fn as_u16(&self) -> u16 {
        *self as u16
    }
}

#[repr(u16)]
#[derive(Copy, Clone, PartialEq)]
pub enum InnerClassAccessFlag {
    Public = 0x0001,
    Private = 0x0002,
    Protected = 0x0004,
    Static = 0x0008,
    Final = 0x0010,
    Interface = 0x0200,
    Abstract = 0x0400,
    Synthetic = 0x1000,
    Annotation = 0x2000,
    Enum = 0x4000,
}

impl AccessFlag for InnerClassAccessFlag {
    fn java_repr(&self) -> &'static str {
        todo!()
    }

    fn modifier_repr(&self) -> &'static str {
        match self {
            InnerClassAccessFlag::Public => "public",
            InnerClassAccessFlag::Private => "private",
            InnerClassAccessFlag::Protected => todo!(),
            InnerClassAccessFlag::Static => "static",
            InnerClassAccessFlag::Final => "final",
            InnerClassAccessFlag::Interface => "interface",
            InnerClassAccessFlag::Abstract => "abstract",
            InnerClassAccessFlag::Synthetic => todo!(),
            InnerClassAccessFlag::Annotation => todo!(),
            InnerClassAccessFlag::Enum => "",
        }
    }

    fn as_u16(&self) -> u16 {
        *self as u16
    }
}

#[repr(u16)]
#[derive(Copy, Clone, PartialEq)]
pub enum FieldAccessFlag {
    Public = 0x0001,
    Private = 0x0002,
    Protected = 0x0004,
    Static = 0x0008,
    Final = 0x0010,
    Volatile = 0x0040,
    Transient = 0x0080,
    Synthetic = 0x1000,
    Enum = 0x4000,
}

impl AccessFlag for FieldAccessFlag {
    fn java_repr(&self) -> &'static str {
        match self {
            FieldAccessFlag::Public => "ACC_PUBLIC",
            FieldAccessFlag::Private => "ACC_PRIVATE",
            FieldAccessFlag::Protected => todo!(),
            FieldAccessFlag::Static => "ACC_STATIC",
            FieldAccessFlag::Final => "ACC_FINAL",
            FieldAccessFlag::Volatile => todo!(),
            FieldAccessFlag::Transient => todo!(),
            FieldAccessFlag::Synthetic => "ACC_SYNTHETIC",
            FieldAccessFlag::Enum => "ACC_ENUM",
        }
    }

    fn modifier_repr(&self) -> &'static str {
        match self {
            FieldAccessFlag::Public => "public",
            FieldAccessFlag::Private => "private",
            FieldAccessFlag::Protected => todo!(),
            FieldAccessFlag::Static => "static",
            FieldAccessFlag::Final => "final",
            FieldAccessFlag::Volatile => todo!(),
            FieldAccessFlag::Transient => todo!(),
            FieldAccessFlag::Synthetic => "",
            FieldAccessFlag::Enum => "",
        }
    }

    fn as_u16(&self) -> u16 {
        *self as u16
    }
}

#[repr(u16)]
#[derive(Copy, Clone, PartialEq)]
pub enum MethodAccessFlag {
    Public = 0x0001,
    Private = 0x0002,
    Protected = 0x0004,
    Static = 0x0008,
    Final = 0x0010,
    Synchronized = 0x0020,
    Bridge = 0x0040,
    Varargs = 0x0080,
    Native = 0x0100,
    Abstract = 0x0400,
    Strict = 0x0800,
    Synthetic = 0x1000,
}

impl AccessFlag for MethodAccessFlag {
    fn java_repr(&self) -> &'static str {
        match self {
            MethodAccessFlag::Public => "ACC_PUBLIC",
            MethodAccessFlag::Private => "ACC_PRIVATE",
            MethodAccessFlag::Protected => todo!(),
            MethodAccessFlag::Static => "ACC_STATIC",
            MethodAccessFlag::Final => "ACC_FINAL",
            MethodAccessFlag::Synchronized => todo!(),
            MethodAccessFlag::Bridge => todo!(),
            MethodAccessFlag::Varargs => "ACC_VARARGS",
            MethodAccessFlag::Native => todo!(),
            MethodAccessFlag::Abstract => todo!(),
            MethodAccessFlag::Strict => todo!(),
            MethodAccessFlag::Synthetic => "ACC_SYNTHETIC",
        }
    }

    fn modifier_repr(&self) -> &'static str {
        match self {
            MethodAccessFlag::Public => "public",
            MethodAccessFlag::Private => "private",
            MethodAccessFlag::Protected => todo!(),
            MethodAccessFlag::Static => "static",
            MethodAccessFlag::Final => "final",
            MethodAccessFlag::Synchronized => todo!(),
            MethodAccessFlag::Bridge => todo!(),
            MethodAccessFlag::Varargs => "varargs",
            MethodAccessFlag::Native => todo!(),
            MethodAccessFlag::Abstract => todo!(),
            MethodAccessFlag::Strict => todo!(),
            MethodAccessFlag::Synthetic => "",
        }
    }

    fn as_u16(&self) -> u16 {
        *self as u16
    }
}

#[repr(u16)]
#[derive(Copy, Clone, PartialEq)]
pub enum MethodParameterAccessFlag {
    Final = 0x0010,
    Synthetic = 0x1000,
    Mandated = 0x8000,
}

impl AccessFlag for MethodParameterAccessFlag {
    fn java_repr(&self) -> &'static str {
        todo!()
    }

    fn modifier_repr(&self) -> &'static str {
        match self {
            MethodParameterAccessFlag::Final => "final",
            MethodParameterAccessFlag::Synthetic => "synthetic",
            MethodParameterAccessFlag::Mandated => "mandated",
        }
    }

    fn as_u16(&self) -> u16 {
        *self as u16
    }
}

const ALL_CLASS_ACCESS_FLAGS: &[(ClassAccessFlag, u16)] = &[
    (ClassAccessFlag::Public, ClassAccessFlag::Public as u16),
    (ClassAccessFlag::Final, ClassAccessFlag::Final as u16),
    (ClassAccessFlag::Super, ClassAccessFlag::Super as u16),
    (
        ClassAccessFlag::Interface,
        ClassAccessFlag::Interface as u16,
    ),
    (ClassAccessFlag::Abstract, ClassAccessFlag::Abstract as u16),
    (
        ClassAccessFlag::Synthetic,
        ClassAccessFlag::Synthetic as u16,
    ),
    (
        ClassAccessFlag::Annotation,
        ClassAccessFlag::Annotation as u16,
    ),
    (ClassAccessFlag::Enum, ClassAccessFlag::Enum as u16),
    (ClassAccessFlag::Module, ClassAccessFlag::Module as u16),
];

const CLASS_ACCESS_FLAGS_MASK: u16 = 0xf631;

const ALL_INNER_CLASS_ACCESS_FLAGS: &[(InnerClassAccessFlag, u16)] = &[
    (
        InnerClassAccessFlag::Public,
        InnerClassAccessFlag::Public as u16,
    ),
    (
        InnerClassAccessFlag::Private,
        InnerClassAccessFlag::Private as u16,
    ),
    (
        InnerClassAccessFlag::Protected,
        InnerClassAccessFlag::Protected as u16,
    ),
    (
        InnerClassAccessFlag::Static,
        InnerClassAccessFlag::Static as u16,
    ),
    (
        InnerClassAccessFlag::Final,
        InnerClassAccessFlag::Final as u16,
    ),
    (
        InnerClassAccessFlag::Interface,
        InnerClassAccessFlag::Interface as u16,
    ),
    (
        InnerClassAccessFlag::Abstract,
        InnerClassAccessFlag::Abstract as u16,
    ),
    (
        InnerClassAccessFlag::Synthetic,
        InnerClassAccessFlag::Synthetic as u16,
    ),
    (
        InnerClassAccessFlag::Annotation,
        InnerClassAccessFlag::Annotation as u16,
    ),
    (
        InnerClassAccessFlag::Enum,
        InnerClassAccessFlag::Enum as u16,
    ),
];

const INNER_CLASS_ACCESS_FLAGS_MASK: u16 = 0x761f;

const ALL_FIELD_ACCESS_FLAGS: &[(FieldAccessFlag, u16)] = &[
    (FieldAccessFlag::Public, FieldAccessFlag::Public as u16),
    (FieldAccessFlag::Private, FieldAccessFlag::Private as u16),
    (
        FieldAccessFlag::Protected,
        FieldAccessFlag::Protected as u16,
    ),
    (FieldAccessFlag::Static, FieldAccessFlag::Static as u16),
    (FieldAccessFlag::Final, FieldAccessFlag::Final as u16),
    (FieldAccessFlag::Volatile, FieldAccessFlag::Volatile as u16),
    (
        FieldAccessFlag::Transient,
        FieldAccessFlag::Transient as u16,
    ),
    (
        FieldAccessFlag::Synthetic,
        FieldAccessFlag::Synthetic as u16,
    ),
    (FieldAccessFlag::Enum, FieldAccessFlag::Enum as u16),
];

const FIELD_ACCESS_FLAGS_MASK: u16 = 0x50df;

const ALL_METHOD_ACCESS_FLAGS: &[(MethodAccessFlag, u16)] = &[
    (MethodAccessFlag::Public, MethodAccessFlag::Public as u16),
    (MethodAccessFlag::Private, MethodAccessFlag::Private as u16),
    (
        MethodAccessFlag::Protected,
        MethodAccessFlag::Protected as u16,
    ),
    (MethodAccessFlag::Static, MethodAccessFlag::Static as u16),
    (MethodAccessFlag::Final, MethodAccessFlag::Final as u16),
    (
        MethodAccessFlag::Synchronized,
        MethodAccessFlag::Synchronized as u16,
    ),
    (MethodAccessFlag::Bridge, MethodAccessFlag::Bridge as u16),
    (MethodAccessFlag::Varargs, MethodAccessFlag::Varargs as u16),
    (MethodAccessFlag::Native, MethodAccessFlag::Native as u16),
    (
        MethodAccessFlag::Abstract,
        MethodAccessFlag::Abstract as u16,
    ),
    (MethodAccessFlag::Strict, MethodAccessFlag::Strict as u16),
    (
        MethodAccessFlag::Synthetic,
        MethodAccessFlag::Synthetic as u16,
    ),
];

const METHOD_ACCESS_FLAGS_MASK: u16 = 0x1dff;

const ALL_METHOD_PARAMETER_ACCESS_FLAGS: &[(MethodParameterAccessFlag, u16)] = &[
    (
        MethodParameterAccessFlag::Final,
        MethodParameterAccessFlag::Final as u16,
    ),
    (
        MethodParameterAccessFlag::Synthetic,
        MethodParameterAccessFlag::Synthetic as u16,
    ),
    (
        MethodParameterAccessFlag::Mandated,
        MethodParameterAccessFlag::Mandated as u16,
    ),
];

const METHOD_PARAMETER_ACCESS_FLAGS_MASK: u16 = 0x9010;

pub fn parse_class_access_flags(flags: u16) -> Vec<ClassAccessFlag> {
    if (flags & !CLASS_ACCESS_FLAGS_MASK) != 0 {
        panic!(
            "{} is not a valid combination of class access flags.",
            flags
        );
    }

    let mut result: Vec<ClassAccessFlag> = Vec::new();
    for (f, mask) in ALL_CLASS_ACCESS_FLAGS.iter() {
        if (flags & mask) != 0u16 {
            result.push(*f);
        }
    }
    result
}

pub fn parse_inner_class_access_flags(flags: u16) -> Vec<InnerClassAccessFlag> {
    if (flags & !INNER_CLASS_ACCESS_FLAGS_MASK) != 0 {
        panic!(
            "{} is not a valid combination of inner class access flags.",
            flags
        );
    }

    let mut result: Vec<InnerClassAccessFlag> = Vec::new();
    for (f, mask) in ALL_INNER_CLASS_ACCESS_FLAGS.iter() {
        if (flags & mask) != 0u16 {
            result.push(*f);
        }
    }
    result
}

pub fn parse_field_access_flags(flags: u16) -> Vec<FieldAccessFlag> {
    if (flags & !FIELD_ACCESS_FLAGS_MASK) != 0 {
        panic!(
            "{} is not a valid combination of field access flags.",
            flags
        );
    }

    let mut result: Vec<FieldAccessFlag> = Vec::new();
    for (f, mask) in ALL_FIELD_ACCESS_FLAGS.iter() {
        if (flags & mask) != 0u16 {
            result.push(*f);
        }
    }
    result
}

pub fn parse_method_access_flags(flags: u16) -> Vec<MethodAccessFlag> {
    if (flags & !METHOD_ACCESS_FLAGS_MASK) != 0 {
        panic!(
            "{} is not a valid combination of method access flags.",
            flags
        );
    }

    let mut result: Vec<MethodAccessFlag> = Vec::new();
    for (f, mask) in ALL_METHOD_ACCESS_FLAGS.iter() {
        if (flags & mask) != 0u16 {
            result.push(*f);
        }
    }
    result
}

pub fn parse_method_parameter_access_flags(flags: u16) -> Vec<MethodParameterAccessFlag> {
    if (flags & !METHOD_PARAMETER_ACCESS_FLAGS_MASK) != 0 {
        panic!(
            "{} is not a valid combination of method parameter access flags.",
            flags
        );
    }

    let mut result: Vec<MethodParameterAccessFlag> = Vec::new();
    for (f, mask) in ALL_METHOD_PARAMETER_ACCESS_FLAGS.iter() {
        if (flags & mask) != 0u16 {
            result.push(*f);
        }
    }
    result
}

pub fn to_u16<T: AccessFlag>(flags: &[T]) -> u16 {
    flags.iter().fold(0, |acc, x| acc | x.as_u16())
}

pub fn java_repr_vec<T: AccessFlag>(flags: &[T]) -> String {
    flags
        .iter()
        .map(|f| f.java_repr())
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn modifier_repr_vec<T: AccessFlag>(flags: &[T]) -> String {
    flags
        .iter()
        .map(|f| f.modifier_repr())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
