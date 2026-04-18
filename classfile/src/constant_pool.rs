#![forbid(unsafe_code)]

use std::ops::Index;

use binary_reader::BinaryReader;

use crate::{
    attributes::{AttributeInfo, AttributeKind, find_attribute},
    reference_kind::ReferenceKind,
};

pub struct ConstantPool {
    pub(crate) entries: Vec<ConstantPoolInfo>,
}

impl ConstantPool {
    pub fn get_class_name(&self, cp_index: u16) -> String {
        let class_entry: &ConstantPoolInfo = &self[cp_index - 1];
        match class_entry {
            ConstantPoolInfo::Class { name_index } => self.get_wrapped_utf8_content(*name_index),
            _ => panic!("Expected entry #{cp_index} to be of Class type but it wasn't."),
        }
    }

    pub fn get_method_ref(&self, cp_index: u16) -> String {
        let method_ref_entry: &ConstantPoolInfo = &self[cp_index - 1];
        match method_ref_entry {
            ConstantPoolInfo::FieldRef {
                class_index,
                name_and_type_index,
            } => self.get_field_ref_string(*class_index, *name_and_type_index),
            ConstantPoolInfo::MethodRef {
                class_index,
                name_and_type_index,
            } => self.get_method_ref_string(*class_index, *name_and_type_index),
            ConstantPoolInfo::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => self.get_method_ref_string(*class_index, *name_and_type_index),
            _ => panic!("Expected entry #{cp_index} to be of Methodref type but it wasn't."),
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
            _ => panic!("Expected entry #{cp_index} to be of Fieldref type but it wasn't."),
        }
    }

    pub fn get_field_ref_string(&self, class_index: u16, name_and_type_index: u16) -> String {
        self.get_class_name(class_index) + "." + &self.get_name_and_type(name_and_type_index)
    }

    pub fn get_field_ref_name_and_type(&self, cp_index: u16) -> String {
        let field_ref_entry: &ConstantPoolInfo = &self[cp_index - 1];
        match field_ref_entry {
            ConstantPoolInfo::FieldRef {
                name_and_type_index,
                ..
            } => self.get_name_and_type(*name_and_type_index),
            _ => panic!("Expected entry #{cp_index} to be of Fieldref type but it wasn't."),
        }
    }

