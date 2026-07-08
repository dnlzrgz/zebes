use crate::{
    bus::Bus,
    cpu::{Cpu, addressing::Operand, flags::*},
};

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
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
    pub fn adc(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let (address, page_crossed) = operand.expect_address();
        let value = bus.read(address);
        let carry_in = if contains(self.status, CARRY) { 1 } else { 0 };

        let sum = self.a as u16 + value as u16 + carry_in as u16;
        let result = sum as u8;

        set(&mut self.status, CARRY, sum > 0xFF);

        // Overflow happens when the two operands have the same sign, but the result's sign differs
        // from theirs. XOR-ing self.a and value gives 0 in the sign bit when they match or 1 if the
        // result's sign flipped.
        let overflow = (!(self.a ^ value) & (self.a ^ result) & 0x80) != 0;
        set(&mut self.status, OVERFLOW, overflow);

        self.update_zn(result);

        self.a = result;

        page_crossed as u8
    }

    /// Bitwise AND
    /// A = A & memory
    ///
    /// ANDs a memory value and the accumulator, bit by bit. If both input bits are 1, the resulting
    /// bit is 1. Otherwise, it is 0.
    pub fn and(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let (address, page_crossed) = operand.expect_address();
        let value = bus.read(address);
        self.a &= value;
        self.update_zn(self.a);
        page_crossed as u8
    }

    /// Arithmetic Shift Left
    /// value = value << 1
    ///
    /// ASL shifts all the bits of a memory value or the accumulator one position to the left,
    /// moving the value of each bit into the next bit. Bit 7 is shifted into the carry flag, and 0
    /// is shifted into bit 0. This is equivalent to multiplying an usigned value by 2, with carry
    /// indicating overflow.
    /// This is a read-modify instruction, meaning that its addressing modes that operate on memory
    /// first write the original value back to the memory before the modified value.
    pub fn asl(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let value = match operand {
            Operand::Accumulator => self.a,
            Operand::Address { address, .. } => bus.read(address),
        };

        let temp = (value as u16) << 1;
        let result = temp as u8;

        set(&mut self.status, CARRY, temp > 0xFF);
        self.update_zn(result);

        match operand {
            Operand::Accumulator => self.a = result,
            Operand::Address { address, .. } => bus.write(address, result),
        }

        0
    }

    /// Branch if Carry Clear
    /// PC = PC + 2 memory (signed)
    ///
    /// If the carry flag is clear, BCC branches to a nearby location by adding the relative offset
    /// to the program counter. The offset is signed and has a range of [-128, 127] relative to the
    /// first byte *after* the branch instruction.
    /// The carry flag has different meanings depending on the context. BCC can be used after a
    /// compare to branch if the register is less than the memory value, so it is sometimes called
    /// BLT for Branch if Less Than. It can also be used after SBC to branch if the unsigned value
    /// underflowed or after ADC to branch if it did not overflow.
    pub fn bcc(&mut self, operand: Operand, _: &mut Bus) -> u8 {
        self.branch_if(!contains(self.status, CARRY), operand)
    }

    /// Branch if Carry Set
    /// PC = PC + 2 + memory (signed)
    ///
    /// If the carry flag is set, BCS branches to a nearby location by adding the branch offset to
    /// the program counter. The offset is signed and has a range of [-128, 127] relative to the
    /// first byte *after* the branch instruction.
    /// The carry flag has different meanings depending on the context. BCS can be used after a
    /// compare to branch if the register is greater than or equal to the memory value, so it is
    /// sometimes called BGE for Branch if Greater Than or Equal. It can also be used after ADC to
    /// branch if the usigned value overflowed or after SBC to branch if it did not underflow.
    pub fn bcs(&mut self, operand: Operand, _: &mut Bus) -> u8 {
        self.branch_if(contains(self.status, CARRY), operand)
    }

    /// Branch if Equal
    /// PC = PC + 2 + memory (signed)
    ///
    /// If the zero flag is set, BEQ branches to a nearby location by adding the branch offset to
    /// the program counter. The offset is signed and has a range of [-128, 127] relative to the
    /// first byte *after* the branch instruction.
    /// Comparison uses this flag to indicate if the compared values are equal. All instructions
    /// that change A, X, or Y also implicitly set or clear the zero flag depending on whether the
    /// register becomes 0.
    pub fn beq(&mut self, operand: Operand, _: &mut Bus) -> u8 {
        self.branch_if(contains(self.status, ZERO), operand)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Bus;
    use crate::cpu::Cpu;
    use crate::cpu::addressing::Operand;

    fn operand_at(address: u16) -> Operand {
        Operand::Address {
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

        let extra_cycles = cpu.adc(operand_at(0x0000), &mut bus);

        // 16 + 5 = 21 (0x15)
        assert_eq!(cpu.a, 0x15);
        assert!(!contains(cpu.status, CARRY));
        assert!(!contains(cpu.status, ZERO));
        assert!(!contains(cpu.status, OVERFLOW));
        assert!(!contains(cpu.status, NEGATIVE));
        assert_eq!(extra_cycles, 0);
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

    #[test]
    fn adc_returns_one_extra_cycle_when_page_crossed() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x10;
        bus.write(0x0000, 0x05);

        let operand = Operand::Address {
            address: 0x0000,
            page_crossed: true,
        };
        let extra_cycles = cpu.adc(operand, &mut bus);

        assert_eq!(extra_cycles, 1);
    }

    #[test]
    fn and_masks_acc_with_memory() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0b1100_1100;
        bus.write(0x0000, 0b1010_1010);

        let extra_cycles = cpu.and(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.a, 0b1000_1000);
        assert!(!contains(cpu.status, ZERO));
        assert!(contains(cpu.status, NEGATIVE)); // bit 7 is set in the result
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn and_sets_zero_when_result_is_zero() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0b1111_0000;
        bus.write(0x0000, 0b0000_1111); // no overlapping bits

        cpu.and(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.a, 0);
        assert!(contains(cpu.status, ZERO));
    }

    #[test]
    fn and_returns_extra_cycle_when_page_crossed() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0xFF;
        bus.write(0x0000, 0xFF);

        let operand = Operand::Address {
            address: 0x0000,
            page_crossed: true,
        };
        let extra_cycles = cpu.and(operand, &mut bus);

        assert_eq!(extra_cycles, 1);
    }

    #[test]
    fn asl_shifts_memory_value_left() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0b0000_0001);

        cpu.asl(operand_at(0x0000), &mut bus);

        // The shifted result must be written back to the same address it was read from.
        assert_eq!(bus.peek(0x0000), 0b0000_0010);
    }

    #[test]
    fn asl_sets_carry_when_bit_7_shifts_out() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0b1000_0000); // bit 7 set

        cpu.asl(operand_at(0x0000), &mut bus);

        // Shifting 0b1000_0000 left pushes that 1 out of the byte entirely. CARRY is where that
        // lost bit should end up.
        assert_eq!(bus.peek(0x0000), 0b0000_0000);
        assert!(contains(cpu.status, CARRY));
        assert!(contains(cpu.status, ZERO));
    }

    #[test]
    fn asl_does_not_set_carry_when_bit_7_is_clear() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0b0100_0000); // bit 7 clear, bit 6 set

        cpu.asl(operand_at(0x0000), &mut bus);

        assert_eq!(bus.peek(0x0000), 0b1000_0000);
        assert!(!contains(cpu.status, CARRY));
        assert!(contains(cpu.status, NEGATIVE)); // shifted bit 6 into bit 7
    }

    #[test]
    fn asl_accumulator_mode_shifts_register_not_memory() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0b0000_0011;
        bus.write(0x0000, 0xFF); // must remain untouched

        cpu.asl(Operand::Accumulator, &mut bus);

        // No bus access should happen at all.
        assert_eq!(cpu.a, 0b0000_0110);
        assert_eq!(
            bus.peek(0x0000),
            0xFF,
            "accumulator mode must not touch the bus"
        );
    }

    #[test]
    fn asl_accumulator_mode_sets_carry_correctly() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0b1100_0000;

        cpu.asl(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.a, 0b1000_0000);
        assert!(contains(cpu.status, CARRY));
        assert!(contains(cpu.status, NEGATIVE));
    }

    #[test]
    fn bcc_does_not_branch_when_carry_is_set() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, CARRY, true);

        let operand = Operand::Address {
            address: 0x2000,
            page_crossed: false,
        };
        let extra_cycles = cpu.bcc(operand, &mut bus);

        assert_eq!(
            cpu.pc, 0x1000,
            "pc must be untouched when the branch isn't taken"
        );
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn bcc_branches_when_carry_is_clear_same_page() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, CARRY, false);

        let operand = Operand::Address {
            address: 0x1010,
            page_crossed: false,
        };
        let extra_cycles = cpu.bcc(operand, &mut bus);

        assert_eq!(cpu.pc, 0x1010);
        assert_eq!(
            extra_cycles, 1,
            "branch taken, same page, costs 1 extra cycle"
        );
    }

    #[test]
    fn bcc_branch_taken_adds_two_cycles_when_page_crossed() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, CARRY, false);

        let operand = Operand::Address {
            address: 0x1105,
            page_crossed: false,
        };
        let extra_cycles = cpu.bcc(operand, &mut bus);

        assert_eq!(cpu.pc, 0x1105);
        assert_eq!(
            extra_cycles, 2,
            "branch taken, crosses a page, costs 2 extra cycles"
        );
    }

    #[test]
    fn bcs_does_not_branch_when_carry_clear() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, CARRY, false);

        let operand = Operand::Address {
            address: 0x2000,
            page_crossed: false,
        };
        let extra_cycles = cpu.bcs(operand, &mut bus);

        assert_eq!(
            cpu.pc, 0x1000,
            "pc must be untouched when the branch isn't taken"
        );
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn bcs_branches_when_carry_set_same_page() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, CARRY, true);

        let operand = Operand::Address {
            address: 0x1010,
            page_crossed: false,
        };
        let extra_cycles = cpu.bcs(operand, &mut bus);

        assert_eq!(cpu.pc, 0x1010);
        assert_eq!(
            extra_cycles, 1,
            "branch taken, same page, costs 1 extra cycle"
        );
    }

    #[test]
    fn bcs_branches_across_page_boundary() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, CARRY, true);

        let operand = Operand::Address {
            address: 0x1105,
            page_crossed: false,
        };
        let extra_cycles = cpu.bcs(operand, &mut bus);

        assert_eq!(cpu.pc, 0x1105);
        assert_eq!(
            extra_cycles, 2,
            "branch taken, crosses a page, costs 2 extra cycles"
        );
    }

    #[test]
    fn beq_does_not_branch_when_zero_clear() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, ZERO, false);

        let operand = Operand::Address {
            address: 0x2000,
            page_crossed: false,
        };
        let extra_cycles = cpu.bcs(operand, &mut bus);

        assert_eq!(
            cpu.pc, 0x1000,
            "pc must be untouched when the branch isn't taken"
        );
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn beq_branches_when_zero_set_same_page() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, CARRY, true);

        let operand = Operand::Address {
            address: 0x1010,
            page_crossed: false,
        };
        let extra_cycles = cpu.bcs(operand, &mut bus);

        assert_eq!(cpu.pc, 0x1010);
        assert_eq!(
            extra_cycles, 1,
            "branch taken, same page, costs 1 extra cycle"
        );
    }

    #[test]
    fn beq_branches_across_page_boundary() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, CARRY, true);

        let operand = Operand::Address {
            address: 0x1105,
            page_crossed: false,
        };
        let extra_cycles = cpu.bcs(operand, &mut bus);

        assert_eq!(cpu.pc, 0x1105);
        assert_eq!(
            extra_cycles, 2,
            "branch taken, crosses a page, costs 2 extra cycles"
        );
    }
}
