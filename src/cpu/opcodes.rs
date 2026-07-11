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

    // BIT — Bit Test
    table[0x24] = op!(BIT, ZeroPage, 3);
    table[0x2C] = op!(BIT, Absolute, 4);

    // BMI - Branch if Minus
    table[0x30] = op!(BMI, Relative, 2);

    // BNE — Branch if Not Equal
    table[0xD0] = op!(BNE, Relative, 2);

    // BPL — Branch if Plus
    table[0x10] = op!(BPL, Relative, 2);

    // BRK — Force Interrupt
    table[0x00] = op!(BRK, Implied, 7);

    // BVC — Branch if Overflow Clear
    table[0x50] = op!(BVC, Relative, 2);

    // BVS — Branch if Overflow Set
    table[0x70] = op!(BVS, Relative, 2);

    // CLC — Clear Carry Flag
    table[0x18] = op!(CLC, Implied, 2);

    // CLD — Clear Decimal Mode
    table[0xD8] = op!(CLD, Implied, 2);

    // CLI — Clear Interrupt Disable
    table[0x58] = op!(CLI, Implied, 2);

    // CLV — Clear Overflow Flag
    table[0xB8] = op!(CLV, Implied, 2);

    // CMP — Compare Accumulator
    table[0xC9] = op!(CMP, Immediate, 2);
    table[0xC5] = op!(CMP, ZeroPage, 3);
    table[0xD5] = op!(CMP, ZeroPageX, 4);
    table[0xCD] = op!(CMP, Absolute, 4);
    table[0xDD] = op!(CMP, AbsoluteX, 4); // +1 if page crossed
    table[0xD9] = op!(CMP, AbsoluteY, 4); // +1 if page crossed
    table[0xC1] = op!(CMP, IndirectX, 6);
    table[0xD1] = op!(CMP, IndirectY, 5); // +1 if page crossed

    // CPX — Compare X Register
    table[0xE0] = op!(CPX, Immediate, 2);
    table[0xE4] = op!(CPX, ZeroPage, 3);
    table[0xEC] = op!(CPX, Absolute, 4);

    // CPY — Compare Y Register
    table[0xC0] = op!(CPY, Immediate, 2);
    table[0xC4] = op!(CPY, ZeroPage, 3);
    table[0xCC] = op!(CPY, Absolute, 4);

    // DEC — Decrement Memory
    table[0xC6] = op!(DEC, ZeroPage, 5);
    table[0xD6] = op!(DEC, ZeroPageX, 6);
    table[0xCE] = op!(DEC, Absolute, 6);
    table[0xDE] = op!(DEC, AbsoluteX, 7); // RMW: always max cycles, no page-crossing check

    // // DEX — Decrement X
    table[0xCA] = op!(DEX, Implied, 2);

    // DEY — Decrement Y
    table[0x88] = op!(DEY, Implied, 2);

    // EOR — Bitwise Exclusive OR
    table[0x49] = op!(EOR, Immediate, 2);
    table[0x45] = op!(EOR, ZeroPage, 3);
    table[0x55] = op!(EOR, ZeroPageX, 4);
    table[0x4D] = op!(EOR, Absolute, 4);
    table[0x5D] = op!(EOR, AbsoluteX, 4); // +1 if page crossed
    table[0x59] = op!(EOR, AbsoluteY, 4); // +1 if page crossed
    table[0x41] = op!(EOR, IndirectX, 6);
    table[0x51] = op!(EOR, IndirectY, 5); // +1 if page crossed

    // INC — Increment Memory
    table[0xE6] = op!(INC, ZeroPage, 5);
    table[0xF6] = op!(INC, ZeroPageX, 6);
    table[0xEE] = op!(INC, Absolute, 6);
    table[0xFE] = op!(INC, AbsoluteX, 7); // RMW: always max cycles

    // INX — Increment X
    table[0xE8] = op!(INX, Implied, 2);

    // INY — Increment Y
    table[0xC8] = op!(INY, Implied, 2);

    // JMP — Jump
    table[0x4C] = op!(JMP, Absolute, 3);
    table[0x6C] = op!(JMP, Indirect, 5);

    // JSR — Jump to Subroutine
    table[0x20] = op!(JSR, Absolute, 6);

    // LDA — Load A
    table[0xA9] = op!(LDA, Immediate, 2);
    table[0xA5] = op!(LDA, ZeroPage, 3);
    table[0xB5] = op!(LDA, ZeroPageX, 4);
    table[0xAD] = op!(LDA, Absolute, 4);
    table[0xBD] = op!(LDA, AbsoluteX, 4); // +1 if page crossed
    table[0xB9] = op!(LDA, AbsoluteY, 4); // +1 if page crossed
    table[0xA1] = op!(LDA, IndirectX, 6);
    table[0xB1] = op!(LDA, IndirectY, 5); // +1 if page crossed

    // LDX — Load X
    table[0xA2] = op!(LDX, Immediate, 2);
    table[0xA6] = op!(LDX, ZeroPage, 3);
    table[0xB6] = op!(LDX, ZeroPageY, 4);
    table[0xAE] = op!(LDX, Absolute, 4);
    table[0xBE] = op!(LDX, AbsoluteY, 4); // +1 if page crossed

    // LDY — Load Y
    table[0xA0] = op!(LDY, Immediate, 2);
    table[0xA4] = op!(LDY, ZeroPage, 3);
    table[0xB4] = op!(LDY, ZeroPageX, 4);
    table[0xAC] = op!(LDY, Absolute, 4);
    table[0xBC] = op!(LDY, AbsoluteX, 4); // +1 if page crossed

    // LSR — Logical Shift Right
    table[0x4A] = op!(LSR, Accumulator, 2);
    table[0x46] = op!(LSR, ZeroPage, 5);
    table[0x56] = op!(LSR, ZeroPageX, 6);
    table[0x4E] = op!(LSR, Absolute, 6);
    table[0x5E] = op!(LSR, AbsoluteX, 7); // RMW: always max cycles, no page-crossing check

    // NOP — No Operation
    table[0xEA] = op!(NOP, Implied, 2);

    table
}

pub fn opcode_table() -> &'static [Opcode; 256] {
    static TABLE: OnceLock<[Opcode; 256]> = OnceLock::new();
    TABLE.get_or_init(build_opcode_table)
}
