#![forbid(unsafe_code)]

use binary_reader::BinaryReader;

use crate::access_flags::{InnerClassAccessFlags, MethodParameterAccessFlags};
use crate::assert_valid_and_type;
use crate::bytecode::{BytecodeInstruction, parse_bytecode};
use crate::constant_pool::{ConstantPool, ConstantPoolTag};
use crate::writer::{get_annotation_length, get_stack_map_entry_length};

pub enum AttributeInfo {
    Code {
        name_index: u16,
        max_stack: u16,
        max_locals: u16,
        code: Vec<(u32, BytecodeInstruction)>,
        exception_table: Vec<ExceptionTableEntry>,
        attributes: Vec<AttributeInfo>,
    },
    LineNumberTable {
        name_index: u16,
        line_number_table: Vec<LineNumberTableEntry>,
    },
    LocalVariableTable {
        name_index: u16,
        local_variable_table: Vec<LocalVariableTableEntry>,
    },
    LocalVariableTypeTable {
        name_index: u16,
        local_variable_type_table: Vec<LocalVariableTypeTableEntry>,
    },
    StackMapTable {
        name_index: u16,
        stack_map_table: Vec<StackMapFrame>,
    },
    SourceFile {
        name_index: u16,
        source_file_index: u16,
    },
    BootstrapMethods {
        name_index: u16,
        methods: Vec<BootstrapMethod>,
    },
    InnerClasses {
        name_index: u16,
        classes: Vec<InnerClassInfo>,
    },
    MethodParameters {
        name_index: u16,
        parameters: Vec<MethodParameter>,
    },
    Record {
        name_index: u16,
        components: Vec<RecordComponentInfo>,
    },
    Signature {
        name_index: u16,
        signature_index: u16,
    },
    NestMembers {
        name_index: u16,
        classes: Vec<u16>,
    },
    RuntimeVisibleAnnotations {
        name_index: u16,
        annotations: Vec<Annotation>,
    },
    ConstantValue {
        name_index: u16,
        constant_value_index: u16,
    },
    Exceptions {
        name_index: u16,
        exception_indices: Vec<u16>,
    },
}

#[derive(Debug, PartialEq)]
pub enum AttributeKind {
    Code,
    LineNumberTable,
    LocalVariableTable,
    LocalVariableTypeTable,
    StackMapTable,
    SourceFile,
    BootstrapMethods,
    InnerClasses,
    MethodParameters,
    Record,
    Signature,
    NestMembers,
    RuntimeVisibleAnnotations,
    ConstantValue,
    Exceptions,
}

impl std::fmt::Display for AttributeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl AttributeInfo {
    pub fn kind(&self) -> AttributeKind {
        match self {
            AttributeInfo::Code { .. } => AttributeKind::Code,
            AttributeInfo::LineNumberTable { .. } => AttributeKind::LineNumberTable,
            AttributeInfo::LocalVariableTable { .. } => AttributeKind::LocalVariableTable,
            AttributeInfo::LocalVariableTypeTable { .. } => AttributeKind::LocalVariableTypeTable,
            AttributeInfo::StackMapTable { .. } => AttributeKind::StackMapTable,
            AttributeInfo::SourceFile { .. } => AttributeKind::SourceFile,
            AttributeInfo::BootstrapMethods { .. } => AttributeKind::BootstrapMethods,
            AttributeInfo::InnerClasses { .. } => AttributeKind::InnerClasses,
            AttributeInfo::MethodParameters { .. } => AttributeKind::MethodParameters,
            AttributeInfo::Record { .. } => AttributeKind::Record,
            AttributeInfo::Signature { .. } => AttributeKind::Signature,
            AttributeInfo::NestMembers { .. } => AttributeKind::NestMembers,
            AttributeInfo::RuntimeVisibleAnnotations { .. } => {
                AttributeKind::RuntimeVisibleAnnotations
            }
            AttributeInfo::ConstantValue { .. } => AttributeKind::ConstantValue,
            AttributeInfo::Exceptions { .. } => AttributeKind::Exceptions,
        }
    }
}

pub struct Annotation {
    pub type_index: u16,
    pub element_value_pairs: Vec<ElementValuePair>,
}

pub struct ElementValuePair {
    pub element_name_index: u16,
    pub value: ElementValue,
}

