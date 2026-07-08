use std::sync::OnceLock;

use crate::cpu::{addressing::AddressingMode, instructions::Instruction};

#[derive(Copy, Clone, Debug)]
pub struct Opcode {
    pub instruction: Instruction,
    pub mode: AddressingMode,
    pub cycles: u8,
}

impl Default for Opcode {
    fn default() -> Self {
        Self {
            instruction: Instruction::HLT,
            mode: AddressingMode::Implied,
            cycles: 2,
        }
    }
}

macro_rules! op {
    ($instr:ident, $mode:ident, $cycles:expr) => {
        Opcode {
            instruction: Instruction::$instr,
            mode: AddressingMode::$mode,
            cycles: $cycles,
        }
    };
}

fn build_opcode_table() -> [Opcode; 256] {
    let mut table = [Opcode::default(); 256];

    // ADC — Add with Carry
    table[0x69] = op!(ADC, Immediate, 2);
    table[0x65] = op!(ADC, ZeroPage, 3);
    table[0x75] = op!(ADC, ZeroPageX, 4);
    table[0x6D] = op!(ADC, Absolute, 4);
    table[0x7D] = op!(ADC, AbsoluteX, 4); // +1 if page crossed
    table[0x79] = op!(ADC, AbsoluteY, 4); // +1 if page crossed
    table[0x61] = op!(ADC, IndirectX, 6);
    table[0x71] = op!(ADC, IndirectY, 5); // +1 if page crossed

    // AND — Logical AND
    table[0x29] = op!(AND, Immediate, 2);
    table[0x25] = op!(AND, ZeroPage, 3);
    table[0x35] = op!(AND, ZeroPageX, 4);
    table[0x2D] = op!(AND, Absolute, 4);
    table[0x3D] = op!(AND, AbsoluteX, 4); // +1 if page crossed
    table[0x39] = op!(AND, AbsoluteY, 4); // +1 if page crossed
    table[0x21] = op!(AND, IndirectX, 6);
    table[0x31] = op!(AND, IndirectY, 5); // +1 if page crossed

    // ASL — Arithmetic Shift Left
    table[0x0A] = op!(ASL, Accumulator, 2);
    table[0x06] = op!(ASL, ZeroPage, 5);
    table[0x16] = op!(ASL, ZeroPageX, 6);
    table[0x0E] = op!(ASL, Absolute, 6);
    table[0x1E] = op!(ASL, AbsoluteX, 7); // RMW instructions always take the max cycles, no page-crossing check
    //
    // BCC — Branch if Carry Clear
    table[0x90] = op!(BCC, Relative, 2);

    // BCS — Branch if Carry Set
    table[0xB0] = op!(BCS, Relative, 2);

    // BEQ — Branch if Equal
    table[0xF0] = op!(BEQ, Relative, 2);

    table
}

pub fn opcode_table() -> &'static [Opcode; 256] {
    static TABLE: OnceLock<[Opcode; 256]> = OnceLock::new();
    TABLE.get_or_init(build_opcode_table)
}
