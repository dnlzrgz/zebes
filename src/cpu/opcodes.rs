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

    // DEX — Decrement X
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

    // ORA — Bitwise OR
    table[0x09] = op!(ORA, Immediate, 2);
    table[0x05] = op!(ORA, ZeroPage, 3);
    table[0x15] = op!(ORA, ZeroPageX, 4);
    table[0x0D] = op!(ORA, Absolute, 4);
    table[0x1D] = op!(ORA, AbsoluteX, 4); // +1 if page crossed
    table[0x19] = op!(ORA, AbsoluteY, 4); // +1 if page crossed
    table[0x01] = op!(ORA, IndirectX, 6);
    table[0x11] = op!(ORA, IndirectY, 5); // +1 if page crossed

    // PHA — Push Accumulator
    table[0x48] = op!(PHA, Implied, 3);

    // PHP — Push Processor Status
    table[0x08] = op!(PHP, Implied, 3);

    // PLA — Pull A
    table[0x68] = op!(PLA, Implied, 4);

    // PLP — Pull Processor Status
    table[0x28] = op!(PLP, Implied, 4);

    // ROL — Rotate Left
    table[0x2A] = op!(ROL, Accumulator, 2);
    table[0x26] = op!(ROL, ZeroPage, 5);
    table[0x36] = op!(ROL, ZeroPageX, 6);
    table[0x2E] = op!(ROL, Absolute, 6);
    table[0x3E] = op!(ROL, AbsoluteX, 7); // RMW: always max cycles, no page-crossing check

    // ROR — Rotate Right
    table[0x6A] = op!(ROR, Accumulator, 2);
    table[0x66] = op!(ROR, ZeroPage, 5);
    table[0x76] = op!(ROR, ZeroPageX, 6);
    table[0x6E] = op!(ROR, Absolute, 6);
    table[0x7E] = op!(ROR, AbsoluteX, 7); // RMW: always max cycles, no page-crossing check

    // RTI — Return from Interrupt
    table[0x40] = op!(RTI, Implied, 6);

    // RTS — Return from Subroutine
    table[0x60] = op!(RTS, Implied, 6);

    // SBC - Subtract with Carry
    table[0xE9] = op!(SBC, Immediate, 2);
    table[0xE5] = op!(SBC, ZeroPage, 3);
    table[0xF5] = op!(SBC, ZeroPageX, 4);
    table[0xED] = op!(SBC, Absolute, 4);
    table[0xFD] = op!(SBC, AbsoluteX, 4); // +1 if page crossed
    table[0xF9] = op!(SBC, AbsoluteY, 4); // +1 if page crossed
    table[0xE1] = op!(SBC, IndirectX, 6);
    table[0xF1] = op!(SBC, IndirectY, 5); // +1 if page crossed

    // SEC — Set Carry Flag
    table[0x38] = op!(SEC, Implied, 2);

    // SED — Set Decimal Flag
    table[0xF8] = op!(SED, Implied, 2);

    // SEI — Set Interrupt Disable Flag
    table[0x78] = op!(SEI, Implied, 2);

    // STA — Store Accumulator
    table[0x85] = op!(STA, ZeroPage, 3);
    table[0x95] = op!(STA, ZeroPageX, 4);
    table[0x8D] = op!(STA, Absolute, 4);
    table[0x9D] = op!(STA, AbsoluteX, 5);
    table[0x99] = op!(STA, AbsoluteY, 5);
    table[0x81] = op!(STA, IndirectX, 6);
    table[0x91] = op!(STA, IndirectY, 6);

    // STX — Store X Register
    table[0x86] = op!(STX, ZeroPage, 3);
    table[0x96] = op!(STX, ZeroPageY, 4);
    table[0x8E] = op!(STX, Absolute, 4);

    // STY — Store Y Register
    table[0x84] = op!(STY, ZeroPage, 3);
    table[0x94] = op!(STY, ZeroPageX, 4);
    table[0x8C] = op!(STY, Absolute, 4);

    table
}

pub fn opcode_table() -> &'static [Opcode; 256] {
    static TABLE: OnceLock<[Opcode; 256]> = OnceLock::new();
    TABLE.get_or_init(build_opcode_table)
}