pub enum ElementValue {
    Byte {
        const_value_index: u16,
    },
    Char {
        const_value_index: u16,
    },
    Double {
        const_value_index: u16,
    },
    Float {
        const_value_index: u16,
    },
    Int {
        const_value_index: u16,
    },
    Long {
        const_value_index: u16,
    },
    Short {
        const_value_index: u16,
    },
    Boolean {
        const_value_index: u16,
    },
    String {
        const_value_index: u16,
    },
    Enum {
        type_name_index: u16,
        const_name_index: u16,
    },
    Class {
        class_info_index: u16,
    },
    Annotation {
        value: Annotation,
    },
    Array {
        values: Vec<ElementValue>,
    },
}

impl ElementValue {
    pub fn tag(&self) -> char {
        match self {
            ElementValue::Byte { .. } => 'B',
            ElementValue::Char { .. } => 'C',
            ElementValue::Double { .. } => 'D',
            ElementValue::Float { .. } => 'F',
            ElementValue::Int { .. } => 'I',
            ElementValue::Long { .. } => 'J',
            ElementValue::Short { .. } => 'S',
            ElementValue::Boolean { .. } => 'Z',
            ElementValue::String { .. } => 's',
            ElementValue::Enum { .. } => 'e',
            ElementValue::Class { .. } => 'c',
            ElementValue::Annotation { .. } => '@',
            ElementValue::Array { .. } => '[',
        }
    }
}

pub struct RecordComponentInfo {
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>,
}

pub struct MethodParameter {
    pub name_index: u16,
    pub access_flags: MethodParameterAccessFlags,
}

pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

pub struct LineNumberTableEntry {
    pub start_pc: u16,
    pub line_number: u16,
}

pub struct LocalVariableTableEntry {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub index: u16,
}

pub struct LocalVariableTypeTableEntry {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub index: u16,
}

pub enum StackMapFrame {
    SameFrame {
        frame_type: u8,
    },
    SameLocals1StackItemFrame {
        frame_type: u8,
        stack: VerificationTypeInfo,
    },
    SameLocals1StackItemFrameExtended {
        offset_delta: u16,
        stack: VerificationTypeInfo,
    },
    ChopFrame {
        frame_type: u8,
        offset_delta: u16,
    },
    SameFrameExtended {
        offset_delta: u16,
    },
    AppendFrame {
        frame_type: u8,
        offset_delta: u16,
        locals: Vec<VerificationTypeInfo>,
    },
    FullFrame {
        offset_delta: u16,
        locals: Vec<VerificationTypeInfo>,
        stack: Vec<VerificationTypeInfo>,
    },
}

#[derive(Debug)]
pub enum VerificationTypeInfo {
    TopVariable,
    IntegerVariable,
    FloatVariable,
    LongVariable,
    DoubleVariable,
    NullVariable,
    UninitializedThisVariable,
    ObjectVariable { constant_pool_index: u16 },
    UninitializedVariable { offset: u16 },
}

pub struct BootstrapMethod {
    pub bootstrap_method_ref: u16,
    pub bootstrap_arguments: Vec<u16>,
}

pub struct InnerClassInfo {
    pub inner_class_info_index: u16,
    pub outer_class_info_index: u16,
    pub inner_name_index: u16,
    pub inner_class_access_flags: InnerClassAccessFlags,
}

impl InnerClassInfo {
    /// Indicates whether this inner class is anonymous, meaning that it has no simple name.
    pub fn is_anonymous(&self) -> bool {
        self.inner_name_index == 0
    }

    /// Indicates whether this inner class is local, meaning that this class has a simple name and no enclosing class entry.
    pub fn is_local(&self) -> bool {
        self.inner_name_index != 0 && self.outer_class_info_index == 0
    }

    /// Indicates whether this inner class is member, meaning that this class has both a simple name and an enclosing class entry.
    pub fn is_member(&self) -> bool {
        self.inner_name_index != 0 && self.outer_class_info_index != 0
    }
}

pub fn parse_class_attributes(
    reader: &mut BinaryReader,
    cp: &ConstantPool,
    num_attributes: usize,
) -> Vec<AttributeInfo> {
    let mut attributes: Vec<AttributeInfo> = Vec::with_capacity(num_attributes);
    for i in 0..num_attributes {
        attributes.push(parse_classfile_attribute(reader, cp));
        for j in 0..i {
            assert!(
                attributes[i].kind() != attributes[j].kind(),
                "Found duplicate class attributes with kind {} at indices {} and {}.",
                attributes[i].kind(),
                i,
                j
            );
        }
    }
    attributes
}

