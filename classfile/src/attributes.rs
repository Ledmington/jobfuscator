#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use binary_reader::BinaryReader;

use crate::access_flags::{self, AccessFlag};
use crate::bytecode::{BytecodeInstruction, parse_bytecode};
use crate::constant_pool::ConstantPool;

pub enum AttributeInfo {
    Code {
        max_stack: u16,
        max_locals: u16,
        code: BTreeMap<u32, BytecodeInstruction>,
        exception_table: Vec<ExceptionTableEntry>,
        attributes: Vec<AttributeInfo>,
    },
    LineNumberTable {
        line_number_table: Vec<LineNumberTableEntry>,
    },
    LocalVariableTable {
        local_variable_table: Vec<LocalVariableTableEntry>,
    },
    StackMapTable {
        stack_map_table: Vec<StackMapFrame>,
    },
    SourceFile {
        source_file_index: u16,
    },
    BootstrapMethods {
        methods: Vec<BootstrapMethod>,
    },
    InnerClasses {
        classes: Vec<Class>,
    },
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

// TODO: find a better name
pub struct Class {
    pub inner_class_info_index: u16,
    pub outer_class_info_index: u16,
    pub inner_name_index: u16,
    pub inner_class_access_flags: Vec<AccessFlag>,
}

pub fn parse_class_attributes(
    reader: &mut BinaryReader,
    cp: &ConstantPool,
    num_attributes: usize,
) -> Vec<AttributeInfo> {
    let mut attributes: Vec<AttributeInfo> = Vec::with_capacity(num_attributes);
    for _ in 0..num_attributes {
        attributes.push(parse_class_attribute(reader, cp));
    }
    attributes
}

fn parse_class_attribute(reader: &mut BinaryReader, cp: &ConstantPool) -> AttributeInfo {
    let attribute_name_index: u16 = reader.read_u16().unwrap();
    let attribute_name: String = cp.get_utf8_content(attribute_name_index);
    let _attribute_length: u32 = reader.read_u32().unwrap(); // ignored
    match attribute_name.as_str() {
        "SourceFile" => AttributeInfo::SourceFile {
            source_file_index: reader.read_u16().unwrap(),
        },
        "BootstrapMethods" => {
            let num_bootstrap_methods: u16 = reader.read_u16().unwrap();
            let mut methods: Vec<BootstrapMethod> =
                Vec::with_capacity(num_bootstrap_methods.into());
            for _ in 0..num_bootstrap_methods {
                let bootstrap_method_ref: u16 = reader.read_u16().unwrap();
                let num_bootstrap_arguments: u16 = reader.read_u16().unwrap();
                let bootstrap_arguments: Vec<u16> =
                    reader.read_u16_vec(num_bootstrap_arguments.into()).unwrap();
                methods.push(BootstrapMethod {
                    bootstrap_method_ref,
                    bootstrap_arguments,
                });
            }
            AttributeInfo::BootstrapMethods { methods }
        }
        "InnerClasses" => {
            let number_of_classes: u16 = reader.read_u16().unwrap();
            let mut classes: Vec<Class> = Vec::with_capacity(number_of_classes.into());
            for _ in 0..number_of_classes {
                classes.push(Class {
                    inner_class_info_index: reader.read_u16().unwrap(),
                    outer_class_info_index: reader.read_u16().unwrap(),
                    inner_name_index: reader.read_u16().unwrap(),
                    inner_class_access_flags: access_flags::parse_access_flags(
                        reader.read_u16().unwrap(),
                    ),
                });
            }
            AttributeInfo::InnerClasses { classes }
        }
        _ => panic!(
            "The name '{}' is either not of an attribute or not a class attribute.",
            attribute_name
        ),
    }
}

pub fn parse_field_attributes(
    reader: &mut BinaryReader,
    cp: &ConstantPool,
    num_attributes: usize,
) -> Vec<AttributeInfo> {
    let mut attributes: Vec<AttributeInfo> = Vec::with_capacity(num_attributes);
    for _ in 0..num_attributes {
        attributes.push(parse_field_attribute(cp, reader));
    }
    attributes
}

fn parse_field_attribute(_cp: &ConstantPool, _reader: &mut BinaryReader) -> AttributeInfo {
    unreachable!()
}

pub fn parse_method_attributes(
    reader: &mut BinaryReader,
    cp: &ConstantPool,
    num_attributes: usize,
) -> Vec<AttributeInfo> {
    let mut attributes: Vec<AttributeInfo> = Vec::with_capacity(num_attributes);
    for _ in 0..num_attributes {
        attributes.push(parse_method_attribute(cp, reader));
    }
    attributes
}

fn parse_method_attribute(cp: &ConstantPool, reader: &mut BinaryReader) -> AttributeInfo {
    let attribute_name_index: u16 = reader.read_u16().unwrap();
    let attribute_name: String = cp.get_utf8_content(attribute_name_index);
    let _attribute_length: u32 = reader.read_u32().unwrap(); // ignored
    match attribute_name.as_str() {
        "Code" => {
            let max_stack: u16 = reader.read_u16().unwrap();
            let max_locals: u16 = reader.read_u16().unwrap();
            let code_length: u32 = reader.read_u32().unwrap();
            let code_bytes: Vec<u8> = reader.read_u8_vec(code_length.try_into().unwrap()).unwrap();
            let code: BTreeMap<u32, BytecodeInstruction> = parse_bytecode(&mut BinaryReader::new(
                &code_bytes,
                binary_reader::Endian::Big,
            ));
            let exception_table_length: u16 = reader.read_u16().unwrap();
            let mut exception_table: Vec<ExceptionTableEntry> =
                Vec::with_capacity(exception_table_length.into());
            for _ in 0..exception_table_length {
                let start_pc: u16 = reader.read_u16().unwrap();
                let end_pc: u16 = reader.read_u16().unwrap();
                let handler_pc: u16 = reader.read_u16().unwrap();
                let catch_type: u16 = reader.read_u16().unwrap();
                exception_table.push(ExceptionTableEntry {
                    start_pc,
                    end_pc,
                    handler_pc,
                    catch_type,
                });
            }
            let attribute_count: u16 = reader.read_u16().unwrap();
            let attributes: Vec<AttributeInfo> =
                parse_code_attributes(reader, cp, attribute_count.into());
            AttributeInfo::Code {
                max_stack,
                max_locals,
                code,
                exception_table,
                attributes,
            }
        }
        _ => panic!(
            "The name '{}' is either not of an attribute or not a method attribute.",
            attribute_name
        ),
    }
}

fn parse_code_attributes(
    reader: &mut BinaryReader,
    cp: &ConstantPool,
    num_attributes: usize,
) -> Vec<AttributeInfo> {
    let mut attributes: Vec<AttributeInfo> = Vec::with_capacity(num_attributes);
    for _ in 0..num_attributes {
        attributes.push(parse_code_attribute(cp, reader));
    }
    attributes
}

fn parse_code_attribute(cp: &ConstantPool, reader: &mut BinaryReader) -> AttributeInfo {
    let attribute_name_index: u16 = reader.read_u16().unwrap();
    let attribute_name: String = cp.get_utf8_content(attribute_name_index);
    let _attribute_length: u32 = reader.read_u32().unwrap(); // ignored
    match attribute_name.as_str() {
        "LineNumberTable" => {
            let line_number_table_length: u16 = reader.read_u16().unwrap();
            let mut line_number_table: Vec<LineNumberTableEntry> =
                Vec::with_capacity(line_number_table_length.into());
            for _ in 0..line_number_table_length {
                let start_pc: u16 = reader.read_u16().unwrap();
                let line_number: u16 = reader.read_u16().unwrap();
                line_number_table.push(LineNumberTableEntry {
                    start_pc,
                    line_number,
                });
            }
            AttributeInfo::LineNumberTable { line_number_table }
        }
        "LocalVariableTable" => {
            let local_variable_table_length: u16 = reader.read_u16().unwrap();
            let mut local_variable_table: Vec<LocalVariableTableEntry> =
                Vec::with_capacity(local_variable_table_length.into());
            for _ in 0..local_variable_table_length {
                let start_pc: u16 = reader.read_u16().unwrap();
                let length: u16 = reader.read_u16().unwrap();
                let name_index: u16 = reader.read_u16().unwrap();
                let descriptor_index: u16 = reader.read_u16().unwrap();
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
                local_variable_table,
            }
        }
        "StackMapTable" => {
            let number_of_entries: u16 = reader.read_u16().unwrap();
            let mut stack_map_table: Vec<StackMapFrame> =
                Vec::with_capacity(number_of_entries.into());
            for _ in 0..number_of_entries {
                stack_map_table.push(parse_stack_map_entry(reader));
            }
            AttributeInfo::StackMapTable { stack_map_table }
        }
        _ => panic!(
            "The name '{}' is either not of an attribute or not a code attribute.",
            attribute_name
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
        128..=246 => panic!("Frame type {} is reserved.", frame_type),
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
        _ => panic!("Wrong verification type info tag {}", tag),
    }
}
