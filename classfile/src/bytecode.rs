#![forbid(unsafe_code)]

use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
    io::Result,
};

use binary_reader::BinaryReader;
use binary_writer::BinaryWriter;

use crate::constant_pool::{ConstantPool, ConstantPoolTag, assert_valid_and_type};

/**
 * Reference available at <https://docs.oracle.com/javase/specs/jvms/se25/html/jvms-6.html#jvms-6.5>
 */
pub enum BytecodeInstruction {
    Dup {},
    AConstNull {},
    IConst {
        constant: i32,
    },
    LConst {
        constant: i64,
    },
    FConst {
        constant: f32,
    },
    DConst {
        constant: f64,
    },
    Ldc {
        constant_pool_index: u8,
    },
    LdcW {
        constant_pool_index: u16,
    },
    Ldc2W {
        constant_pool_index: u16,
    },
    ALoad {
        local_variable_index: u8,
    },
    AStore {
        local_variable_index: u8,
    },
    ILoad {
        local_variable_index: u8,
    },
    IStore {
        local_variable_index: u8,
    },
    LLoad {
        local_variable_index: u8,
    },
    LStore {
        local_variable_index: u8,
    },
    FLoad {
        local_variable_index: u8,
    },
    FStore {
        local_variable_index: u8,
    },
    DLoad {
        local_variable_index: u8,
    },
    DStore {
        local_variable_index: u8,
    },
    AaLoad {},
    BaLoad {},
    AaStore {},
    BaStore {},
    CaStore {},
    SaStore {},
    NewArray {
        atype: ArrayType,
    },
    ANewArray {
        constant_pool_index: u16,
    },
    AThrow {},
    New {
        constant_pool_index: u16,
    },
    BiPush {
        immediate: u8,
    },
    SiPush {
        immediate: u16,
    },
    Pop {},
    Pop2 {},
    Return {},
    IReturn {},
    LReturn {},
    FReturn {},
    DReturn {},
    AReturn {},
    GetStatic {
        field_ref_index: u16,
    },
    PutStatic {
        field_ref_index: u16,
    },
    GetField {
        field_ref_index: u16,
    },
    PutField {
        field_ref_index: u16,
    },
    InvokeSpecial {
        method_ref_index: u16,
    },
    InvokeStatic {
        method_ref_index: u16,
    },
    InvokeVirtual {
        method_ref_index: u16,
    },
    InvokeDynamic {
        constant_pool_index: u16,
    },
    InvokeInterface {
        constant_pool_index: u16,
        count: u8,
    },
    ArrayLength {},
    LCmp {},
    FCmpL {},
    FCmpG {},
    DCmpL {},
    DCmpG {},
    IfAcmpEq {
        offset: i16,
    },
    IfAcmpNe {
        offset: i16,
    },
    IfIcmpEq {
        offset: i16,
    },
    IfIcmpNe {
        offset: i16,
    },
    IfIcmpLt {
        offset: i16,
    },
    IfIcmpGe {
        offset: i16,
    },
    IfIcmpGt {
        offset: i16,
    },
    IfIcmpLe {
        offset: i16,
    },
    IfEq {
        offset: i16,
    },
    IfNe {
        offset: i16,
    },
    IfLt {
        offset: i16,
    },
    IfGe {
        offset: i16,
    },
    IfGt {
        offset: i16,
    },
    IfLe {
        offset: i16,
    },
    IfNull {
        offset: i16,
    },
    IfNonNull {
        offset: i16,
    },
    GoTo {
        offset: i16,
    },
    TableSwitch {
        default: i32,
        low: i32,
        offsets: Vec<i32>,
    },
    LookupSwitch {
        default: i32,
        pairs: Vec<LookupSwitchPair>,
    },
    CheckCast {
        constant_pool_index: u16,
    },
    Instanceof {
        constant_pool_index: u16,
    },
    IInc {
        index: u8,
        constant: i8,
    },

    I2L {},
    I2F {},
    I2D {},
    L2I {},
    L2F {},
    L2D {},
    F2I {},
    F2L {},
    F2D {},
    D2I {},
    D2L {},
    D2F {},
    I2B {},
    I2C {},
    I2S {},

    IAdd {},
    ISub {},
    IMul {},
    IDiv {},
    IRem {},
    IAnd {},
    IShl {},
    IShr {},
    IUshr {},
    IOr {},
    IXor {},
    INeg {},

    LAdd {},
    LSub {},
    LMul {},
    LDiv {},
    LRem {},
    LAnd {},
    LOr {},
    LXor {},
    LShl {},
    LShr {},
    LUshr {},
    LNeg {},

    FAdd {},
    FMul {},
    FNeg {},
    FDiv {},
    FRem {},
    FSub {},

    DAdd {},
    DMul {},
    DNeg {},
    DDiv {},
    DRem {},
    DSub {},
}

pub struct LookupSwitchPair {
    pub match_value: i32,
    pub offset: i32,
}