fn parse_classfile_attribute(reader: &mut BinaryReader, cp: &ConstantPool) -> AttributeInfo {
    let attribute_name_index: u16 = reader.read_u16().unwrap();
    assert_valid_and_type!(cp, attribute_name_index, ConstantPoolTag::Utf8);
    let attribute_name: String = cp.get_utf8_content(attribute_name_index);
    let attribute_length: u32 = reader.read_u32().unwrap();
    match attribute_name.as_str() {
        "SourceFile" => {
            assert!(
                attribute_length == 2,
                "The attribute_length field of SourceFile must be 2 but was {attribute_length}.",
            );
            let source_file_index: u16 = reader.read_u16().unwrap();
            assert_valid_and_type!(cp, source_file_index, ConstantPoolTag::Utf8);
            AttributeInfo::SourceFile {
                name_index: attribute_name_index,
                source_file_index,
            }
        }
        "BootstrapMethods" => {
            let mut running_length: u32 = 0;
            let num_bootstrap_methods: u16 = reader.read_u16().unwrap();
            running_length += 2;
            let mut methods: Vec<BootstrapMethod> =
                Vec::with_capacity(num_bootstrap_methods.into());
            for _ in 0..num_bootstrap_methods {
                let bootstrap_method_ref: u16 = reader.read_u16().unwrap();
                assert_valid_and_type!(cp, bootstrap_method_ref, ConstantPoolTag::MethodHandle);
                let num_bootstrap_arguments: u16 = reader.read_u16().unwrap();
                let bootstrap_arguments: Vec<u16> =
                    reader.read_u16_vec(num_bootstrap_arguments.into()).unwrap();
                for index in bootstrap_arguments.iter() {
                    assert_valid_and_type!(
                        cp,
                        *index,
                        ConstantPoolTag::Integer,
                        ConstantPoolTag::Float,
                        ConstantPoolTag::Long,
                        ConstantPoolTag::Double,
                        ConstantPoolTag::Class,
                        ConstantPoolTag::String,
                        ConstantPoolTag::MethodHandle,
                        ConstantPoolTag::MethodType,
                        ConstantPoolTag::Dynamic
                    );
                }
                running_length += 2 + 2 + 2 * (bootstrap_arguments.len() as u32);
                methods.push(BootstrapMethod {
                    bootstrap_method_ref,
                    bootstrap_arguments,
                });
            }
            assert!(
                attribute_length == running_length,
                "Expected length of attribute BootstrapMethods (with {} methods) to be {} bytes but was {}.",
                methods.len(),
                attribute_length,
                running_length
            );
            AttributeInfo::BootstrapMethods {
                name_index: attribute_name_index,
                methods,
            }
        }
        "InnerClasses" => {
            let number_of_classes: u16 = reader.read_u16().unwrap();
            let expected_attribute_length: u32 = 2 + (2 * 4) * (number_of_classes as u32);
            assert!(
                attribute_length == expected_attribute_length,
                "Expected length of attribute InnerClasses (with {number_of_classes} inner classes) to be {expected_attribute_length} bytes but was {attribute_length}.",
            );
            let mut classes: Vec<InnerClassInfo> = Vec::with_capacity(number_of_classes.into());
            for i in 0..number_of_classes {
                let inner_class_info_index = reader.read_u16().unwrap();
                let outer_class_info_index = reader.read_u16().unwrap();
                if inner_class_info_index == 0 {
                    assert!(
                        outer_class_info_index == 0,
                        "Expected field outer_class_info_index of entry n.{i} to be 0 since inner_class_info_index was zero, but it was {outer_class_info_index}.",
                    );
                } else {
                    assert_valid_and_type!(cp, inner_class_info_index, ConstantPoolTag::Class);
                    assert!(
                        inner_class_info_index != outer_class_info_index,
                        "Expected field outer_class_info_index of entry n.{i} to be different from inner_class_info_index ({inner_class_info_index}) but it was.",
                    );
                    if outer_class_info_index != 0 {
                        assert_valid_and_type!(cp, outer_class_info_index, ConstantPoolTag::Class);
                    }
                }
                let inner_name_index = reader.read_u16().unwrap();
                if inner_name_index != 0 {
                    // The inner class is not anonymous
                    assert_valid_and_type!(cp, inner_name_index, ConstantPoolTag::Utf8);
                }
                let inner_class_access_flags: InnerClassAccessFlags =
                    InnerClassAccessFlags::from(reader.read_u16().unwrap());
                classes.push(InnerClassInfo {
                    inner_class_info_index,
                    outer_class_info_index,
                    inner_name_index,
                    inner_class_access_flags,
                });
            }
            AttributeInfo::InnerClasses {
                name_index: attribute_name_index,
                classes,
            }
        }
        "Record" => {
            let components_count: u16 = reader.read_u16().unwrap();
            let mut components: Vec<RecordComponentInfo> =
                Vec::with_capacity(components_count.into());
            for _ in 0..components_count {
                let name_index: u16 = reader.read_u16().unwrap();
                let descriptor_index: u16 = reader.read_u16().unwrap();
                let attributes_count: u16 = reader.read_u16().unwrap();
                let attributes: Vec<AttributeInfo> =
                    parse_class_attributes(reader, cp, attributes_count.into());
                components.push(RecordComponentInfo {
                    name_index,
                    descriptor_index,
                    attributes,
                });
            }
            AttributeInfo::Record {
                name_index: attribute_name_index,
                components,
            }
        }
        "Signature" => {
            check_attribute_length(attribute_length, 2, attribute_name);
            let signature_index: u16 = reader.read_u16().unwrap();
            AttributeInfo::Signature {
                name_index: attribute_name_index,
                signature_index,
            }
        }
        "NestMembers" => {
            let number_of_classes: u16 = reader.read_u16().unwrap();
            let classes: Vec<u16> = reader.read_u16_vec(number_of_classes.into()).unwrap();
            AttributeInfo::NestMembers {
                name_index: attribute_name_index,
                classes,
            }
        }
        _ => panic!(
            "The name '{attribute_name}' is either not of an attribute or not a class attribute.",
        ),
    }
}

