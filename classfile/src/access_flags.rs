#![forbid(unsafe_code)]

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

pub struct ClassAccessFlags(u16);

impl ClassAccessFlags {
    pub fn to_u16(&self) -> u16 {
        self.0
    }

    pub fn contains(&self, flag: ClassAccessFlag) -> bool {
        (self.0 & (flag as u16)) != 0
    }

    pub fn java_repr(&self) -> String {
        let parts: Vec<&str> = [
            (ClassAccessFlag::Public, "ACC_PUBLIC"),
            (ClassAccessFlag::Final, "ACC_FINAL"),
            (ClassAccessFlag::Super, "ACC_SUPER"),
            (ClassAccessFlag::Interface, "ACC_INTERFACE"),
            (ClassAccessFlag::Abstract, "ACC_ABSTRACT"),
            (ClassAccessFlag::Synthetic, "ACC_SYNTHETIC"),
            (ClassAccessFlag::Annotation, "ACC_ANNOTATION"),
            (ClassAccessFlag::Enum, "ACC_ENUM"),
            (ClassAccessFlag::Module, "ACC_MODULE"),
        ]
        .iter()
        .filter(|(flag, _)| self.contains(*flag))
        .map(|(_, repr)| *repr)
        .collect();
        parts.join(", ")
    }

    pub fn modifier_repr(&self) -> String {
        let parts: Vec<&str> = [
            (ClassAccessFlag::Public, "public"),
            (ClassAccessFlag::Final, "final"),
            (ClassAccessFlag::Super, "class"),
            (ClassAccessFlag::Interface, "interface"),
            (ClassAccessFlag::Abstract, ""),
            (ClassAccessFlag::Synthetic, "synthetic"),
            (ClassAccessFlag::Annotation, "annotation"),
            (ClassAccessFlag::Enum, "enum"),
            (ClassAccessFlag::Module, "module"),
        ]
        .iter()
        .filter(|(flag, repr)| self.contains(*flag) && !repr.is_empty())
        .map(|(_, repr)| *repr)
        .collect();
        parts.join(" ")
    }
}

