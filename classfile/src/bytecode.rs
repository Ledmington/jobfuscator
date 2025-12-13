#![forbid(unsafe_code)]

use std::io::Result;

use binary_reader::BinaryReader;

/**
 * Reference available at <https://docs.oracle.com/javase/specs/jvms/se25/html/jvms-6.html#jvms-6.5>
 */
pub enum BytecodeInstruction {
    Dup {},
    AConstNull {},
    IConst {
        constant: i32,
    },
    Ldc {
        constant_pool_index: u8,
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
    AaLoad {},
    AaStore {},
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
    Return {},
    GetStatic {
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
    LDiv {},
    IInc {
        index: u8,
        constant: i8,
    },
    IAdd {},
}

pub struct LookupSwitchPair {
    match_value: i32,
    offset: i32,
}

pub fn parse_bytecode(reader: &mut BinaryReader) -> Vec<BytecodeInstruction> {
    let mut instructions: Vec<BytecodeInstruction> = Vec::new();
    while reader.position() < reader.len() {
        let tmp: Result<u8> = reader.read_u8();
        if tmp.is_err() {
            break;
        }
        let opcode: u8 = tmp.unwrap();
        instructions.push(match opcode {
            0x01 => BytecodeInstruction::AConstNull {},
            0x02 => BytecodeInstruction::IConst { constant: -1 },
            0x03 => BytecodeInstruction::IConst { constant: 0 },
            0x04 => BytecodeInstruction::IConst { constant: 1 },
            0x05 => BytecodeInstruction::IConst { constant: 2 },
            0x06 => BytecodeInstruction::IConst { constant: 3 },
            0x07 => BytecodeInstruction::IConst { constant: 4 },
            0x08 => BytecodeInstruction::IConst { constant: 5 },
            0x10 => BytecodeInstruction::BiPush {
                immediate: reader.read_u8().unwrap(),
            },
            0x12 => BytecodeInstruction::Ldc {
                constant_pool_index: reader.read_u8().unwrap(),
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
            0x19 => BytecodeInstruction::ALoad {
                local_variable_index: reader.read_u8().unwrap(),
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
            0x36 => BytecodeInstruction::IStore {
                local_variable_index: reader.read_u8().unwrap(),
            },
            0x37 => BytecodeInstruction::LStore {
                local_variable_index: reader.read_u8().unwrap(),
            },
            0x3a => BytecodeInstruction::AStore {
                local_variable_index: reader.read_u8().unwrap(),
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
            0x59 => BytecodeInstruction::Dup {},
            0x60 => BytecodeInstruction::IAdd {},
            0x6d => BytecodeInstruction::LDiv {},
            0x84 => BytecodeInstruction::IInc {
                index: reader.read_u8().unwrap(),
                constant: reader.read_i8().unwrap(),
            },
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
            0xb1 => BytecodeInstruction::Return {},
            0xb2 => BytecodeInstruction::GetStatic {
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
            0xbd => BytecodeInstruction::ANewArray {
                constant_pool_index: reader.read_u16().unwrap(),
            },
            0xbe => BytecodeInstruction::ArrayLength {},
            0xbf => BytecodeInstruction::AThrow {},
            0xc0 => BytecodeInstruction::CheckCast {
                constant_pool_index: reader.read_u16().unwrap(),
            },
            _ => panic!("Unknown bytecode instruction 0x{:02x}", opcode),
        });
    }
    instructions
}