pub fn parse_field_attributes(
    reader: &mut BinaryReader,
    cp: &ConstantPool,
    num_attributes: usize,
) -> Vec<AttributeInfo> {
    let mut attributes: Vec<AttributeInfo> = Vec::with_capacity(num_attributes);
    for i in 0..num_attributes {
        attributes.push(parse_field_attribute(reader, cp));
        for j in 0..i {
            assert!(
                attributes[i].kind() != attributes[j].kind(),
                "Found duplicate field attributes with kind {} at indices {} and {}.",
                attributes[i].kind(),
                i,
                j
            );
        }
    }
    attributes
}

fn check_attribute_length(
    expected_attribute_length: u32,
    actual_attribute_length: u32,
    attribute_name: String,
) {
    assert!(
        expected_attribute_length == actual_attribute_length,
        "Expected length of attribute {attribute_name} to be {expected_attribute_length} bytes but was {actual_attribute_length} bytes.",
    );
}

fn parse_field_attribute(reader: &mut BinaryReader, cp: &ConstantPool) -> AttributeInfo {
    let attribute_name_index: u16 = reader.read_u16().unwrap();
    assert_valid_and_type!(cp, attribute_name_index, ConstantPoolTag::Utf8);
    let attribute_name: String = cp.get_utf8_content(attribute_name_index);
    let attribute_length: u32 = reader.read_u32().unwrap();
    match attribute_name.as_str() {
        "Signature" => {
            check_attribute_length(attribute_length, 2, attribute_name);
            let signature_index: u16 = reader.read_u16().unwrap();
            AttributeInfo::Signature {
                name_index: attribute_name_index,
                signature_index,
            }
        }
        "ConstantValue" => {
            let constant_value_index: u16 = reader.read_u16().unwrap();
            check_attribute_length(attribute_length, 2, attribute_name);
            AttributeInfo::ConstantValue {
                name_index: attribute_name_index,
                constant_value_index,
            }
        }
        _ => panic!(
            "The name '{attribute_name}' is either not of an attribute or not a field attribute.",
        ),
    }
}

pub fn parse_method_attributes(
    reader: &mut BinaryReader,
    cp: &ConstantPool,
    num_attributes: usize,
) -> Vec<AttributeInfo> {
    let mut attributes: Vec<AttributeInfo> = Vec::with_capacity(num_attributes);
    for i in 0..num_attributes {
        attributes.push(parse_method_attribute(reader, cp));
        for j in 0..i {
            assert!(
                attributes[i].kind() != attributes[j].kind(),
                "Found duplicate method attributes with kind {} at indices {} and {}.",
                attributes[i].kind(),
                i,
                j
            );
        }
    }
    attributes
}

