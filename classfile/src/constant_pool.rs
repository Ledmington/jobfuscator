#![forbid(unsafe_code)]

use std::ops::Index;

use binary_reader::BinaryReader;

use crate::reference_kind::ReferenceKind;

pub struct ConstantPool {
    pub(crate) entries: Vec<ConstantPoolInfo>,
}

impl ConstantPool {
    pub fn get_class_name(&self, cp_index: u16) -> String {
        let class_entry: &ConstantPoolInfo = &self[cp_index - 1];
        match class_entry {
            ConstantPoolInfo::Class { name_index } => self.get_utf8_content(*name_index),
            _ => panic!(
                "Expected entry #{} to be of Class type but it wasn't.",
                cp_index
            ),
        }
    }

    pub fn get_method_ref(&self, cp_index: u16) -> String {
        let method_ref_entry: &ConstantPoolInfo = &self[cp_index - 1];
        match method_ref_entry {
            ConstantPoolInfo::MethodRef {
                class_index,
                name_and_type_index,
            } => self.get_method_ref_string(*class_index, *name_and_type_index),
            ConstantPoolInfo::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => self.get_method_ref_string(*class_index, *name_and_type_index),
            _ => panic!(
                "Expected entry #{} to be of Methodref type but it wasn't.",
                cp_index
            ),
        }
    }

    pub fn get_method_ref_string(&self, class_index: u16, name_and_type_index: u16) -> String {
        self.get_class_name(class_index) + "." + &self.get_name_and_type(name_and_type_index)
    }

    pub fn get_field_ref(&self, cp_index: u16) -> String {
        let field_ref_entry: &ConstantPoolInfo = &self[cp_index - 1];
        match field_ref_entry {
            ConstantPoolInfo::FieldRef {
                class_index,
                name_and_type_index,
            } => self.get_field_ref_string(*class_index, *name_and_type_index),
            _ => panic!(
                "Expected entry #{} to be of Fieldref type but it wasn't.",
                cp_index
            ),
        }
    }

    pub fn get_field_ref_string(&self, class_index: u16, name_and_type_index: u16) -> String {
        self.get_class_name(class_index) + "." + &self.get_name_and_type(name_and_type_index)
    }

    pub fn get_field_ref_name_and_type(&self, cp_index: u16) -> String {
        let field_ref_entry: &ConstantPoolInfo = &self[cp_index - 1];
        match field_ref_entry {
            ConstantPoolInfo::FieldRef {
                class_index: _,
                name_and_type_index,
            } => self.get_name_and_type(*name_and_type_index),
            _ => panic!(
                "Expected entry #{} to be of Fieldref type but it wasn't.",
                cp_index
            ),
        }
    }

    pub fn get_invoke_dynamic(&self, cp_index: u16) -> String {
        let invoke_dynamic_entry: &ConstantPoolInfo = &self[cp_index - 1];
        match invoke_dynamic_entry {
            ConstantPoolInfo::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => self.get_invoke_dynamic_string(*bootstrap_method_attr_index, *name_and_type_index),
            _ => panic!(
                "Expected entry #{} to be of InvokeDynamic type but it wasn't.",
                cp_index
            ),
        }
    }

    pub fn get_invoke_dynamic_string(
        &self,
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    ) -> String {
        "#".to_owned()
            + &bootstrap_method_attr_index.to_string()
            + ":"
            + &self.get_name_and_type(name_and_type_index)
    }

    pub fn get_name_and_type(&self, cp_index: u16) -> String {
        let name_and_type_entry: &ConstantPoolInfo = &self[cp_index - 1];
        match name_and_type_entry {
            ConstantPoolInfo::NameAndType {
                name_index,
                descriptor_index,
            } => self.get_name_and_type_string(*name_index, *descriptor_index),
            _ => panic!(
                "Expected entry #{} to be of NameAndType type but it wasn't.",
                cp_index
            ),
        }
    }

