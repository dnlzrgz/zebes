use crate::{
    bus::Bus,
    cpu::{Cpu, addressing::Operand, flags::*},
};

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    /// Add with Carry
    /// A = A + memory + C
    ///
    /// Adds the carry flag and a memory value to the accumulator. The carry flag is then set to the
    /// carry value comint out of bit 7, allowing larger than 1 byte to be added together by
    /// carrying the 1 into the next byte's addition. This can also be thought of as unsigned
    /// overflow.
    /// It is common to clear carry with CLC before adding the first byte to ensure it is in a known
    /// state, avoiding off-by-one error. The overflow flag indicates whether signed overflow or
    /// underflow occurred.
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,

    // Unofficial opcodes
    ISB,
    DCP,
    AXS,
    LAS,
    LAX,
    AHX,
    SAX,
    XAA,
    SXA,
    RRA,
    TAS,
    SYA,
    ARR,
    SRE,
    ALR,
    RLA,
    ANC,
    SHAZ,
    ATX,
    SHAA,
    SLO,
    #[default]
    HLT,
}

impl Cpu {
    fn update_zn(&mut self, result: u8) {
        set(&mut self.status, ZERO, result == 0);
        set(&mut self.status, NEGATIVE, result & 0x80 != 0);
    }

    pub fn adc(&mut self, operand: Operand, bus: &mut Bus) {
        let value = bus.read(operand.address);
        let carry_in = if contains(self.status, CARRY) { 1 } else { 0 };

        let sum = self.a as u16 + value as u16 + carry_in as u16;
        let result = sum as u8;

        set(&mut self.status, CARRY, sum > 0xFF);

        // Overflow happens when the two operands have the same sign, but the result's sign differs
        // from theirs. XOR-ing self.a and value gives 0 in the sign bit when they match or 1 if the
        // result result's sign flipped.
        let overflow = (!(self.a ^ value) & (self.a ^ result) & 0x80) != 0;
        set(&mut self.status, OVERFLOW, overflow);

        self.update_zn(result);

        self.a = result;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Bus;
    use crate::cpu::Cpu;
    use crate::cpu::addressing::Operand;

    fn operand_at(address: u16) -> Operand {
        Operand {
            address,
            page_crossed: false,
        }
    }

    #[test]
    fn adc_simple_addition_with_no_carry() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x10;
        bus.write(0x0000, 0x05);

        cpu.adc(operand_at(0x0000), &mut bus);

        // 16 + 5 = 21 (0x15)
        assert_eq!(cpu.a, 0x15);
        assert!(!contains(cpu.status, CARRY));
        assert!(!contains(cpu.status, ZERO));
        assert!(!contains(cpu.status, OVERFLOW));
        assert!(!contains(cpu.status, NEGATIVE));
    }

    #[test]
    fn adc_sets_carry_flag_on_unsigned_overflow() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0xFF; // largest possible u8
        bus.write(0x0000, 0x01);

        cpu.adc(operand_at(0x0000), &mut bus);

        // 255 + 1 = 256, which wraps to 0
        assert_eq!(cpu.a, 0x00);
        assert!(contains(cpu.status, CARRY));
        assert!(contains(cpu.status, ZERO));
    }

    #[test]
    fn adc_includes_carry_bit_from_previous_adc() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x01;
        set(&mut cpu.status, CARRY, true); // simulate carry left
        bus.write(0x0000, 0x01);

        cpu.adc(operand_at(0x0000), &mut bus);

        // 1 + 1 + carry-in(1) = 3
        assert_eq!(cpu.a, 0x03);
    }
}