fn parse_method_attribute(reader: &mut BinaryReader, cp: &ConstantPool) -> AttributeInfo {
    let attribute_name_index: u16 = reader.read_u16().unwrap();
    assert_valid_and_type!(cp, attribute_name_index, ConstantPoolTag::Utf8);
    let attribute_name: String = cp.get_utf8_content(attribute_name_index);
    let attribute_length: u32 = reader.read_u32().unwrap();
    match attribute_name.as_str() {
        "Code" => {
            let max_stack: u16 = reader.read_u16().unwrap();
            let max_locals: u16 = reader.read_u16().unwrap();
            let code_length: u32 = reader.read_u32().unwrap();
            assert!(
                code_length > 0 && code_length < 65_536,
                "Invalid code length: must be > 0 and < 65536 but was {code_length}.",
            );
            let code_bytes: Vec<u8> = reader.read_u8_vec(code_length.try_into().unwrap()).unwrap();
            let code: Vec<(u32, BytecodeInstruction)> = parse_bytecode(
                &mut BinaryReader::new(&code_bytes, binary_reader::Endianness::Big),
                cp,
            );
            let exception_table_length: u16 = reader.read_u16().unwrap();
            let mut exception_table: Vec<ExceptionTableEntry> =
                Vec::with_capacity(exception_table_length.into());
            for i in 0..exception_table_length {
                let start_pc: u16 = reader.read_u16().unwrap();
                let end_pc: u16 = reader.read_u16().unwrap();
                assert!(
                    start_pc < end_pc,
                    "Exception {i} has start_pc ({start_pc}) >= end_pc ({end_pc}).",
                );
                assert!(
                    code.iter()
                        .any(|(position, _)| *position == (start_pc as u32)),
                    "Exception {i} has start_pc ({start_pc}) which does not correspond to a valid instruction.",
                );
                assert!(
                    code.iter()
                        .any(|(position, _)| *position == (end_pc as u32))
                        || (end_pc as u32) == code_length,
                    "Exception {i} has end_pc ({end_pc}) which does not correspond to a valid instruction.",
                );
                let handler_pc: u16 = reader.read_u16().unwrap();
                assert!(
                    code.iter()
                        .any(|(position, _)| *position == (handler_pc as u32)),
                    "Exception {i} has handler_pc ({handler_pc}) which does not correspond to a valid instruction.",
                );
                let catch_type: u16 = reader.read_u16().unwrap();
                if catch_type != 0 {
                    assert_valid_and_type!(cp, catch_type, ConstantPoolTag::Class);
                }
                exception_table.push(ExceptionTableEntry {
                    start_pc,
                    end_pc,
                    handler_pc,
                    catch_type,
                });
            }
            let attribute_count: u16 = reader.read_u16().unwrap();
            let attributes: Vec<AttributeInfo> =
                parse_code_attributes(reader, cp, attribute_count.into(), &code, code_length);
            AttributeInfo::Code {
                name_index: attribute_name_index,
                max_stack,
                max_locals,
                code,
                exception_table,
                attributes,
            }
        }
        "MethodParameters" => {
            let parameters_count: u8 = reader.read_u8().unwrap();
            let expected_attribute_length = 1 + (2 * 2) * (parameters_count as u32);
            check_attribute_length(expected_attribute_length, attribute_length, attribute_name);
            let mut parameters: Vec<MethodParameter> = Vec::with_capacity(parameters_count.into());
            for _ in 0..parameters_count {
                let name_index: u16 = reader.read_u16().unwrap();
                if name_index != 0 {
                    assert_valid_and_type!(cp, name_index, ConstantPoolTag::Utf8);
                }
                let access_flags: MethodParameterAccessFlags =
                    MethodParameterAccessFlags::from(reader.read_u16().unwrap());
                parameters.push(MethodParameter {
                    name_index,
                    access_flags,
                });
            }
            AttributeInfo::MethodParameters {
                name_index: attribute_name_index,
                parameters,
            }
        }
        "Signature" => {
            check_attribute_length(attribute_length, 2, attribute_name);
            let signature_index: u16 = reader.read_u16().unwrap();
            assert_valid_and_type!(cp, signature_index, ConstantPoolTag::Utf8);
            AttributeInfo::Signature {
                name_index: attribute_name_index,
                signature_index,
            }
        }
        "RuntimeVisibleAnnotations" => {
            let num_annotations: u16 = reader.read_u16().unwrap();
            let mut annotations: Vec<Annotation> = Vec::with_capacity(num_annotations.into());
            for _ in 0..num_annotations {
                annotations.push(parse_annotation(cp, reader));
            }
            let expected_attribute_length = 2 + annotations
                .iter()
                .map(|ann| get_annotation_length(ann))
                .sum::<u32>();
            check_attribute_length(expected_attribute_length, attribute_length, attribute_name);
            AttributeInfo::RuntimeVisibleAnnotations {
                name_index: attribute_name_index,
                annotations,
            }
        }
        "Exceptions" => {
            let num_exceptions: u16 = reader.read_u16().unwrap();
            let expected_attribute_length = 2 + 2 * (num_exceptions as u32);
            check_attribute_length(expected_attribute_length, attribute_length, attribute_name);
            let exception_indices = reader.read_u16_vec(num_exceptions.into()).unwrap();
            for exception_index in exception_indices.iter() {
                assert_valid_and_type!(cp, *exception_index, ConstantPoolTag::Class);
            }
            AttributeInfo::Exceptions {
                name_index: attribute_name_index,
                exception_indices,
            }
        }
        _ => panic!(
            "The name '{attribute_name}' is either not of an attribute or not a method attribute.",
        ),
    }
}

