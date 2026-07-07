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

fn build_opcode_table() -> [Opcode; 256] {
    let mut table = [Opcode::default(); 256];

    // ADC — Add with Carry
    table[0x69] = Opcode {
        instruction: Instruction::ADC,
        mode: AddressingMode::Immediate,
        cycles: 2,
    };
    table[0x65] = Opcode {
        instruction: Instruction::ADC,
        mode: AddressingMode::ZeroPage,
        cycles: 3,
    };
    table[0x75] = Opcode {
        instruction: Instruction::ADC,
        mode: AddressingMode::ZeroPageX,
        cycles: 4,
    };
    table[0x6D] = Opcode {
        instruction: Instruction::ADC,
        mode: AddressingMode::Absolute,
        cycles: 4,
    };
    table[0x7D] = Opcode {
        instruction: Instruction::ADC,
        mode: AddressingMode::AbsoluteX,
        cycles: 4,
    }; // +1 if page crossed
    table[0x79] = Opcode {
        instruction: Instruction::ADC,
        mode: AddressingMode::AbsoluteY,
        cycles: 4,
    }; // +1 if page crossed
    table[0x61] = Opcode {
        instruction: Instruction::ADC,
        mode: AddressingMode::IndirectX,
        cycles: 6,
    };
    table[0x71] = Opcode {
        instruction: Instruction::ADC,
        mode: AddressingMode::IndirectY,
        cycles: 5,
    }; // +1 if page crossed

    table
}

pub fn opcode_table() -> &'static [Opcode; 256] {
    static TABLE: OnceLock<[Opcode; 256]> = OnceLock::new();
    TABLE.get_or_init(build_opcode_table)
}
