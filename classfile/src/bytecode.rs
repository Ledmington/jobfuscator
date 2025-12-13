use std::io::Result;

use binary_reader::BinaryReader;

/**
 * Reference available at <https://docs.oracle.com/javase/specs/jvms/se25/html/jvms-6.html#jvms-6.5>
 */
pub enum BytecodeInstruction {
    AConstNull {},
    IConst { constant: i32 },
    Ldc { constant_pool_index: u8 },
    ALoad { local_variable_index: u8 },
    AStore { local_variable_index: u8 },
    ILoad { local_variable_index: u8 },
    IStore { local_variable_index: u8 },
    AaLoad {},
    Return {},
    GetStatic { field_ref_index: u16 },
    InvokeSpecial { method_ref_index: u16 },
    InvokeStatic { method_ref_index: u16 },
    InvokeVirtual { method_ref_index: u16 },
    ArrayLength {},
    IfIcmpEq { offset: u16 },
    IfIcmpNe { offset: u16 },
    IfIcmpLt { offset: u16 },
    IfIcmpGe { offset: u16 },
    IfIcmpGt { offset: u16 },
    IfIcmpLe { offset: u16 },
}

pub fn parse_bytecode(reader: &mut BinaryReader) -> Vec<BytecodeInstruction> {
    let mut instructions: Vec<BytecodeInstruction> = Vec::new();
    // FIXME: not an infinite loop
    loop {
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
            0x12 => BytecodeInstruction::Ldc {
                constant_pool_index: reader.read_u8().unwrap(),
            },
            0x15 => BytecodeInstruction::ILoad {
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
            0x9f => BytecodeInstruction::IfIcmpEq {
                offset: reader.read_u16().unwrap(),
            },
            0xa0 => BytecodeInstruction::IfIcmpNe {
                offset: reader.read_u16().unwrap(),
            },
            0xa1 => BytecodeInstruction::IfIcmpLt {
                offset: reader.read_u16().unwrap(),
            },
            0xa2 => BytecodeInstruction::IfIcmpGe {
                offset: reader.read_u16().unwrap(),
            },
            0xa3 => BytecodeInstruction::IfIcmpGt {
                offset: reader.read_u16().unwrap(),
            },
            0xa4 => BytecodeInstruction::IfIcmpLe {
                offset: reader.read_u16().unwrap(),
            },
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
            0xbe => BytecodeInstruction::ArrayLength {},
            _ => panic!("Unknown bytecode instruction 0x{:02x}", opcode),
        });
    }
    instructions
}