fn parse_annotation(cp: &ConstantPool, reader: &mut BinaryReader) -> Annotation {
    let type_index: u16 = reader.read_u16().unwrap();
    assert_valid_and_type!(cp, type_index, ConstantPoolTag::Utf8);
    let num_element_value_pairs: u16 = reader.read_u16().unwrap();
    let mut element_value_pairs: Vec<ElementValuePair> =
        Vec::with_capacity(num_element_value_pairs.into());
    for _ in 0..num_element_value_pairs {
        let element_name_index: u16 = reader.read_u16().unwrap();
        assert_valid_and_type!(cp, element_name_index, ConstantPoolTag::Utf8);
        let value: ElementValue = parse_element_value(cp, reader);
        element_value_pairs.push(ElementValuePair {
            element_name_index,
            value,
        });
    }
    Annotation {
        type_index,
        element_value_pairs,
    }
}

fn parse_element_value(cp: &ConstantPool, reader: &mut BinaryReader) -> ElementValue {
    let element_value_tag: char = reader.read_u8().unwrap() as char;
    match element_value_tag {
        'B' => ElementValue::Byte {
            const_value_index: reader.read_u16().unwrap(),
        },
        'C' => ElementValue::Char {
            const_value_index: reader.read_u16().unwrap(),
        },
        'D' => ElementValue::Double {
            const_value_index: reader.read_u16().unwrap(),
        },
        'F' => ElementValue::Float {
            const_value_index: reader.read_u16().unwrap(),
        },
        'I' => ElementValue::Int {
            const_value_index: reader.read_u16().unwrap(),
        },
        'J' => ElementValue::Long {
            const_value_index: reader.read_u16().unwrap(),
        },
        'S' => ElementValue::Short {
            const_value_index: reader.read_u16().unwrap(),
        },
        'Z' => ElementValue::Boolean {
            const_value_index: reader.read_u16().unwrap(),
        },
        's' => ElementValue::String {
            const_value_index: reader.read_u16().unwrap(),
        },
        'e' => ElementValue::Enum {
            type_name_index: reader.read_u16().unwrap(),
            const_name_index: reader.read_u16().unwrap(),
        },
        'c' => ElementValue::Class {
            class_info_index: reader.read_u16().unwrap(),
        },
        '@' => ElementValue::Annotation {
            value: parse_annotation(cp, reader),
        },
        '[' => {
            let num_values: u16 = reader.read_u16().unwrap();
            let mut values: Vec<ElementValue> = Vec::with_capacity(num_values.into());
            for _ in 0..num_values {
                values.push(parse_element_value(cp, reader));
            }
            ElementValue::Array { values }
        }
        _ => panic!("'{element_value_tag}' is not a valid element value tag."),
    }
}

fn parse_code_attributes(
    reader: &mut BinaryReader,
    cp: &ConstantPool,
    num_attributes: usize,
    code: &[(u32, BytecodeInstruction)],
    code_length: u32,
) -> Vec<AttributeInfo> {
    let mut attributes: Vec<AttributeInfo> = Vec::with_capacity(num_attributes);
    for i in 0..num_attributes {
        attributes.push(parse_code_attribute(cp, reader, code, code_length));
        for j in 0..i {
            assert!(
                attributes[i].kind() != attributes[j].kind(),
                "Found duplicate code attributes with kind {} at indices {} and {}.",
                attributes[i].kind(),
                i,
                j
            );
        }
    }
    attributes
}