#[repr(u8)]
pub enum ArrayType {
    Boolean = 4,
    Char = 5,
    Float = 6,
    Double = 7,
    Byte = 8,
    Short = 9,
    Int = 10,
    Long = 11,
}

impl From<u8> for ArrayType {
    fn from(value: u8) -> Self {
        match value {
            4 => ArrayType::Boolean,
            5 => ArrayType::Char,
            6 => ArrayType::Float,
            7 => ArrayType::Double,
            8 => ArrayType::Byte,
            9 => ArrayType::Short,
            10 => ArrayType::Int,
            11 => ArrayType::Long,
            _ => panic!("Unknown array type value {}.", value),
        }
    }
}

impl Display for ArrayType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ArrayType::Boolean => write!(f, "boolean"),
            ArrayType::Char => write!(f, "char"),
            ArrayType::Float => write!(f, "float"),
            ArrayType::Double => write!(f, "double"),
            ArrayType::Byte => write!(f, "byte"),
            ArrayType::Short => write!(f, "short"),
            ArrayType::Int => write!(f, "int"),
            ArrayType::Long => write!(f, "long"),
        }
    }
}

pub fn parse_bytecode(
    reader: &mut BinaryReader,
    cp: &ConstantPool,
) -> BTreeMap<u32, BytecodeInstruction> {
    let mut instructions: BTreeMap<u32, BytecodeInstruction> = BTreeMap::new();
    while reader.position() < reader.len() {
        let position: u32 = reader.position().try_into().unwrap();
        let tmp: Result<u8> = reader.read_u8();
        if tmp.is_err() {
            break;
        }
        let opcode: u8 = tmp.unwrap();
        instructions.insert(
            position,
            match opcode {
                0x01 => BytecodeInstruction::AConstNull {},
                0x02 => BytecodeInstruction::IConst { constant: -1 },
                0x03 => BytecodeInstruction::IConst { constant: 0 },
                0x04 => BytecodeInstruction::IConst { constant: 1 },
                0x05 => BytecodeInstruction::IConst { constant: 2 },
                0x06 => BytecodeInstruction::IConst { constant: 3 },
                0x07 => BytecodeInstruction::IConst { constant: 4 },
                0x08 => BytecodeInstruction::IConst { constant: 5 },
                0x09 => BytecodeInstruction::LConst { constant: 0 },
                0x0a => BytecodeInstruction::LConst { constant: 1 },
                0x0b => BytecodeInstruction::FConst { constant: 0.0f32 },
                0x0c => BytecodeInstruction::FConst { constant: 1.0f32 },
                0x0d => BytecodeInstruction::FConst { constant: 2.0f32 },
                0x0e => BytecodeInstruction::DConst { constant: 0.0 },
                0x0f => BytecodeInstruction::DConst { constant: 1.0 },
                0x10 => BytecodeInstruction::BiPush {
                    immediate: reader.read_u8().unwrap(),
                },
                0x11 => BytecodeInstruction::SiPush {
                    immediate: reader.read_u16().unwrap(),
                },
                0x12 => BytecodeInstruction::Ldc {
                    constant_pool_index: reader.read_u8().unwrap(),
                },
                0x13 => BytecodeInstruction::LdcW {
                    constant_pool_index: reader.read_u16().unwrap(),
                },
                0x14 => BytecodeInstruction::Ldc2W {
                    constant_pool_index: reader.read_u16().unwrap(),
                },
                0x15 => BytecodeInstruction::ILoad {
                    local_variable_index: reader.read_u8().unwrap(),
                },
                0x16 => BytecodeInstruction::LLoad {
                    local_variable_index: reader.read_u8().unwrap(),
                },
                0x17 => BytecodeInstruction::FLoad {
                    local_variable_index: reader.read_u8().unwrap(),
                },
                0x18 => BytecodeInstruction::DLoad {
                    local_variable_index: reader.read_u8().unwrap(),
                },
                0x19 => BytecodeInstruction::ALoad {
                    local_variable_index: reader.read_u8().unwrap(),
                },
                0x1a => BytecodeInstruction::ILoad {
                    local_variable_index: 0,
                },
                0x1b => BytecodeInstruction::ILoad {
                    local_variable_index: 1,
                },
                0x1c => BytecodeInstruction::ILoad {
                    local_variable_index: 2,
                },
                0x1d => BytecodeInstruction::ILoad {
                    local_variable_index: 3,
                },
                0x1e => BytecodeInstruction::LLoad {
                    local_variable_index: 0,
                },
                0x1f => BytecodeInstruction::LLoad {
                    local_variable_index: 1,
                },
                0x20 => BytecodeInstruction::LLoad {
                    local_variable_index: 2,
                },
                0x21 => BytecodeInstruction::LLoad {
                    local_variable_index: 3,
                },
                0x22 => BytecodeInstruction::FLoad {
                    local_variable_index: 0,
                },
                0x23 => BytecodeInstruction::FLoad {
                    local_variable_index: 1,
                },
                0x24 => BytecodeInstruction::FLoad {
                    local_variable_index: 2,
                },
                0x25 => BytecodeInstruction::FLoad {
                    local_variable_index: 3,
                },
                0x26 => BytecodeInstruction::DLoad {
                    local_variable_index: 0,
                },
                0x27 => BytecodeInstruction::DLoad {
                    local_variable_index: 1,
                },
                0x28 => BytecodeInstruction::DLoad {
                    local_variable_index: 2,
                },
                0x29 => BytecodeInstruction::DLoad {
                    local_variable_index: 3,
                },
                0x2a => BytecodeInstruction::ALoad {
                    local_variable_index: 0,
                },
                0x2b => BytecodeInstruction::ALoad {
                    local_variable_index: 1,
                },
                0x2c => BytecodeInstruction::ALoad {
                    local_variable_index: 2,
                },
                0x2d => BytecodeInstruction::ALoad {
                    local_variable_index: 3,
                },
                0x32 => BytecodeInstruction::AaLoad {},
                0x33 => BytecodeInstruction::BaLoad {},
                0x36 => BytecodeInstruction::IStore {
                    local_variable_index: reader.read_u8().unwrap(),
                },
                0x37 => BytecodeInstruction::LStore {
                    local_variable_index: reader.read_u8().unwrap(),
                },
                0x38 => BytecodeInstruction::FStore {
                    local_variable_index: reader.read_u8().unwrap(),
                },
                0x39 => BytecodeInstruction::DStore {
                    local_variable_index: reader.read_u8().unwrap(),
                },
                0x3a => BytecodeInstruction::AStore {
                    local_variable_index: reader.read_u8().unwrap(),
                },
                0x3b => BytecodeInstruction::IStore {
                    local_variable_index: 0,
                },
                0x3c => BytecodeInstruction::IStore {
                    local_variable_index: 1,
                },
                0x3d => BytecodeInstruction::IStore {
                    local_variable_index: 2,
                },
                0x3e => BytecodeInstruction::IStore {
                    local_variable_index: 3,
                },
                0x3f => BytecodeInstruction::LStore {
                    local_variable_index: 0,
                },
                0x40 => BytecodeInstruction::LStore {
                    local_variable_index: 1,
                },
                0x41 => BytecodeInstruction::LStore {
                    local_variable_index: 2,
                },
                0x42 => BytecodeInstruction::LStore {
                    local_variable_index: 3,
                },
                0x4b => BytecodeInstruction::AStore {
                    local_variable_index: 0,
                },
                0x4c => BytecodeInstruction::AStore {
                    local_variable_index: 1,
                },
                0x4d => BytecodeInstruction::AStore {
                    local_variable_index: 2,
                },
                0x4e => BytecodeInstruction::AStore {
                    local_variable_index: 3,
                },
                0x53 => BytecodeInstruction::AaStore {},
                0x54 => BytecodeInstruction::BaStore {},
                0x55 => BytecodeInstruction::CaStore {},
                0x56 => BytecodeInstruction::SaStore {},
                0x57 => BytecodeInstruction::Pop {},
                0x58 => BytecodeInstruction::Pop2 {},
                0x59 => BytecodeInstruction::Dup {},
                0x60 => BytecodeInstruction::IAdd {},
                0x61 => BytecodeInstruction::LAdd {},
                0x62 => BytecodeInstruction::FAdd {},
                0x63 => BytecodeInstruction::DAdd {},
                0x64 => BytecodeInstruction::ISub {},
                0x65 => BytecodeInstruction::LSub {},
                0x66 => BytecodeInstruction::FSub {},
                0x67 => BytecodeInstruction::DSub {},
                0x68 => BytecodeInstruction::IMul {},
                0x69 => BytecodeInstruction::LMul {},
                0x6a => BytecodeInstruction::FMul {},
                0x6b => BytecodeInstruction::DMul {},
                0x6c => BytecodeInstruction::IDiv {},
                0x6d => BytecodeInstruction::LDiv {},
                0x6e => BytecodeInstruction::FDiv {},
                0x6f => BytecodeInstruction::DDiv {},
                0x70 => BytecodeInstruction::IRem {},
                0x71 => BytecodeInstruction::LRem {},
                0x72 => BytecodeInstruction::FRem {},
                0x73 => BytecodeInstruction::DRem {},
                0x74 => BytecodeInstruction::INeg {},
                0x75 => BytecodeInstruction::LNeg {},
                0x76 => BytecodeInstruction::FNeg {},
                0x77 => BytecodeInstruction::DNeg {},
                0x78 => BytecodeInstruction::IShl {},
                0x79 => BytecodeInstruction::LShl {},
                0x7a => BytecodeInstruction::IShr {},
                0x7b => BytecodeInstruction::LShr {},
                0x7c => BytecodeInstruction::IUshr {},
                0x7d => BytecodeInstruction::LUshr {},
                0x7e => BytecodeInstruction::IAnd {},
                0x7f => BytecodeInstruction::LAnd {},
                0x80 => BytecodeInstruction::IOr {},
                0x81 => BytecodeInstruction::LOr {},
                0x82 => BytecodeInstruction::IXor {},
                0x83 => BytecodeInstruction::LXor {},
                0x84 => BytecodeInstruction::IInc {
                    index: reader.read_u8().unwrap(),
                    constant: reader.read_i8().unwrap(),
                },
                0x85 => BytecodeInstruction::I2L {},
                0x86 => BytecodeInstruction::I2F {},
                0x87 => BytecodeInstruction::I2D {},
                0x88 => BytecodeInstruction::L2I {},
                0x89 => BytecodeInstruction::L2F {},
                0x8a => BytecodeInstruction::L2D {},
                0x8b => BytecodeInstruction::F2I {},
                0x8c => BytecodeInstruction::F2L {},
                0x8d => BytecodeInstruction::F2D {},
                0x8e => BytecodeInstruction::D2I {},
                0x8f => BytecodeInstruction::D2L {},
                0x90 => BytecodeInstruction::D2F {},
                0x91 => BytecodeInstruction::I2B {},
                0x92 => BytecodeInstruction::I2C {},
                0x93 => BytecodeInstruction::I2S {},
                0x94 => BytecodeInstruction::LCmp {},
                0x95 => BytecodeInstruction::FCmpL {},
                0x96 => BytecodeInstruction::FCmpG {},
                0x97 => BytecodeInstruction::DCmpL {},
                0x98 => BytecodeInstruction::DCmpG {},
                0x99 => BytecodeInstruction::IfEq {
                    offset: reader.read_i16().unwrap(),
                },
                0x9a => BytecodeInstruction::IfNe {
                    offset: reader.read_i16().unwrap(),
                },
                0x9b => BytecodeInstruction::IfLt {
                    offset: reader.read_i16().unwrap(),
                },
                0x9c => BytecodeInstruction::IfGe {
                    offset: reader.read_i16().unwrap(),
                },
                0x9d => BytecodeInstruction::IfGt {
                    offset: reader.read_i16().unwrap(),
                },
                0x9e => BytecodeInstruction::IfLe {
                    offset: reader.read_i16().unwrap(),
                },
                0x9f => BytecodeInstruction::IfIcmpEq {
                    offset: reader.read_i16().unwrap(),
                },
                0xa0 => BytecodeInstruction::IfIcmpNe {
                    offset: reader.read_i16().unwrap(),
                },
                0xa1 => BytecodeInstruction::IfIcmpLt {
                    offset: reader.read_i16().unwrap(),
                },
                0xa2 => BytecodeInstruction::IfIcmpGe {
                    offset: reader.read_i16().unwrap(),
                },
                0xa3 => BytecodeInstruction::IfIcmpGt {
                    offset: reader.read_i16().unwrap(),
                },
                0xa4 => BytecodeInstruction::IfIcmpLe {
                    offset: reader.read_i16().unwrap(),
                },
                0xa5 => BytecodeInstruction::IfAcmpEq {
                    offset: reader.read_i16().unwrap(),
                },
                0xa6 => BytecodeInstruction::IfAcmpNe {
                    offset: reader.read_i16().unwrap(),
                },
                0xa7 => BytecodeInstruction::GoTo {
                    offset: reader.read_i16().unwrap(),
                },
                0xaa => {
                    // skip padding
                    while !reader.position().is_multiple_of(4) {
                        _ = reader.read_u8();
                    }
                    let default: i32 = reader.read_i32().unwrap();
                    let low: i32 = reader.read_i32().unwrap();
                    let high: i32 = reader.read_i32().unwrap();
                    let offsets: Vec<i32> = reader
                        .read_i32_vec((high - low + 1).try_into().unwrap())
                        .unwrap();
                    BytecodeInstruction::TableSwitch {
                        default,
                        low,
                        offsets,
                    }
                }
                0xab => {
                    // skip padding
                    while !reader.position().is_multiple_of(4) {
                        _ = reader.read_u8();
                    }
                    let default: i32 = reader.read_i32().unwrap();
                    let npairs: i32 = reader.read_i32().unwrap();
                    debug_assert!(npairs >= 0);
                    let mut pairs: Vec<LookupSwitchPair> =
                        Vec::with_capacity(npairs.try_into().unwrap());
                    for _ in 0..npairs {
                        let match_value: i32 = reader.read_i32().unwrap();
                        let offset: i32 = reader.read_i32().unwrap();
                        pairs.push(LookupSwitchPair {
                            match_value,
                            offset,
                        });
                    }
                    BytecodeInstruction::LookupSwitch { default, pairs }
                }
                0xac => BytecodeInstruction::IReturn {},
                0xad => BytecodeInstruction::LReturn {},
                0xae => BytecodeInstruction::FReturn {},
                0xaf => BytecodeInstruction::DReturn {},
                0xb0 => BytecodeInstruction::AReturn {},
                0xb1 => BytecodeInstruction::Return {},
                0xb2 => {
                    let field_ref_index: u16 = reader.read_u16().unwrap();
                    assert_valid_and_type(cp, field_ref_index, ConstantPoolTag::Fieldref);
                    BytecodeInstruction::GetStatic { field_ref_index }
                }
                0xb3 => BytecodeInstruction::PutStatic {
                    field_ref_index: reader.read_u16().unwrap(),
                },
                0xb4 => BytecodeInstruction::GetField {
                    field_ref_index: reader.read_u16().unwrap(),
                },
                0xb5 => BytecodeInstruction::PutField {
                    field_ref_index: reader.read_u16().unwrap(),
                },
                0xb6 => BytecodeInstruction::InvokeVirtual {
                    method_ref_index: reader.read_u16().unwrap(),
                },
                0xb7 => BytecodeInstruction::InvokeSpecial {
                    method_ref_index: reader.read_u16().unwrap(),
                },
                0xb8 => BytecodeInstruction::InvokeStatic {
                    method_ref_index: reader.read_u16().unwrap(),
                },
                0xb9 => {
                    let constant_pool_index: u16 = reader.read_u16().unwrap();
                    let count: u8 = reader.read_u8().unwrap();
                    // skip one zero byte
                    _ = reader.read_u8().unwrap();
                    BytecodeInstruction::InvokeInterface {
                        constant_pool_index,
                        count,
                    }
                }
                0xba => {
                    let constant_pool_index: u16 = reader.read_u16().unwrap();
                    // skip two zero bytes
                    _ = reader.read_u8();
                    _ = reader.read_u8();
                    BytecodeInstruction::InvokeDynamic {
                        constant_pool_index,
                    }
                }
                0xbb => BytecodeInstruction::New {
                    constant_pool_index: reader.read_u16().unwrap(),
                },
                0xbc => BytecodeInstruction::NewArray {
                    atype: ArrayType::from(reader.read_u8().unwrap()),
                },
                0xbd => BytecodeInstruction::ANewArray {
                    constant_pool_index: reader.read_u16().unwrap(),
                },
                0xbe => BytecodeInstruction::ArrayLength {},
                0xbf => BytecodeInstruction::AThrow {},
                0xc0 => BytecodeInstruction::CheckCast {
                    constant_pool_index: reader.read_u16().unwrap(),
                },
                0xc1 => BytecodeInstruction::Instanceof {
                    constant_pool_index: reader.read_u16().unwrap(),
                },
                0xc6 => BytecodeInstruction::IfNull {
                    offset: reader.read_i16().unwrap(),
                },
                0xc7 => BytecodeInstruction::IfNonNull {
                    offset: reader.read_i16().unwrap(),
                },
                _ => panic!("Unknown bytecode instruction 0x{:02x}", opcode),
            },
        );
    }
    instructions
}

