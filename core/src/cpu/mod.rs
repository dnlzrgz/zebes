pub mod addressing;
pub mod flags;
pub mod instructions;
pub mod opcodes;

use crate::{
    cpu::{addressing::Operand, flags::*, instructions::Instruction, opcodes::opcode_table},
    cpu_bus::CpuBus,
};

/// Models the MOS 6502.
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
    total_cycles: u64,
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
            total_cycles: 0,
        }
    }
}

impl Cpu {
    pub fn new() -> Self {
        Self::default()
    }

    /// The program counter is loaded from the "reset vector". This lets a
    /// cartridge tell the CPU where to start executing, since different programs are
    /// Forces the CPU into the state it would have after a reset. The program counter is loaded
    /// from the "reset vector". This lets a cartridge tell the CPU where to start executing, since
    /// different programs expect to begin at different locations.
    pub fn reset(&mut self, bus: &CpuBus) {
        self.pc = u16::from_le_bytes([bus.peek(0xFFFC), bus.peek(0xFFFD)]);
        self.sp = 0xFD;
        self.status = RESET_STATUS;
        self.cycles = 7;
    }

    pub fn clock(&mut self, bus: &mut CpuBus) {
        if self.cycles == 0 {
            let opcode = bus.read(self.pc);
            self.pc = self.pc.wrapping_add(1);
            self.cycles = self.execute(opcode, bus);
        }

        self.cycles = self.cycles.wrapping_sub(1);
        self.total_cycles = self.total_cycles.wrapping_add(1);
    }

    /// Pushes a byte into the stack and then decrementing the stack pointer.
    fn push_byte(&mut self, bus: &mut CpuBus, value: u8) {
        bus.write(0x0100 + self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    /// Pulls a byte off the stack. The stack pointer is increment first, then the byte is read.
    fn pull_byte(&mut self, bus: &mut CpuBus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        bus.read(0x0100 + self.sp as u16)
    }

    /// Abstraction of all 8 conditional branch instructions. Each of them just calls this method
    /// with their own flag checks as `condition`.
    /// If the condition is false the branch is not taken. If its true, then the branch is taken the
    /// cost in cycles is incremented by 1, or 2 if the jump crosses a page boundary.
    fn branch_if(&mut self, condition: bool, operand: Operand) -> u8 {
        if !condition {
            return 0;
        }

        let (address, _) = operand.expect_address();
        let old_pc = self.pc;
        self.pc = address;
        if (old_pc & 0xFF00) != (address & 0xFF00) {
            2
        } else {
            1
        }
    }

    fn execute(&mut self, opcode: u8, bus: &mut CpuBus) -> u8 {
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
            Instruction::NOP => self.nop(operand, bus),
            Instruction::ORA => self.ora(operand, bus),
            Instruction::PHA => self.pha(operand, bus),
            Instruction::PHP => self.php(operand, bus),
            Instruction::PLA => self.pla(operand, bus),
            Instruction::PLP => self.plp(operand, bus),
            Instruction::ROL => self.rol(operand, bus),
            Instruction::ROR => self.ror(operand, bus),
            Instruction::RTI => self.rti(operand, bus),
            Instruction::RTS => self.rts(operand, bus),
            Instruction::SBC => self.sbc(operand, bus),
            Instruction::SEC => self.sec(operand, bus),
            Instruction::SED => self.sed(operand, bus),
            Instruction::SEI => self.sei(operand, bus),
            Instruction::STA => self.sta(operand, bus),
            Instruction::STX => self.stx(operand, bus),
            Instruction::STY => self.sty(operand, bus),
            Instruction::TAX => self.tax(operand, bus),
            Instruction::TAY => self.tay(operand, bus),
            Instruction::TXA => self.txa(operand, bus),
            Instruction::TYA => self.tya(operand, bus),
            Instruction::TSX => self.tsx(operand, bus),
            Instruction::TXS => self.txs(operand, bus),
            _ => todo!("not implemented: {:?}", info.instruction),
        };

        info.cycles + extra
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.pc = pc;
    }

    pub fn sp(&self) -> u8 {
        self.sp
    }

    pub fn a(&self) -> u8 {
        self.a
    }

    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn y(&self) -> u8 {
        self.y
    }

    pub fn status(&self) -> u8 {
        self.status
    }

    pub fn cycles(&self) -> u8 {
        self.cycles
    }

    pub fn total_cycles(&self) -> u64 {
        self.total_cycles
    }
}