fn parse_code_attribute(
    cp: &ConstantPool,
    reader: &mut BinaryReader,
    code: &[(u32, BytecodeInstruction)],
    code_length: u32,
) -> AttributeInfo {
    let attribute_name_index: u16 = reader.read_u16().unwrap();
    assert_valid_and_type!(cp, attribute_name_index, ConstantPoolTag::Utf8);
    let attribute_name: String = cp.get_utf8_content(attribute_name_index);
    let attribute_length: u32 = reader.read_u32().unwrap();
    match attribute_name.as_str() {
        "LineNumberTable" => {
            let line_number_table_length: u16 = reader.read_u16().unwrap();
            let expected_attribute_length: u32 = 2 + (2 * 2) * (line_number_table_length as u32);
            check_attribute_length(expected_attribute_length, attribute_length, attribute_name);
            let mut line_number_table: Vec<LineNumberTableEntry> =
                Vec::with_capacity(line_number_table_length.into());
            for i in 0..line_number_table_length {
                let start_pc: u16 = reader.read_u16().unwrap();
                assert!(
                    code.iter()
                        .any(|(position, _)| *position == (start_pc as u32)),
                    "LineNumberTable entry {i} has start_pc ({start_pc}) which does not correspond to a valid instruction.",
                );
                let line_number: u16 = reader.read_u16().unwrap();
                line_number_table.push(LineNumberTableEntry {
                    start_pc,
                    line_number,
                });
            }
            AttributeInfo::LineNumberTable {
                name_index: attribute_name_index,
                line_number_table,
            }
        }
        "LocalVariableTable" => {
            let local_variable_table_length: u16 = reader.read_u16().unwrap();
            let expected_attribute_length: u32 = 2 + (2 * 5) * (local_variable_table_length as u32);
            check_attribute_length(expected_attribute_length, attribute_length, attribute_name);
            let mut local_variable_table: Vec<LocalVariableTableEntry> =
                Vec::with_capacity(local_variable_table_length.into());
            for i in 0..local_variable_table_length {
                let start_pc: u16 = reader.read_u16().unwrap();
                assert!(
                    code.iter()
                        .any(|(position, _)| *position == (start_pc as u32)),
                    "LocalVariableTable entry {i} has start_pc ({start_pc}) which does not correspond to a valid instruction.",
                );
                let length: u16 = reader.read_u16().unwrap();
                assert!(
                    code.iter()
                        .any(|(position, _)| *position == ((start_pc + length) as u32))
                        || ((start_pc + length) as u32) == code_length,
                    "LocalVariableTable entry {i} has start_pc + length ({start_pc} + {length}) which does not correspond to a valid instruction.",
                );
                let name_index: u16 = reader.read_u16().unwrap();
                assert_valid_and_type!(cp, name_index, ConstantPoolTag::Utf8);
                let descriptor_index: u16 = reader.read_u16().unwrap();
                assert_valid_and_type!(cp, descriptor_index, ConstantPoolTag::Utf8);
                let index: u16 = reader.read_u16().unwrap();
                local_variable_table.push(LocalVariableTableEntry {
                    start_pc,
                    length,
                    name_index,
                    descriptor_index,
                    index,
                });
            }
            AttributeInfo::LocalVariableTable {
                name_index: attribute_name_index,
                local_variable_table,
            }
        }
        "LocalVariableTypeTable" => {
            let local_variable_type_table_length: u16 = reader.read_u16().unwrap();
            let expected_attribute_length: u32 =
                2 + (2 * 5) * (local_variable_type_table_length as u32);
            check_attribute_length(expected_attribute_length, attribute_length, attribute_name);
            let mut local_variable_type_table: Vec<LocalVariableTypeTableEntry> =
                Vec::with_capacity(local_variable_type_table_length.into());
            for i in 0..local_variable_type_table_length {
                let start_pc: u16 = reader.read_u16().unwrap();
                assert!(
                    code.iter()
                        .any(|(position, _)| *position == (start_pc as u32)),
                    "LocalVariableTypeTable entry {i} has start_pc ({start_pc}) which does not correspond to a valid instruction.",
                );
                let length: u16 = reader.read_u16().unwrap();
                assert!(
                    code.iter()
                        .any(|(position, _)| *position == ((start_pc + length) as u32))
                        || ((start_pc + length) as u32) == code_length,
                    "LocalVariableTypeTable entry {i} has start_pc + length ({start_pc} + {length}) which does not correspond to a valid instruction.",
                );
                let name_index: u16 = reader.read_u16().unwrap();
                assert_valid_and_type!(cp, name_index, ConstantPoolTag::Utf8);
                let descriptor_index: u16 = reader.read_u16().unwrap();
                assert_valid_and_type!(cp, descriptor_index, ConstantPoolTag::Utf8);
                let index: u16 = reader.read_u16().unwrap();
                local_variable_type_table.push(LocalVariableTypeTableEntry {
                    start_pc,
                    length,
                    name_index,
                    descriptor_index,
                    index,
                });
            }
            AttributeInfo::LocalVariableTypeTable {
                name_index: attribute_name_index,
                local_variable_type_table,
            }
        }
        "StackMapTable" => {
            let number_of_entries: u16 = reader.read_u16().unwrap();
            let mut stack_map_table: Vec<StackMapFrame> =
                Vec::with_capacity(number_of_entries.into());
            for _ in 0..number_of_entries {
                stack_map_table.push(parse_stack_map_entry(reader));
            }
            let expected_attribute_length = 2 + stack_map_table
                .iter()
                .map(|smf| get_stack_map_entry_length(smf))
                .sum::<u32>();
            check_attribute_length(expected_attribute_length, attribute_length, attribute_name);
            AttributeInfo::StackMapTable {
                name_index: attribute_name_index,
                stack_map_table,
            }
        }
        _ => panic!(
            "The name '{attribute_name}' is either not of an attribute or not a code attribute."
        ),
    }
}