pub fn write_instruction(w: &mut BinaryWriter, instruction: &BytecodeInstruction) {
    match instruction {
        BytecodeInstruction::Dup {} => todo!(),
        BytecodeInstruction::AConstNull {} => todo!(),
        BytecodeInstruction::IConst { .. } => todo!(),
        BytecodeInstruction::LConst { .. } => todo!(),
        BytecodeInstruction::FConst { .. } => todo!(),
        BytecodeInstruction::DConst { .. } => todo!(),
        BytecodeInstruction::Ldc {
            constant_pool_index,
        } => {
            w.write_u8(0x12);
            w.write_u8(*constant_pool_index);
        }
        BytecodeInstruction::LdcW { .. } => todo!(),
        BytecodeInstruction::Ldc2W {
            constant_pool_index,
        } => {
            w.write_u8(0x14);
            w.write_u16(*constant_pool_index);
        }
        BytecodeInstruction::ALoad {
            local_variable_index,
        } => match local_variable_index {
            0 => w.write_u8(0x2a),
            1 => w.write_u8(0x2b),
            2 => w.write_u8(0x2c),
            3 => w.write_u8(0x2d),
            _ => {
                w.write_u8(0x19);
                w.write_u8(*local_variable_index);
            }
        },
        BytecodeInstruction::AStore {
            local_variable_index,
        } => match local_variable_index {
            0 => w.write_u8(0x4b),
            1 => w.write_u8(0x4c),
            2 => w.write_u8(0x4d),
            3 => w.write_u8(0x4e),
            _ => {
                w.write_u8(0x3a);
                w.write_u8(*local_variable_index);
            }
        },
        BytecodeInstruction::ILoad { .. } => todo!(),
        BytecodeInstruction::IStore { .. } => todo!(),
        BytecodeInstruction::LLoad { .. } => todo!(),
        BytecodeInstruction::LStore { .. } => todo!(),
        BytecodeInstruction::FLoad { .. } => todo!(),
        BytecodeInstruction::FStore { .. } => todo!(),
        BytecodeInstruction::DLoad {
            local_variable_index,
        } => match local_variable_index {
            0 => w.write_u8(0x26),
            1 => w.write_u8(0x27),
            2 => w.write_u8(0x28),
            3 => w.write_u8(0x29),
            _ => {
                w.write_u8(0x18);
                w.write_u8(*local_variable_index);
            }
        },
        BytecodeInstruction::DStore { .. } => todo!(),
        BytecodeInstruction::AaLoad {} => todo!(),
        BytecodeInstruction::BaLoad {} => todo!(),
        BytecodeInstruction::AaStore {} => todo!(),
        BytecodeInstruction::BaStore {} => todo!(),
        BytecodeInstruction::CaStore {} => todo!(),
        BytecodeInstruction::SaStore {} => todo!(),
        BytecodeInstruction::NewArray { .. } => todo!(),
        BytecodeInstruction::ANewArray { .. } => todo!(),
        BytecodeInstruction::AThrow {} => todo!(),
        BytecodeInstruction::New { .. } => todo!(),
        BytecodeInstruction::BiPush { .. } => todo!(),
        BytecodeInstruction::SiPush { .. } => todo!(),
        BytecodeInstruction::Pop {} => todo!(),
        BytecodeInstruction::Pop2 {} => todo!(),
        BytecodeInstruction::Return {} => w.write_u8(0xb1),
        BytecodeInstruction::IReturn {} => todo!(),
        BytecodeInstruction::LReturn {} => todo!(),
        BytecodeInstruction::FReturn {} => todo!(),
        BytecodeInstruction::DReturn {} => w.write_u8(0xaf),
        BytecodeInstruction::AReturn {} => todo!(),
        BytecodeInstruction::GetStatic { field_ref_index } => {
            w.write_u8(0xb2);
            w.write_u16(*field_ref_index);
        }
        BytecodeInstruction::PutStatic { .. } => todo!(),
        BytecodeInstruction::GetField { .. } => todo!(),
        BytecodeInstruction::PutField { .. } => todo!(),
        BytecodeInstruction::InvokeSpecial { method_ref_index } => {
            w.write_u8(0xb7);
            w.write_u16(*method_ref_index);
        }

        BytecodeInstruction::InvokeStatic { method_ref_index } => {
            w.write_u8(0xb8);
            w.write_u16(*method_ref_index);
        }
        BytecodeInstruction::InvokeVirtual { method_ref_index } => {
            w.write_u8(0xb6);
            w.write_u16(*method_ref_index);
        }
        BytecodeInstruction::InvokeDynamic { .. } => todo!(),
        BytecodeInstruction::InvokeInterface { .. } => todo!(),
        BytecodeInstruction::ArrayLength {} => todo!(),
        BytecodeInstruction::LCmp {} => todo!(),
        BytecodeInstruction::FCmpL {} => todo!(),
        BytecodeInstruction::FCmpG {} => todo!(),
        BytecodeInstruction::DCmpL {} => todo!(),
        BytecodeInstruction::DCmpG {} => todo!(),
        BytecodeInstruction::IfAcmpEq { .. } => todo!(),
        BytecodeInstruction::IfAcmpNe { .. } => todo!(),
        BytecodeInstruction::IfIcmpEq { .. } => todo!(),
        BytecodeInstruction::IfIcmpNe { .. } => todo!(),
        BytecodeInstruction::IfIcmpLt { .. } => todo!(),
        BytecodeInstruction::IfIcmpGe { .. } => todo!(),
        BytecodeInstruction::IfIcmpGt { .. } => todo!(),
        BytecodeInstruction::IfIcmpLe { .. } => todo!(),
        BytecodeInstruction::IfEq { .. } => todo!(),
        BytecodeInstruction::IfNe { .. } => todo!(),
        BytecodeInstruction::IfLt { .. } => todo!(),
        BytecodeInstruction::IfGe { .. } => todo!(),
        BytecodeInstruction::IfGt { .. } => todo!(),
        BytecodeInstruction::IfLe { .. } => todo!(),
        BytecodeInstruction::IfNull { .. } => todo!(),
        BytecodeInstruction::IfNonNull { .. } => todo!(),
        BytecodeInstruction::GoTo { .. } => todo!(),
        BytecodeInstruction::TableSwitch { .. } => todo!(),
        BytecodeInstruction::LookupSwitch { .. } => todo!(),
        BytecodeInstruction::CheckCast { .. } => todo!(),
        BytecodeInstruction::Instanceof { .. } => todo!(),
        BytecodeInstruction::IInc { .. } => todo!(),
        BytecodeInstruction::I2L {} => todo!(),
        BytecodeInstruction::I2F {} => todo!(),
        BytecodeInstruction::I2D {} => todo!(),
        BytecodeInstruction::L2I {} => todo!(),
        BytecodeInstruction::L2F {} => todo!(),
        BytecodeInstruction::L2D {} => todo!(),
        BytecodeInstruction::F2I {} => todo!(),
        BytecodeInstruction::F2L {} => todo!(),
        BytecodeInstruction::F2D {} => todo!(),
        BytecodeInstruction::D2I {} => todo!(),
        BytecodeInstruction::D2L {} => todo!(),
        BytecodeInstruction::D2F {} => todo!(),
        BytecodeInstruction::I2B {} => todo!(),
        BytecodeInstruction::I2C {} => todo!(),
        BytecodeInstruction::I2S {} => todo!(),
        BytecodeInstruction::IAdd {} => todo!(),
        BytecodeInstruction::ISub {} => todo!(),
        BytecodeInstruction::IMul {} => todo!(),
        BytecodeInstruction::IDiv {} => todo!(),
        BytecodeInstruction::IRem {} => todo!(),
        BytecodeInstruction::IAnd {} => todo!(),
        BytecodeInstruction::IShl {} => todo!(),
        BytecodeInstruction::IShr {} => todo!(),
        BytecodeInstruction::IUshr {} => todo!(),
        BytecodeInstruction::IOr {} => todo!(),
        BytecodeInstruction::IXor {} => todo!(),
        BytecodeInstruction::INeg {} => todo!(),
        BytecodeInstruction::LAdd {} => todo!(),
        BytecodeInstruction::LSub {} => todo!(),
        BytecodeInstruction::LMul {} => todo!(),
        BytecodeInstruction::LDiv {} => todo!(),
        BytecodeInstruction::LRem {} => todo!(),
        BytecodeInstruction::LAnd {} => todo!(),
        BytecodeInstruction::LOr {} => todo!(),
        BytecodeInstruction::LXor {} => todo!(),
        BytecodeInstruction::LShl {} => todo!(),
        BytecodeInstruction::LShr {} => todo!(),
        BytecodeInstruction::LUshr {} => todo!(),
        BytecodeInstruction::LNeg {} => todo!(),
        BytecodeInstruction::FAdd {} => todo!(),
        BytecodeInstruction::FMul {} => todo!(),
        BytecodeInstruction::FNeg {} => todo!(),
        BytecodeInstruction::FDiv {} => todo!(),
        BytecodeInstruction::FRem {} => todo!(),
        BytecodeInstruction::FSub {} => todo!(),
        BytecodeInstruction::DAdd {} => todo!(),
        BytecodeInstruction::DMul {} => todo!(),
        BytecodeInstruction::DNeg {} => todo!(),
        BytecodeInstruction::DDiv {} => todo!(),
        BytecodeInstruction::DRem {} => todo!(),
        BytecodeInstruction::DSub {} => todo!(),
    }
}

