use crate::reference_kind::ReferenceKind;

pub enum ConstantPoolInfo {
    /**
     * The type of constant pool entry which can be found right after a Long or Double one.
     */
    Null {},
    Utf8 {
        bytes: Vec<u8>,
    },
    Long {
        high_bytes: u32,
        low_bytes: u32,
    },
    Double {
        high_bytes: u32,
        low_bytes: u32,
    },
    String {
        string_index: u16,
    },
    Class {
        name_index: u16,
    },
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    InterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    MethodType {
        descriptor_index: u16,
    },
    MethodHandle {
        reference_kind: ReferenceKind,
        reference_index: u16,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
}

#[derive(Debug, Clone)]
pub enum ConstantPoolTag {
    Utf8,
    Integer,
    Float,
    Long,
    Double,
    String,
    Class,
    Fieldref,
    Methodref,
    InterfaceMethodref,
    NameAndType,
    MethodHandle,
    MethodType,
    Dynamic,
    InvokeDynamic,
    Module,
    Package,
}

impl From<u8> for ConstantPoolTag {
    fn from(value: u8) -> Self {
        match value {
            1 => ConstantPoolTag::Utf8,
            3 => ConstantPoolTag::Integer,
            4 => ConstantPoolTag::Float,
            5 => ConstantPoolTag::Long,
            6 => ConstantPoolTag::Double,
            8 => ConstantPoolTag::String,
            7 => ConstantPoolTag::Class,
            9 => ConstantPoolTag::Fieldref,
            10 => ConstantPoolTag::Methodref,
            11 => ConstantPoolTag::InterfaceMethodref,
            12 => ConstantPoolTag::NameAndType,
            15 => ConstantPoolTag::MethodHandle,
            16 => ConstantPoolTag::MethodType,
            17 => ConstantPoolTag::Dynamic,
            18 => ConstantPoolTag::InvokeDynamic,
            19 => ConstantPoolTag::Module,
            20 => ConstantPoolTag::Package,
            _ => panic!("Unknown constant pool tag value {}.", value),
        }
    }
}