fn parse_stack_map_entry(reader: &mut BinaryReader) -> StackMapFrame {
    let frame_type: u8 = reader.read_u8().unwrap();
    match frame_type {
        0..=63 => StackMapFrame::SameFrame { frame_type },
        64..=127 => StackMapFrame::SameLocals1StackItemFrame {
            frame_type,
            stack: parse_verification_type_info(reader),
        },
        128..=246 => panic!("Frame type {frame_type} is reserved."),
        247 => StackMapFrame::SameLocals1StackItemFrameExtended {
            offset_delta: reader.read_u16().unwrap(),
            stack: parse_verification_type_info(reader),
        },
        248..=250 => StackMapFrame::ChopFrame {
            frame_type,
            offset_delta: reader.read_u16().unwrap(),
        },
        251 => StackMapFrame::SameFrameExtended {
            offset_delta: reader.read_u16().unwrap(),
        },
        252..=254 => StackMapFrame::AppendFrame {
            frame_type,
            offset_delta: reader.read_u16().unwrap(),
            locals: parse_verification_type_info_vec(reader, (frame_type - 251).into()),
        },
        255 => {
            let offset_delta: u16 = reader.read_u16().unwrap();
            let number_of_locals: u16 = reader.read_u16().unwrap();
            let locals: Vec<VerificationTypeInfo> =
                parse_verification_type_info_vec(reader, number_of_locals.into());
            let number_of_stack_items: u16 = reader.read_u16().unwrap();
            let stack: Vec<VerificationTypeInfo> =
                parse_verification_type_info_vec(reader, number_of_stack_items.into());
            StackMapFrame::FullFrame {
                offset_delta,
                locals,
                stack,
            }
        }
    }
}

fn parse_verification_type_info_vec(
    reader: &mut BinaryReader,
    num: usize,
) -> Vec<VerificationTypeInfo> {
    let mut result: Vec<VerificationTypeInfo> = Vec::with_capacity(num);
    for _ in 0..num {
        result.push(parse_verification_type_info(reader));
    }
    result
}

fn parse_verification_type_info(reader: &mut BinaryReader) -> VerificationTypeInfo {
    let tag: u8 = reader.read_u8().unwrap();
    match tag {
        0 => VerificationTypeInfo::TopVariable,
        1 => VerificationTypeInfo::IntegerVariable,
        2 => VerificationTypeInfo::FloatVariable,
        3 => VerificationTypeInfo::DoubleVariable,
        4 => VerificationTypeInfo::LongVariable,
        5 => VerificationTypeInfo::NullVariable,
        6 => VerificationTypeInfo::UninitializedThisVariable,
        7 => VerificationTypeInfo::ObjectVariable {
            constant_pool_index: reader.read_u16().unwrap(),
        },
        8 => VerificationTypeInfo::UninitializedVariable {
            offset: reader.read_u16().unwrap(),
        },
        _ => panic!("Wrong verification type info tag {tag}."),
    }
}

pub fn find_attribute(attributes: &[AttributeInfo], kind: AttributeKind) -> Option<&AttributeInfo> {
    attributes.iter().find(|a| a.kind() == kind)
}