/**
 * Returns the number of bytes required to fully encode (opcode included, padding excluded) the given instruction.
 */
pub fn get_instruction_length(instruction: &BytecodeInstruction) -> u32 {
    match instruction {
        BytecodeInstruction::Dup {} => 1,
        BytecodeInstruction::AConstNull {} => 1,
        BytecodeInstruction::IConst { .. } => 5,
        BytecodeInstruction::LConst { .. } => 9,
        BytecodeInstruction::FConst { .. } => 5,
        BytecodeInstruction::DConst { .. } => 9,
        BytecodeInstruction::Ldc { .. } => 2,
        BytecodeInstruction::LdcW { .. } => 3,
        BytecodeInstruction::Ldc2W { .. } => 3,
        BytecodeInstruction::ALoad {
            local_variable_index,
        } => match local_variable_index {
            0..=3 => 1,
            _ => 2,
        },
        BytecodeInstruction::AStore {
            local_variable_index,
        } => match local_variable_index {
            0..=3 => 1,
            _ => 2,
        },
        BytecodeInstruction::ILoad { .. } => todo!(),
        BytecodeInstruction::IStore { .. } => todo!(),
        BytecodeInstruction::LLoad { .. } => todo!(),
        BytecodeInstruction::LStore { .. } => todo!(),
        BytecodeInstruction::FLoad { .. } => todo!(),
        BytecodeInstruction::FStore { .. } => todo!(),
        BytecodeInstruction::DLoad {
            local_variable_index,
        } => match local_variable_index {
            0..=3 => 1,
            _ => 2,
        },
        BytecodeInstruction::DStore { .. } => todo!(),
        BytecodeInstruction::AaLoad {} => 1,
        BytecodeInstruction::BaLoad {} => 1,
        BytecodeInstruction::AaStore {} => 1,
        BytecodeInstruction::BaStore {} => 1,
        BytecodeInstruction::CaStore {} => 1,
        BytecodeInstruction::SaStore {} => 1,
        BytecodeInstruction::NewArray { .. } => todo!(),
        BytecodeInstruction::ANewArray { .. } => todo!(),
        BytecodeInstruction::AThrow {} => 1,
        BytecodeInstruction::New { .. } => 3,
        BytecodeInstruction::BiPush { .. } => 2,
        BytecodeInstruction::SiPush { .. } => 3,
        BytecodeInstruction::Pop {} => 1,
        BytecodeInstruction::Pop2 {} => 1,
        BytecodeInstruction::Return {} => 1,
        BytecodeInstruction::IReturn {} => 1,
        BytecodeInstruction::LReturn {} => 1,
        BytecodeInstruction::FReturn {} => 1,
        BytecodeInstruction::DReturn {} => 1,
        BytecodeInstruction::AReturn {} => 1,
        BytecodeInstruction::GetStatic { .. } => 3,
        BytecodeInstruction::PutStatic { .. } => 3,
        BytecodeInstruction::GetField { .. } => 3,
        BytecodeInstruction::PutField { .. } => 3,
        BytecodeInstruction::InvokeSpecial { .. } => 3,
        BytecodeInstruction::InvokeStatic { .. } => 3,
        BytecodeInstruction::InvokeVirtual { .. } => 3,
        BytecodeInstruction::InvokeDynamic { .. } => todo!(),
        BytecodeInstruction::InvokeInterface { .. } => todo!(),
        BytecodeInstruction::ArrayLength {} => todo!(),
        BytecodeInstruction::LCmp {} => todo!(),
        BytecodeInstruction::FCmpL {} => todo!(),
        BytecodeInstruction::FCmpG {} => todo!(),
        BytecodeInstruction::DCmpL {} => todo!(),
        BytecodeInstruction::DCmpG {} => todo!(),
        BytecodeInstruction::IfAcmpEq { .. } => todo!(),
        BytecodeInstruction::IfAcmpNe { .. } => todo!(),
        BytecodeInstruction::IfIcmpEq { .. } => todo!(),
        BytecodeInstruction::IfIcmpNe { .. } => todo!(),
        BytecodeInstruction::IfIcmpLt { .. } => todo!(),
        BytecodeInstruction::IfIcmpGe { .. } => todo!(),
        BytecodeInstruction::IfIcmpGt { .. } => todo!(),
        BytecodeInstruction::IfIcmpLe { .. } => todo!(),
        BytecodeInstruction::IfEq { .. } => todo!(),
        BytecodeInstruction::IfNe { .. } => todo!(),
        BytecodeInstruction::IfLt { .. } => todo!(),
        BytecodeInstruction::IfGe { .. } => todo!(),
        BytecodeInstruction::IfGt { .. } => todo!(),
        BytecodeInstruction::IfLe { .. } => todo!(),
        BytecodeInstruction::IfNull { .. } => todo!(),
        BytecodeInstruction::IfNonNull { .. } => todo!(),
        BytecodeInstruction::GoTo { .. } => todo!(),
        BytecodeInstruction::TableSwitch { .. } => todo!(),
        BytecodeInstruction::LookupSwitch { .. } => todo!(),
        BytecodeInstruction::CheckCast { .. } => todo!(),
        BytecodeInstruction::Instanceof { .. } => todo!(),
        BytecodeInstruction::IInc { .. } => todo!(),
        BytecodeInstruction::I2L {} => 1,
        BytecodeInstruction::I2F {} => 1,
        BytecodeInstruction::I2D {} => 1,
        BytecodeInstruction::L2I {} => 1,
        BytecodeInstruction::L2F {} => 1,
        BytecodeInstruction::L2D {} => 1,
        BytecodeInstruction::F2I {} => 1,
        BytecodeInstruction::F2L {} => 1,
        BytecodeInstruction::F2D {} => 1,
        BytecodeInstruction::D2I {} => 1,
        BytecodeInstruction::D2L {} => 1,
        BytecodeInstruction::D2F {} => 1,
        BytecodeInstruction::I2B {} => 1,
        BytecodeInstruction::I2C {} => 1,
        BytecodeInstruction::I2S {} => 1,
        BytecodeInstruction::IAdd {} => 1,
        BytecodeInstruction::ISub {} => 1,
        BytecodeInstruction::IMul {} => 1,
        BytecodeInstruction::IDiv {} => 1,
        BytecodeInstruction::IRem {} => 1,
        BytecodeInstruction::IAnd {} => 1,
        BytecodeInstruction::IShl {} => 1,
        BytecodeInstruction::IShr {} => 1,
        BytecodeInstruction::IUshr {} => 1,
        BytecodeInstruction::IOr {} => 1,
        BytecodeInstruction::IXor {} => 1,
        BytecodeInstruction::INeg {} => 1,
        BytecodeInstruction::LAdd {} => 1,
        BytecodeInstruction::LSub {} => 1,
        BytecodeInstruction::LMul {} => 1,
        BytecodeInstruction::LDiv {} => 1,
        BytecodeInstruction::LRem {} => 1,
        BytecodeInstruction::LAnd {} => 1,
        BytecodeInstruction::LOr {} => 1,
        BytecodeInstruction::LXor {} => 1,
        BytecodeInstruction::LShl {} => 1,
        BytecodeInstruction::LShr {} => 1,
        BytecodeInstruction::LUshr {} => 1,
        BytecodeInstruction::LNeg {} => 1,
        BytecodeInstruction::FAdd {} => 1,
        BytecodeInstruction::FMul {} => 1,
        BytecodeInstruction::FNeg {} => 1,
        BytecodeInstruction::FDiv {} => 1,
        BytecodeInstruction::FRem {} => 1,
        BytecodeInstruction::FSub {} => 1,
        BytecodeInstruction::DAdd {} => 1,
        BytecodeInstruction::DMul {} => 1,
        BytecodeInstruction::DNeg {} => 1,
        BytecodeInstruction::DDiv {} => 1,
        BytecodeInstruction::DRem {} => 1,
        BytecodeInstruction::DSub {} => 1,
    }
}