    pub fn get_name_and_type_string(&self, name_index: u16, descriptor_index: u16) -> String {
        let name = self.get_utf8_content(name_index);
        if name.starts_with('<') {
            "\"".to_owned() + &name + "\":" + &self.get_utf8_content(descriptor_index)
        } else {
            name + ":" + &self.get_utf8_content(descriptor_index)
        }
    }

    pub fn get_utf8_content(&self, cp_index: u16) -> String {
        let name_entry: &ConstantPoolInfo = &self[cp_index - 1];
        match name_entry {
            ConstantPoolInfo::Utf8 { bytes } => {
                let content = convert_utf8(bytes);
                if content.starts_with('[') {
                    "\"".to_owned() + &content + "\""
                } else {
                    content
                }
            }
            _ => panic!(
                "Expected entry #{} to be of Utf8 type but it wasn't.",
                cp_index
            ),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Index<u16> for ConstantPool {
    type Output = ConstantPoolInfo;

    fn index(&self, index: u16) -> &Self::Output {
        &self.entries[index as usize]
    }
}

pub fn convert_utf8(utf8_bytes: &[u8]) -> String {
    String::from_utf8(utf8_bytes.to_vec())
        .unwrap()
        .replace("\n", "\\n")
        .replace("'", "\\'")
        .replace("\u{0001}", "\\u0001")
}

#[derive(Clone)]
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

pub fn parse_constant_pool(reader: &mut BinaryReader, cp_count: usize) -> ConstantPool {
    let mut entries: Vec<ConstantPoolInfo> = Vec::with_capacity(cp_count);
    let mut i = 0;
    while i < cp_count {
        let tag = ConstantPoolTag::from(reader.read_u8().unwrap());
        entries.push(parse_constant_pool_info(reader, tag.clone()));
        if matches!(tag, ConstantPoolTag::Long) || matches!(tag, ConstantPoolTag::Double) {
            entries.push(ConstantPoolInfo::Null {});
            i += 1;
        }
        i += 1;
    }
    ConstantPool { entries }
}

fn parse_constant_pool_info(reader: &mut BinaryReader, tag: ConstantPoolTag) -> ConstantPoolInfo {
    match tag {
        ConstantPoolTag::Utf8 => {
            let length: u16 = reader.read_u16().unwrap();
            ConstantPoolInfo::Utf8 {
                bytes: reader.read_u8_vec(length.into()).unwrap(),
            }
        }
        ConstantPoolTag::Long => ConstantPoolInfo::Long {
            high_bytes: reader.read_u32().unwrap(),
            low_bytes: reader.read_u32().unwrap(),
        },
        ConstantPoolTag::String => ConstantPoolInfo::String {
            string_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::Class => ConstantPoolInfo::Class {
            name_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::Fieldref => ConstantPoolInfo::FieldRef {
            class_index: reader.read_u16().unwrap(),
            name_and_type_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::Methodref => ConstantPoolInfo::MethodRef {
            class_index: reader.read_u16().unwrap(),
            name_and_type_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::InterfaceMethodref => ConstantPoolInfo::InterfaceMethodRef {
            class_index: reader.read_u16().unwrap(),
            name_and_type_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::NameAndType => ConstantPoolInfo::NameAndType {
            name_index: reader.read_u16().unwrap(),
            descriptor_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::MethodHandle => ConstantPoolInfo::MethodHandle {
            reference_kind: ReferenceKind::from(reader.read_u8().unwrap()),
            reference_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::MethodType => ConstantPoolInfo::MethodType {
            descriptor_index: reader.read_u16().unwrap(),
        },
        ConstantPoolTag::InvokeDynamic => ConstantPoolInfo::InvokeDynamic {
            bootstrap_method_attr_index: reader.read_u16().unwrap(),
            name_and_type_index: reader.read_u16().unwrap(),
        },
        _ => panic!("Unknown constant pool tag {:?}.", tag),
    }
}
