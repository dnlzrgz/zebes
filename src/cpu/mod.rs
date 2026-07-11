mod addressing;
mod flags;
mod instructions;
mod opcodes;

use crate::{
    bus::Bus,
    cpu::{flags::*, instructions::Instruction, opcodes::opcode_table},
};

/// Models the core of the MOS 6502.
pub struct Cpu {
    /// Accumulator register.
    a: u8,
    /// X register.
    x: u8,
    /// Y register.
    y: u8,
    /// Program counter.
    pc: u16,
    /// Stack pointer (points to location on bus).
    sp: u8,
    /// Status register.
    status: u8,
    /// Number of clock cycles remaining for the current instruction
    /// that is being executed.
    cycles: u8,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0xFD,
            status: RESET_STATUS,
            cycles: 0,
        }
    }
}

impl Cpu {
    pub fn new() -> Self {
        Self::default()
    }

    /// Forces the CPU into the state it would have after a reset (e.g. when
    /// the physical reset button of the NES was pressed).
    ///
    /// The program counter (`pc`) is loaded from the "reset vector". This lets a
    /// cartridge tell the CPU where to start executing, since different programs are
    /// expected to begin at different locations.
    pub fn reset(&mut self, bus: &Bus) {
        self.pc = u16::from_le_bytes([bus.peek(0xFFFC), bus.peek(0xFFFD)]);
        self.sp = self.sp.wrapping_sub(3);
        self.status = RESET_STATUS;
        self.cycles = 8;
    }

    pub fn clock(&mut self, bus: &mut Bus) {
        if self.cycles == 0 {
            let opcode = bus.read(self.pc);
            self.pc += 1;
            self.cycles = self.execute(opcode, bus);
        }

        self.cycles -= 1;
    }

    fn push_byte(&mut self, bus: &mut Bus, value: u8) {
        bus.write(0x0100 + self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn execute(&mut self, opcode: u8, bus: &mut Bus) -> u8 {
        let info = opcode_table()[opcode as usize];
        let operand = self.resolve_address(info.mode, bus);

        // TODO: check function pointers.
        let extra = match info.instruction {
            Instruction::ADC => self.adc(operand, bus),
            Instruction::AND => self.and(operand, bus),
            Instruction::ASL => self.asl(operand, bus),
            Instruction::BCC => self.bcc(operand, bus),
            Instruction::BCS => self.bcs(operand, bus),
            Instruction::BEQ => self.beq(operand, bus),
            Instruction::BIT => self.bit(operand, bus),
            Instruction::BMI => self.bmi(operand, bus),
            Instruction::BNE => self.bne(operand, bus),
            Instruction::BPL => self.bpl(operand, bus),
            Instruction::BRK => self.brk(operand, bus),
            Instruction::BVC => self.bvc(operand, bus),
            Instruction::BVS => self.bvs(operand, bus),
            Instruction::CLC => self.clc(operand, bus),
            Instruction::CLD => self.cld(operand, bus),
            Instruction::CLI => self.cli(operand, bus),
            Instruction::CLV => self.clv(operand, bus),
            Instruction::CMP => self.cmp(operand, bus),
            Instruction::CPX => self.cpx(operand, bus),
            Instruction::CPY => self.cpy(operand, bus),
            Instruction::DEC => self.dec(operand, bus),
            Instruction::DEX => self.dex(operand, bus),
            Instruction::DEY => self.dey(operand, bus),
            Instruction::EOR => self.eor(operand, bus),
            Instruction::INC => self.inc(operand, bus),
            Instruction::INX => self.inx(operand, bus),
            Instruction::INY => self.iny(operand, bus),
            Instruction::JMP => self.jmp(operand, bus),
            Instruction::JSR => self.jsr(operand, bus),
            Instruction::LDA => self.lda(operand, bus),
            Instruction::LDX => self.ldx(operand, bus),
            Instruction::LDY => self.ldy(operand, bus),
            Instruction::LSR => self.lsr(operand, bus),
            _ => todo!("instruction not yet implemented: {:?}", info.instruction),
        };

        info.cycles + extra
    }
}