impl From<u16> for ClassAccessFlags {
    fn from(flags: u16) -> Self {
        assert!(
            (flags & !0xf631) == 0,
            "0x{flags:04x} is not a valid combination of class access flags.",
        );
        ClassAccessFlags(flags)
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

pub struct InnerClassAccessFlags(u16);

impl InnerClassAccessFlags {
    pub fn to_u16(&self) -> u16 {
        self.0
    }

    pub fn contains(&self, flag: InnerClassAccessFlag) -> bool {
        (self.0 & flag as u16) != 0
    }

    pub fn modifier_repr(&self) -> String {
        [
            (InnerClassAccessFlag::Public, "public"),
            (InnerClassAccessFlag::Private, "private"),
            (InnerClassAccessFlag::Protected, "protected"),
            (InnerClassAccessFlag::Static, "static"),
            (InnerClassAccessFlag::Final, "final"),
            (InnerClassAccessFlag::Interface, ""),
            (InnerClassAccessFlag::Abstract, "abstract"),
            (InnerClassAccessFlag::Synthetic, "synthetic"),
            (InnerClassAccessFlag::Annotation, "annotation"),
            (InnerClassAccessFlag::Enum, ""),
        ]
        .iter()
        .filter(|(flag, repr)| {
            self.contains(*flag)
                && !repr.is_empty()
                && 
                // This weird condition is taken directly from the original javap source code:
                // https://github.com/openjdk/jdk/blob/0dd0108c1a7b3658df536adbc2bd68fa5167539d/src/jdk.jdeps/share/classes/com/sun/tools/javap/AttributeWriter.java#L222
                !(*flag == InnerClassAccessFlag::Abstract
                    && self.contains(InnerClassAccessFlag::Interface))
        })
        .map(|(_, repr)| *repr)
        .collect::<Vec<_>>()
        .join(" ")
    }
}

impl From<u16> for InnerClassAccessFlags {
    fn from(flags: u16) -> Self {
        assert!(
            (flags & !0x761f) == 0,
            "0x{flags:04x} is not a valid combination of inner class access flags.",
        );
        InnerClassAccessFlags(flags)
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

pub struct FieldAccessFlags(u16);

impl FieldAccessFlags {
    pub fn to_u16(&self) -> u16 {
        self.0
    }

    pub fn contains(&self, flag: FieldAccessFlag) -> bool {
        (self.0 & flag as u16) != 0
    }

    pub fn java_repr(&self) -> String {
        [
            (FieldAccessFlag::Public, "ACC_PUBLIC"),
            (FieldAccessFlag::Private, "ACC_PRIVATE"),
            (FieldAccessFlag::Protected, "ACC_PROTECTED"),
            (FieldAccessFlag::Static, "ACC_STATIC"),
            (FieldAccessFlag::Final, "ACC_FINAL"),
            (FieldAccessFlag::Volatile, "ACC_VOLATILE"),
            (FieldAccessFlag::Transient, "ACC_TRANSIENT"),
            (FieldAccessFlag::Synthetic, "ACC_SYNTHETIC"),
            (FieldAccessFlag::Enum, "ACC_ENUM"),
        ]
        .iter()
        .filter(|(flag, _)| self.contains(*flag))
        .map(|(_, repr)| *repr)
        .collect::<Vec<_>>()
        .join(", ")
    }

    pub fn modifier_repr(&self) -> String {
        [
            (FieldAccessFlag::Public, "public"),
            (FieldAccessFlag::Private, "private"),
            (FieldAccessFlag::Protected, "protected"),
            (FieldAccessFlag::Static, "static"),
            (FieldAccessFlag::Final, "final"),
            (FieldAccessFlag::Volatile, "volatile"),
            (FieldAccessFlag::Transient, "transient"),
            (FieldAccessFlag::Synthetic, ""),
            (FieldAccessFlag::Enum, ""),
        ]
        .iter()
        .filter(|(flag, repr)| self.contains(*flag) && !repr.is_empty())
        .map(|(_, repr)| *repr)
        .collect::<Vec<_>>()
        .join(" ")
    }
}

impl From<u16> for FieldAccessFlags {
    fn from(flags: u16) -> Self {
        assert!(
            (flags & !0x50df) == 0,
            "0x{flags:04x} is not a valid combination of field access flags.",
        );
        FieldAccessFlags(flags)
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

pub struct MethodAccessFlags(u16);

impl MethodAccessFlags {
    pub fn to_u16(&self) -> u16 {
        self.0
    }

    pub fn contains(&self, flag: MethodAccessFlag) -> bool {
        (self.0 & flag as u16) != 0
    }

    pub fn java_repr(&self) -> String {
        [
            (MethodAccessFlag::Public, "ACC_PUBLIC"),
            (MethodAccessFlag::Private, "ACC_PRIVATE"),
            (MethodAccessFlag::Protected, "ACC_PROTECTED"),
            (MethodAccessFlag::Static, "ACC_STATIC"),
            (MethodAccessFlag::Final, "ACC_FINAL"),
            (MethodAccessFlag::Synchronized, "ACC_SYNCHRONIZED"),
            (MethodAccessFlag::Bridge, "ACC_BRIDGE"),
            (MethodAccessFlag::Varargs, "ACC_VARARGS"),
            (MethodAccessFlag::Native, "ACC_NATIVE"),
            (MethodAccessFlag::Abstract, "ACC_ABSTRACT"),
            (MethodAccessFlag::Strict, "ACC_STRICT"),
            (MethodAccessFlag::Synthetic, "ACC_SYNTHETIC"),
        ]
        .iter()
        .filter(|(flag, _)| self.contains(*flag))
        .map(|(_, repr)| *repr)
        .collect::<Vec<_>>()
        .join(", ")
    }

    pub fn modifier_repr(&self) -> String {
        [
            (MethodAccessFlag::Public, "public"),
            (MethodAccessFlag::Private, "private"),
            (MethodAccessFlag::Protected, "protected"),
            (MethodAccessFlag::Static, "static"),
            (MethodAccessFlag::Final, "final"),
            (MethodAccessFlag::Synchronized, "synchronized"),
            (MethodAccessFlag::Bridge, "bridge"),
            (MethodAccessFlag::Varargs, ""),
            (MethodAccessFlag::Native, "native"),
            (MethodAccessFlag::Abstract, "abstract"),
            (MethodAccessFlag::Strict, "strictfp"),
            (MethodAccessFlag::Synthetic, ""),
        ]
        .iter()
        .filter(|(flag, repr)| self.contains(*flag) && !repr.is_empty())
        .map(|(_, repr)| *repr)
        .collect::<Vec<_>>()
        .join(" ")
    }
}

impl From<u16> for MethodAccessFlags {
    fn from(flags: u16) -> Self {
        assert!(
            (flags & !0x1dff) == 0,
            "0x{flags:04x} is not a valid combination of method access flags.",
        );
        MethodAccessFlags(flags)
    }
}

#[repr(u16)]
#[derive(Copy, Clone, PartialEq)]
pub enum MethodParameterAccessFlag {
    Final = 0x0010,
    Synthetic = 0x1000,
    Mandated = 0x8000,
}

pub struct MethodParameterAccessFlags(u16);

impl MethodParameterAccessFlags {
    pub fn to_u16(&self) -> u16 {
        self.0
    }

    pub fn contains(&self, flag: MethodParameterAccessFlag) -> bool {
        (self.0 & flag as u16) != 0
    }

    pub fn modifier_repr(&self) -> String {
        [
            (MethodParameterAccessFlag::Final, "final"),
            (MethodParameterAccessFlag::Synthetic, "synthetic"),
            (MethodParameterAccessFlag::Mandated, "mandated"),
        ]
        .iter()
        .filter(|(flag, _)| self.contains(*flag))
        .map(|(_, repr)| *repr)
        .collect::<Vec<_>>()
        .join(" ")
    }
}

impl From<u16> for MethodParameterAccessFlags {
    fn from(flags: u16) -> Self {
        assert!(
            (flags & !0x9010) == 0,
            "0x{flags:04x} is not a valid combination of method parameter access flags.",
        );
        MethodParameterAccessFlags(flags)
    }
}