    pub fn get_invoke_dynamic(&self, cp_index: u16) -> String {
        let invoke_dynamic_entry: &ConstantPoolInfo = &self[cp_index - 1];
        match invoke_dynamic_entry {
            ConstantPoolInfo::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => self.get_invoke_dynamic_string(*bootstrap_method_attr_index, *name_and_type_index),
            _ => panic!("Expected entry #{cp_index} to be of InvokeDynamic type but it wasn't."),
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
            _ => panic!("Expected entry #{cp_index} to be of NameAndType type but it wasn't."),
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

    // FIXME: find a better name
    pub fn get_wrapped_utf8_content(&self, cp_index: u16) -> String {
        let content = self.get_utf8_content(cp_index);
        if content.starts_with('[') {
            "\"".to_owned() + &content + "\""
        } else {
            content
        }
    }

    pub fn get_utf8_content(&self, cp_index: u16) -> String {
        let name_entry: &ConstantPoolInfo = &self[cp_index - 1];
        match name_entry {
            ConstantPoolInfo::Utf8 { bytes } => convert_utf8(bytes),
            _ => panic!("Expected entry #{cp_index} to be of Utf8 type but it wasn't."),
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

pub enum ConstantPoolInfo {
    /**
     * The type of constant pool entry which can be found right after a Long or Double one.
     */
    Null {},
    Utf8 {
        bytes: Vec<u8>,
    },
    Integer {
        bytes: u32,
    },
    Float {
        bytes: u32,
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

impl ConstantPoolInfo {
    pub fn tag(&self) -> ConstantPoolTag {
        match self {
            ConstantPoolInfo::Null {} => panic!("Null entries have no tag"),
            ConstantPoolInfo::Utf8 { .. } => ConstantPoolTag::Utf8,
            ConstantPoolInfo::Integer { .. } => ConstantPoolTag::Integer,
            ConstantPoolInfo::Float { .. } => ConstantPoolTag::Float,
            ConstantPoolInfo::Long { .. } => ConstantPoolTag::Long,
            ConstantPoolInfo::Double { .. } => ConstantPoolTag::Double,
            ConstantPoolInfo::String { .. } => ConstantPoolTag::String,
            ConstantPoolInfo::Class { .. } => ConstantPoolTag::Class,
            ConstantPoolInfo::FieldRef { .. } => ConstantPoolTag::Fieldref,
            ConstantPoolInfo::MethodRef { .. } => ConstantPoolTag::Methodref,
            ConstantPoolInfo::InterfaceMethodRef { .. } => ConstantPoolTag::InterfaceMethodref,
            ConstantPoolInfo::NameAndType { .. } => ConstantPoolTag::NameAndType,
            ConstantPoolInfo::MethodHandle { .. } => ConstantPoolTag::MethodHandle,
            ConstantPoolInfo::MethodType { .. } => ConstantPoolTag::MethodType,
            ConstantPoolInfo::InvokeDynamic { .. } => ConstantPoolTag::InvokeDynamic,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug)]
pub struct UnknownConstantPoolTag(pub u8);

impl TryFrom<u8> for ConstantPoolTag {
    type Error = UnknownConstantPoolTag;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::Utf8,
            3 => Self::Integer,
            4 => Self::Float,
            5 => Self::Long,
            6 => Self::Double,
            8 => Self::String,
            7 => Self::Class,
            9 => Self::Fieldref,
            10 => Self::Methodref,
            11 => Self::InterfaceMethodref,
            12 => Self::NameAndType,
            15 => Self::MethodHandle,
            16 => Self::MethodType,
            17 => Self::Dynamic,
            18 => Self::InvokeDynamic,
            19 => Self::Module,
            20 => Self::Package,
            v => return Err(UnknownConstantPoolTag(v)),
        })
    }
}

impl From<ConstantPoolTag> for u8 {
    fn from(tag: ConstantPoolTag) -> Self {
        match tag {
            ConstantPoolTag::Utf8 => 1,
            ConstantPoolTag::Integer => 3,
            ConstantPoolTag::Float => 4,
            ConstantPoolTag::Long => 5,
            ConstantPoolTag::Double => 6,
            ConstantPoolTag::String => 8,
            ConstantPoolTag::Class => 7,
            ConstantPoolTag::Fieldref => 9,
            ConstantPoolTag::Methodref => 10,
            ConstantPoolTag::InterfaceMethodref => 11,
            ConstantPoolTag::NameAndType => 12,
            ConstantPoolTag::MethodHandle => 15,
            ConstantPoolTag::MethodType => 16,
            ConstantPoolTag::Dynamic => 17,
            ConstantPoolTag::InvokeDynamic => 18,
            ConstantPoolTag::Module => 19,
            ConstantPoolTag::Package => 20,
        }
    }
}

impl std::fmt::Display for ConstantPoolTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub fn parse_constant_pool(reader: &mut BinaryReader, cp_count: usize) -> ConstantPool {
    let mut entries: Vec<ConstantPoolInfo> = Vec::with_capacity(cp_count);
    let mut i = 0;
    while i < cp_count {
        let tag = ConstantPoolTag::try_from(reader.read_u8().unwrap()).unwrap();
        entries.push(parse_constant_pool_entry(reader, tag.clone()));

        if matches!(tag, ConstantPoolTag::Long) || matches!(tag, ConstantPoolTag::Double) {
            entries.push(ConstantPoolInfo::Null {});
            i += 1;
        }
        i += 1;
    }
    ConstantPool { entries }
}

fn parse_constant_pool_entry(reader: &mut BinaryReader, tag: ConstantPoolTag) -> ConstantPoolInfo {
    match tag {
        ConstantPoolTag::Utf8 => {
            let length: u16 = reader.read_u16().unwrap();
            ConstantPoolInfo::Utf8 {
                bytes: reader.read_u8_vec(length.into()).unwrap(),
            }
        }
        ConstantPoolTag::Integer => ConstantPoolInfo::Integer {
            bytes: reader.read_u32().unwrap(),
        },
        ConstantPoolTag::Float => ConstantPoolInfo::Float {
            bytes: reader.read_u32().unwrap(),
        },
        ConstantPoolTag::Long => ConstantPoolInfo::Long {
            high_bytes: reader.read_u32().unwrap(),
            low_bytes: reader.read_u32().unwrap(),
        },
        ConstantPoolTag::Double => ConstantPoolInfo::Double {
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
        _ => panic!("Unknown constant pool tag {tag:?}."),
    }
}

// TODO: find a better name
#[macro_export]
macro_rules! assert_valid_and_type {
    ($cp:expr, $cp_index:expr, $($expected_tag:expr),+) => {{
        let cp = $cp;
        let cp_index = $cp_index;
        let cp_len = cp.len();
        let expected_tags = &[$($expected_tag),+];
        assert!(expected_tags.len() > 0, "Empty expected tags.");
        assert!(
            cp_index >= 1 && cp_index <= (cp_len as u16),
            "Constant pool index must be >= 1 and <= {cp_len} but was {cp_index} (0x{cp_index:04x})."
        );
        let actual_tag = cp[cp_index - 1].tag();
        let mut found: bool = false;
        for i in 0..expected_tags.len() {
            let expected_tag = &expected_tags[i];
            if *expected_tag == actual_tag {
                found = true;
                break;
            }
        }
        if !found {
            if expected_tags.len() == 1 {
                panic!("Expected an entry with tag {:?} at index {cp_index} but was {actual_tag}.", expected_tags[0]);
            } else {
                panic!("Expected an entry with any of the tags {} at index {cp_index} but was {actual_tag}.", expected_tags.iter().map(|t| format!("{t:?}")).collect::<Vec<String>>().join(", "));
            }
        }
    }};
}

pub(crate) fn check_constant_pool(cp: &ConstantPool, attributes: &[AttributeInfo]) {
    let mut i = 0;
    while i < cp.len() {
        let entry = &cp[i.try_into().unwrap()];
        match entry {
            ConstantPoolInfo::Null {} => {
                unreachable!("Checking a null CP entry.");
            }
            ConstantPoolInfo::Utf8 { .. } => {}
            ConstantPoolInfo::Integer { .. } => {}
            ConstantPoolInfo::Float { .. } => {}
            ConstantPoolInfo::Long { .. } => {
                i += 1;
            }
            ConstantPoolInfo::Double { .. } => {
                i += 1;
            }
            ConstantPoolInfo::String { string_index } => {
                assert_valid_and_type!(cp, *string_index, ConstantPoolTag::Utf8);
            }
            ConstantPoolInfo::Class { name_index } => {
                assert_valid_and_type!(cp, *name_index, ConstantPoolTag::Utf8);
            }
            ConstantPoolInfo::FieldRef {
                class_index,
                name_and_type_index,
            } => {
                assert_valid_and_type!(cp, *class_index, ConstantPoolTag::Class);
                assert_valid_and_type!(cp, *name_and_type_index, ConstantPoolTag::NameAndType);
            }
            ConstantPoolInfo::MethodRef {
                class_index,
                name_and_type_index,
            } => {
                assert_valid_and_type!(cp, *class_index, ConstantPoolTag::Class);
                assert_valid_and_type!(cp, *name_and_type_index, ConstantPoolTag::NameAndType);
            }
            ConstantPoolInfo::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => {
                assert_valid_and_type!(cp, *class_index, ConstantPoolTag::Class);
                assert_valid_and_type!(cp, *name_and_type_index, ConstantPoolTag::NameAndType);
            }
            ConstantPoolInfo::NameAndType {
                name_index,
                descriptor_index,
            } => {
                assert_valid_and_type!(cp, *name_index, ConstantPoolTag::Utf8);
                assert_valid_and_type!(cp, *descriptor_index, ConstantPoolTag::Utf8);
            }
            ConstantPoolInfo::MethodType { descriptor_index } => {
                assert_valid_and_type!(cp, *descriptor_index, ConstantPoolTag::Utf8);
            }
            ConstantPoolInfo::MethodHandle {
                reference_kind,
                reference_index,
            } => match reference_kind {
                ReferenceKind::GetField
                | ReferenceKind::GetStatic
                | ReferenceKind::PutField
                | ReferenceKind::PutStatic => {
                    assert_valid_and_type!(cp, *reference_index, ConstantPoolTag::Fieldref);
                }
                ReferenceKind::InvokeVirtual | ReferenceKind::NewInvokeSpecial => {
                    assert_valid_and_type!(cp, *reference_index, ConstantPoolTag::Methodref);
                }
                ReferenceKind::InvokeStatic | ReferenceKind::InvokeSpecial => {
                    assert_valid_and_type!(
                        cp,
                        *reference_index,
                        ConstantPoolTag::Methodref,
                        ConstantPoolTag::InterfaceMethodref
                    );
                }
                ReferenceKind::InvokeInterface => {
                    assert_valid_and_type!(
                        cp,
                        *reference_index,
                        ConstantPoolTag::InterfaceMethodref
                    );
                }
            },
            ConstantPoolInfo::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => {
                let bootstrap_method_attribute =
                    find_attribute(attributes,AttributeKind::BootstrapMethods).expect("The presence of an InvokeDynamic entry in the constant pool implies the presence of a BootstrapMethods attribute in the class file, which is not present.");
                match bootstrap_method_attribute {
                    AttributeInfo::BootstrapMethods { methods, .. } => {
                        let num_bootstrap_methods = methods.len();
                        assert!(
                            (*bootstrap_method_attr_index as usize) < num_bootstrap_methods,
                            "Expected bootstrap_method_attr_index ({bootstrap_method_attr_index}) to be a valid index in the BootstrapMethods array (len: {num_bootstrap_methods}) but it was not."
                        );
                    }
                    _ => unreachable!(),
                }
                assert_valid_and_type!(cp, *name_and_type_index, ConstantPoolTag::NameAndType);
            }
        }

        i += 1;
    }
}
