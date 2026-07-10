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
    /// carry value coming out of bit 7, allowing larger than 1 byte to be added together by
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
    /// is shifted into bit 0. This is equivalent to multiplying an unsigned value by 2, with carry
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

    /// Bit Test
    /// A & memory
    ///
    /// BIT modifies flags, but does not change memory or registers. The zero flag is set depending
    /// on the result of the accumulator AND memory value, effectively applying a bitmask and then
    /// checking if any bits are set. Bits 7 and 6 of the memory value are loaded directly into the
    /// negative and overflow flags, allowing them to be easily checked without having to load a
    /// mask into A.
    /// Because BIT only changes CPU flags, it is sometimes used to trigger the read side effects of
    /// a hardware register without clobbering any CPU registers, or even to waste cycles as a
    /// 3-cycle NOP. As an advanced trick, it is occasionally used to hide a 1- or 2-byte instruction
    /// in its operand that is only executed if jumped to directly, allowing two code paths to be
    /// interleaved. However, because the instruction in the operand is treated as an address from
    /// which to read, this carries risk of triggering side effects if it reads a hardware register.
    /// This trick can be useful when working under tight constraints on space, time, or register
    /// usage.
    pub fn bit(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let (address, _) = operand.expect_address();
        let value = bus.read(address);

        set(&mut self.status, ZERO, (self.a & value) == 0x00);
        set(&mut self.status, NEGATIVE, value & 0x80 != 0);
        set(&mut self.status, OVERFLOW, value & 0x40 != 0);

        0
    }

    /// Branch if Minus
    /// PC = PC + 2 + memory (signed)
    ///
    /// If the negative flag is set, BMI branches to a nearby location by adding the branch offset
    /// to the program counter. The offset is signed and has a range of [-128, 127] relative to the
    /// first byte *after* the branch instructions.
    /// All instructions that change A, X, or Y implicitly set or clear the negative flag based on
    /// bit 7 (the sign bit).
    pub fn bmi(&mut self, operand: Operand, _: &mut Bus) -> u8 {
        self.branch_if(contains(self.status, NEGATIVE), operand)
    }

    /// Branch if Not Equal
    /// PC = PC + 2 + memory (signed)
    ///
    /// If the zero flag is clear, BNE branches to a nearby location by adding the branch offset to
    /// the program counter. The offset is signed and has a range of [-128, 127] relative to the
    /// first byte *after* the branch instruction.
    /// Comparison uses this flag to indicate if the compared values are equal. All instructions
    /// that change A, X, or Y also implicitly set or clear the zero flag depending on whether the
    /// register becomes 0.
    pub fn bne(&mut self, operand: Operand, _: &mut Bus) -> u8 {
        self.branch_if(!contains(self.status, ZERO), operand)
    }

    /// Branch if Plus
    /// PC = PC + 2 + memory (signed)
    ///
    /// If the negative flag is clear, BPL branches to a nearby location by adding the branch offset
    /// to the program counter. The offset is signed and has a range of [-128, 127] relative to the
    /// first byte *after* the branch instruction.
    /// All instructions that change A, X, or Y implicitly set or clear the negative flag based on
    /// bit 7 (the sign bit).
    pub fn bpl(&mut self, operand: Operand, _: &mut Bus) -> u8 {
        self.branch_if(!contains(self.status, NEGATIVE), operand)
    }

    /// Break (Software IRQ)
    /// `push` PC + 2 high byte to stack
    /// `push` PC + 2 low byte to stack
    /// `push` NV11DIZC flags to stack
    /// PC = ($FFFE)
    ///
    /// BRK triggers an interrupt request (IRQ). IRQs are normally triggered by external hardware,
    /// and BRK is the only way to do it in software. Like a typical IRQ, it pushes the current
    /// program counter and processor flags to the stack, sets the interrupt disable flag, and jumps
    /// to the IRQ handler. Unlike a typical IRQ, it sets the break flag in the flags byte that is
    /// pushed to the stack (like PHP) and it triggers an interrupt even if the interrupt disable
    /// flag is set. Notably, the return address that is pushed to the stack skips the byte after the
    /// BRK opcode. For this reason, BRK is often considered a 2-byte instruction with an unused
    /// immediate.
    /// Unfortunately, a 6502 bug allows the BRK IRQ to be overridden by an NMI occurring at the same
    /// time. In this case, only the NMI handler is called; the IRQ handler is skipped. However, the
    /// break flag is still set in the flags byte pushed to the stack, so the NMI hanlder can detect
    /// that this occurred (albeit slowly) by checking this flag.
    /// Because BRK uses the value $000, any byte in a programmable ROM can be overwritten with a
    /// BRK instruction to send execution to an IRQ handler. This is useful for patching one-time
    /// programmable ROMs. BRK can also be used as a system call mechanism, and the unused byte can
    /// be used by software as an argument (although it is inconvenient to access). In the context
    /// of NES games, BRK is often most useful as a crash handler, where the unused program space is
    /// filled with $00 and the IRQ handler displays debugging information or otherwise handles the
    /// crash in a clean way.
    pub fn brk(&mut self, _: Operand, bus: &mut Bus) -> u8 {
        self.pc = self.pc.wrapping_add(1);

        self.push_byte(bus, (self.pc >> 8) as u8); // high byte
        self.push_byte(bus, (self.pc & 0x00FF) as u8); // low byte

        let pushed_status = to_pushed_byte(self.status);
        self.push_byte(bus, pushed_status);

        set(&mut self.status, INTERRUPT_DISABLE, true);

        self.pc = u16::from_le_bytes([bus.read(0xFFFE), bus.read(0xFFFF)]);

        0
    }

    /// Branch if Overflow Clear
    /// PC = PC + 2 + memory (signed)
    ///
    /// If the overflowing flag is clear, BVC branches to a nearby location by adding the branch
    /// offset to the program counter. The offset is signed and has a range of [-128, 127] relative
    /// to the first byte after the branch instruction.
    /// Unlike zero, negative, and even carry, overflow is modified by very few instructions. It is
    /// most often used with the BIT instruction, particularly for polling hardware registers. It is
    /// also sometimes used for signed overflow with ADC and SBC. The standard 6502 chip allows an
    /// external device to set overflow usign a pin, enabling software to poll for that event, but
    /// this is not present on the NES' 2A03.
    pub fn bvc(&mut self, operand: Operand, _: &mut Bus) -> u8 {
        self.branch_if(!contains(self.status, OVERFLOW), operand)
    }

    /// Branch if Overflow Set
    /// PC = PC + 2 + memory (signed)
    ///
    /// If the overflow flag is set, BVS branches to a nearby location by adding the branch offset
    /// to the program counter. The offset is signed and has a range of [-128, 127] relative to the
    /// first byte after the branch instruction.
    /// Unlike zero, negative, and even carry, overflow is modified by very few instructions. It is
    /// most often used with the BIT instruction, particularly for polling hardware registers. It is
    /// also sometimes used for signed overflow with ADC and SBC. The standard 6502 chip allows an
    /// external device to set overflow usign a pin, enabling software to poll for that event, but
    /// this is not present on the NES' 2A03.
    pub fn bvs(&mut self, operand: Operand, _: &mut Bus) -> u8 {
        self.branch_if(contains(self.status, OVERFLOW), operand)
    }

    /// Clear Carry
    /// C = 0
    ///
    /// CLC clears the carry flag. In particular, this is usually done before adding the low byte of
    /// a value with ADC to avoid adding an extra 1.
    pub fn clc(&mut self, _: Operand, _: &mut Bus) -> u8 {
        set(&mut self.status, CARRY, false);
        0
    }

    /// Clear Decimal
    /// D = 0
    ///
    /// CLD clears the decimal flag. The decimal flag normally controls whether binary-coded decimal
    /// mode (BCD) is enabled, but this mode is permanently disabled on the NES' 2A03 CPU. However,
    /// the flag itself still functions and can be used to store state.
    pub fn cld(&mut self, _: Operand, _: &mut Bus) -> u8 {
        set(&mut self.status, DECIMAL, false);
        0
    }

    /// Clear Interrupt Disable
    /// I = 0
    ///
    /// CLI clears the interrupt disable flag, enabling the CPU to handle hardware IRQs. The effect
    /// of changing this flag is delayed one instruction because the flag is changed after IRQ is
    /// polled, allowing the next instruction to execute before any pending IRQ is detected and
    /// serviced. This flag has no effect on NMI, which (as the "non-maskable" name suggest) cannot
    /// be ignored by the CPU.
    pub fn cli(&mut self, _: Operand, _: &mut Bus) -> u8 {
        set(&mut self.status, INTERRUPT_DISABLE, false);
        0
    }

    /// Clear Overflow
    /// V = 0
    ///
    /// CLV clears the overflow flag. There is no corresponding SEV instruction; instead, setting
    /// overflow is exposed on the 6502 CPU as a pin controller by external hardware, and not
    /// exposed at all on the NES' 2A03 CPU.
    pub fn clv(&mut self, _: Operand, _: &mut Bus) -> u8 {
        set(&mut self.status, OVERFLOW, false);
        0
    }

    /// Compare A
    /// A - memory
    ///
    /// CMP compares A to a memory value, setting flags as appropriate but not modifying any
    /// registers. The comparison is implemented as a subtraction, setting carry if there is no
    /// borrow, zero if the result is 0, and negative if the result is negative. However, carry and
    /// zero are often most easily remembered as inequalities.
    /// Note that comparison does not affect overflow.
    pub fn cmp(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let (address, page_crossed) = operand.expect_address();
        let value = bus.read(address);

        let diff = self.a as i16 - value as i16;
        set(&mut self.status, CARRY, self.a >= value);
        self.update_zn(diff as u8);

        page_crossed as u8
    }

    /// Compare X
    /// X - memory
    ///
    /// CPX compares X to a memory value, setting flags as appropriate but not modifying any
    /// registers. The comparison is implemented as a subtraction, setting carry if there is no
    /// borrow, zero if the result is 0, and negative if the result is negative. However, carry and
    /// zero are often most easily remembered as inequalities.
    /// Note that comparison does not affect overflow.
    pub fn cpx(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let (address, page_crossed) = operand.expect_address();
        let value = bus.read(address);

        let diff = self.x as i16 - value as i16;
        set(&mut self.status, CARRY, self.x >= value);
        self.update_zn(diff as u8);

        page_crossed as u8
    }

    /// Compare Y
    /// Y - memory
    ///
    /// CPY compares Y to a memory value, setting flags as appropriate but not modifying any
    /// registers. The comparison is implemented as a subtraction, setting carry if there is no
    /// borrow, zero if the result is 0, and negative if the result is negative. However, carry and
    /// zero are often most easily remembered as inequalities.
    /// Note that comparison does not affect overflow.
    pub fn cpy(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let (address, page_crossed) = operand.expect_address();
        let value = bus.read(address);

        let diff = self.y as i16 - value as i16;
        set(&mut self.status, CARRY, self.y >= value);
        self.update_zn(diff as u8);

        page_crossed as u8
    }

    /// Decrement Memory
    /// memory = memory - 1
    ///
    /// DEC subtracts 1 from a memory location. Notably, there is no version of this instruction for
    /// the accumulator; ADC or SBC must be used, instead.
    /// This is a read-modify-write instruction, meaning that it first writes the original value
    /// back to memory before the modified value. This extra write can matter if targeting hardware
    /// register.
    /// Note that decrement does not affect carry nor overflow.
    pub fn dec(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let (address, _) = operand.expect_address();
        let value = bus.read(address);

        let result = value.wrapping_sub(1);
        bus.write(address, result);
        self.update_zn(result);

        0
    }

    /// Decrement X
    /// X = X - 1
    ///
    /// DEX subtracts 1 from the X register. Note that it does not affect carry nor overflow.
    pub fn dex(&mut self, _: Operand, _: &mut Bus) -> u8 {
        self.x = self.x.wrapping_sub(1);
        self.update_zn(self.x);

        0
    }

    /// Decrement Y
    /// Y = Y - 1
    ///
    /// DEY subtracts 1 from the Y register. Note that it does not affect carry nor overflow.
    pub fn dey(&mut self, _: Operand, _: &mut Bus) -> u8 {
        self.y = self.y.wrapping_sub(1);
        self.update_zn(self.y);

        0
    }

    /// Bitwise Exclusive OR
    /// A = A ^ memory
    ///
    /// EOR exclusive-ORs a memory value and the accumulator, bit by bit. If the input bits are
    /// different, the resulting bit is 1. If they are the same, it is 0. This operation is also
    /// known as XOR.
    /// 6502 does not have a bitwise NOT instruction, but using EOR with value $FF has the same
    /// behaviour, inverting every bit of the other value. In fact, EOR can be thought of as NOT
    /// with a bitmask; all the 1 bits in one vlaue have the effect of inverting the corresponding
    /// bit in the other value, while 0 bits do nothing.
    pub fn eor(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let (address, page_crossed) = operand.expect_address();
        let value = bus.read(address);

        self.a ^= value;
        self.update_zn(self.a);

        page_crossed as u8
    }

    /// Increment Memory
    /// memory = memory + 1
    ///
    /// INC adds 1 to a memory location. Notably, there is no version of this instruction for the
    /// accumulator; ADC or SBC must be used, instead.
    /// This is a read-modify-write instruction, meaning that it first writes the original value
    /// back to memory before the modified value. This extra write han matter if targeting a
    /// hardware register.
    /// Note that increment does not affect carry nor overflow.
    pub fn inc(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let (address, _) = operand.expect_address();
        let value = bus.read(address);

        let result = value.wrapping_add(1);
        bus.write(address, result);
        self.update_zn(result);

        0
    }

    /// Increment X
    /// X = X + 1
    ///
    /// INX adds 1 from the X register. Note that it does not affect carry nor overflow.
    pub fn inx(&mut self, _: Operand, _: &mut Bus) -> u8 {
        self.x = self.x.wrapping_add(1);
        self.update_zn(self.x);

        0
    }

    /// Increment Y
    /// Y = Y + 1
    ///
    /// INY adds 1 from the Y register. Note that it does not affect carry nor overflow.
    pub fn iny(&mut self, _: Operand, _: &mut Bus) -> u8 {
        self.y = self.y.wrapping_add(1);
        self.update_zn(self.y);

        0
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

    fn assert_branch_not_taken(branch_fn: fn(&mut Cpu, Operand, &mut Bus) -> u8, cpu: &mut Cpu) {
        let mut bus = Bus::new();
        cpu.pc = 0x1000;
        let operand = operand_at(0x2000);
        let extra_cycles = branch_fn(cpu, operand, &mut bus);
        assert_eq!(cpu.pc, 0x1000);
        assert_eq!(extra_cycles, 0);
    }

    fn assert_branch_taken(
        branch_fn: fn(&mut Cpu, Operand, &mut Bus) -> u8,
        cpu: &mut Cpu,
        target: u16,
        expected_extra_cycles: u8,
    ) {
        let mut bus = Bus::new();
        let operand = operand_at(target);
        let extra_cycles = branch_fn(cpu, operand, &mut bus);
        assert_eq!(cpu.pc, target);
        assert_eq!(extra_cycles, expected_extra_cycles);
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
        assert_eq!(bus.peek(0x0000), 0xFF);
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
        let mut cpu = Cpu::new();
        set(&mut cpu.status, CARRY, true);
        assert_branch_not_taken(Cpu::bcc, &mut cpu);
    }

    #[test]
    fn bcc_branches_when_carry_is_clear_same_page() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, CARRY, false);
        assert_branch_taken(Cpu::bcc, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn bcc_branch_taken_adds_two_cycles_when_page_crossed() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, CARRY, false);
        assert_branch_taken(Cpu::bcc, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn bcs_does_not_branch_when_carry_clear() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, CARRY, false);
        assert_branch_not_taken(Cpu::bcs, &mut cpu);
    }

    #[test]
    fn bcs_branches_when_carry_set_same_page() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, CARRY, true);
        assert_branch_taken(Cpu::bcs, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn bcs_branches_across_page_boundary() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, CARRY, true);
        assert_branch_taken(Cpu::bcs, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn beq_does_not_branch_when_zero_clear() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, ZERO, false);
        assert_branch_not_taken(Cpu::beq, &mut cpu);
    }

    #[test]
    fn beq_branches_when_zero_set_same_page() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, ZERO, true);
        assert_branch_taken(Cpu::beq, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn beq_branches_across_page_boundary() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, ZERO, true);
        assert_branch_taken(Cpu::beq, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn bit_sets_zero_when_and_result_is_zero() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();

        cpu.a = 0b0000_1111;
        bus.write(0x0000, 0b1111_0000);

        cpu.bit(operand_at(0x0000), &mut bus);

        // No bits overlap, so A & memory is zero.
        assert!(contains(cpu.status, ZERO));
    }

    #[test]
    fn bit_clears_zero_when_and_result_is_nonzero() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();

        cpu.a = 0b0000_1111;
        bus.write(0x0000, 0b0000_1000);

        cpu.bit(operand_at(0x0000), &mut bus);

        assert!(!contains(cpu.status, ZERO));
    }

    #[test]
    fn bit_copies_bit_7_into_negative_flag() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();

        cpu.a = 0xFF;
        bus.write(0x0000, 0b1000_0000);

        cpu.bit(operand_at(0x0000), &mut bus);

        assert!(contains(cpu.status, NEGATIVE));
    }

    #[test]
    fn bit_copies_bit_6_into_overflow_flag() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();

        cpu.a = 0xFF;
        bus.write(0x0000, 0b0100_0000);

        cpu.bit(operand_at(0x0000), &mut bus);

        assert!(contains(cpu.status, OVERFLOW));
    }

    #[test]
    fn bit_does_not_modify_accumulator() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();

        cpu.a = 0x55;
        bus.write(0x0000, 0xFF);

        cpu.bit(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.a, 0x55);
    }

    #[test]
    fn bmi_does_not_branch_when_negative_clear() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, NEGATIVE, false);
        assert_branch_not_taken(Cpu::bmi, &mut cpu);
    }

    #[test]
    fn bmi_branches_when_negative_set_same_page() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, NEGATIVE, true);
        assert_branch_taken(Cpu::bmi, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn bmi_branches_across_page_boundary() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, NEGATIVE, true);
        assert_branch_taken(Cpu::bmi, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn bne_branches_when_zero_clear() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, ZERO, false);
        assert_branch_taken(Cpu::bne, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn bne_does_not_branch_when_zero_is_set() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, ZERO, true);
        assert_branch_not_taken(Cpu::bne, &mut cpu);
    }

    #[test]
    fn bne_branches_across_page_boundary() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, ZERO, false);
        assert_branch_taken(Cpu::bne, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn bpl_branches_when_negative_clear() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, NEGATIVE, false);
        assert_branch_taken(Cpu::bpl, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn bpl_does_not_branch_when_negative_is_set() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, NEGATIVE, true);
        assert_branch_not_taken(Cpu::bpl, &mut cpu);
    }

    #[test]
    fn bpl_branches_across_page_boundary() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, NEGATIVE, false);
        assert_branch_taken(Cpu::bpl, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn brk_pushes_return_address_and_status_then_jumps_to_irq_vector() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();

        cpu.pc = 0x1234;
        cpu.sp = 0xFD;

        // Some arbitrary flags so we can confirm if they have been preserved.
        set(&mut cpu.status, CARRY, true);
        set(&mut cpu.status, NEGATIVE, true);
        set(&mut cpu.status, INTERRUPT_DISABLE, false);

        // Saved status before BRK modifies it.
        let status_before = cpu.status;

        // IRQ/BRK vector: where execution should end up.
        bus.write(0xFFFE, 0x00);
        bus.write(0xFFFF, 0x80); // vector points at 0x8000

        let extra_cycles = cpu.brk(Operand::Accumulator, &mut bus); // operand is unused

        // pc advanced by 1 before pushing, so the return address on the stack should be 0x1235, not 0x1234.
        let expected_return_addr: u16 = 0x1235;

        // Stack pushes happen high byte first, then low byte, then
        // status and sp decrements by 1 after each push. Starting sp
        // was 0xFD, so:
        //   push high byte at 0x0100 + 0xFD = 0x01FD, sp -> 0xFC
        //   push low byte  at 0x0100 + 0xFC = 0x01FC, sp -> 0xFB
        //   push status    at 0x0100 + 0xFB = 0x01FB, sp -> 0xFA
        assert_eq!(
            bus.peek(0x01FD),
            (expected_return_addr >> 8) as u8,
            "high byte of return address"
        );
        assert_eq!(
            bus.peek(0x01FC),
            (expected_return_addr & 0x00FF) as u8,
            "low byte of return address"
        );
        assert_eq!(cpu.sp, 0xFA, "sp must decrement by exactly 3");

        // The pushed status byte must have BREAK and UNUSED forced high while preserving flags
        // already set.
        let pushed_status = bus.peek(0x01FB);
        let expected_status = to_pushed_byte(status_before);
        assert_eq!(pushed_status, expected_status);

        // BREAK is not a "real"" flag. self.status should not have BREAK set after brk() returns.
        assert!(
            !contains(cpu.status, BREAK),
            "BREAK must never be set on the live status"
        );

        assert!(
            contains(cpu.status, INTERRUPT_DISABLE),
            "BRK must set INTERRUPT_DISABLE going forward"
        );

        // Execution must have jumped to the IRQ/BRK vector.
        assert_eq!(cpu.pc, 0x8000);

        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn bvc_branches_when_overflow_clear() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, OVERFLOW, false);
        assert_branch_taken(Cpu::bvc, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn bvc_does_not_branch_when_overflow_is_set() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, OVERFLOW, true);
        assert_branch_not_taken(Cpu::bvc, &mut cpu);
    }

    #[test]
    fn bvc_branches_across_page_boundary() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, OVERFLOW, false);
        assert_branch_taken(Cpu::bvc, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn bvs_branches_when_overflow_is_set() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, OVERFLOW, true);
        assert_branch_taken(Cpu::bvs, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn bvs_does_not_branch_when_overflow_is_clear() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, OVERFLOW, false);
        assert_branch_not_taken(Cpu::bvs, &mut cpu);
    }

    #[test]
    fn bvs_branches_across_page_boundary() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, OVERFLOW, true);
        assert_branch_taken(Cpu::bvs, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn clc_clears_carry_flag() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        set(&mut cpu.status, CARRY, true);

        let extra_cycles = cpu.clc(Operand::Accumulator, &mut bus);

        assert!(!contains(cpu.status, CARRY));
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn cld_clears_decimal_flag() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        set(&mut cpu.status, DECIMAL, true);

        let extra_cycles = cpu.cld(Operand::Accumulator, &mut bus);

        assert!(!contains(cpu.status, DECIMAL));
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn cli_clears_interrupt_disable_flag() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        set(&mut cpu.status, INTERRUPT_DISABLE, true);

        let extra_cycles = cpu.cli(Operand::Accumulator, &mut bus);

        assert!(!contains(cpu.status, INTERRUPT_DISABLE));
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn clv_clears_overflow_flag() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        set(&mut cpu.status, OVERFLOW, true);

        let extra_cycles = cpu.clv(Operand::Accumulator, &mut bus);

        assert!(!contains(cpu.status, OVERFLOW));
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn cmp_sets_carry_and_zero_when_equal() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x42;
        bus.write(0x0000, 0x42);

        let extra_cycles = cpu.cmp(operand_at(0x0000), &mut bus);

        // A == memory: no borrow needed (CARRY set), and the subtraction
        // result is exactly 0 (ZERO set).
        assert!(contains(cpu.status, CARRY));
        assert!(contains(cpu.status, ZERO));
        assert!(!contains(cpu.status, NEGATIVE));
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn cmp_sets_carry_without_zero_when_a_is_greater() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x50;
        bus.write(0x0000, 0x10);

        cpu.cmp(operand_at(0x0000), &mut bus);

        assert!(contains(cpu.status, CARRY));
        assert!(!contains(cpu.status, ZERO));
    }

    #[test]
    fn cmp_clears_carry_when_a_is_less_than_memory() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x10;
        bus.write(0x0000, 0x50);

        cpu.cmp(operand_at(0x0000), &mut bus);

        assert!(!contains(cpu.status, CARRY));
        assert!(!contains(cpu.status, ZERO));
    }

    #[test]
    fn cmp_sets_negative_when_result_high_bit_set() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x10;
        bus.write(0x0000, 0x20);

        cpu.cmp(operand_at(0x0000), &mut bus);

        // 0x10 - 0x20 = -0x10, low byte (0xF0) has bit 7 set.
        assert!(contains(cpu.status, NEGATIVE));
    }

    #[test]
    fn cmp_returns_one_extra_cycle_when_page_crossed() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x10;
        bus.write(0x0000, 0x10);

        let operand = Operand::Address {
            address: 0x0000,
            page_crossed: true,
        };
        let extra_cycles = cpu.cmp(operand, &mut bus);

        assert_eq!(extra_cycles, 1);
    }

    #[test]
    fn cpx_sets_carry_without_zero_when_a_is_greater() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.x = 0x50;
        bus.write(0x0000, 0x10);

        cpu.cpx(operand_at(0x0000), &mut bus);

        assert!(contains(cpu.status, CARRY));
        assert!(!contains(cpu.status, ZERO));
    }

    #[test]
    fn cpx_clears_carry_when_a_is_less_than_memory() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.x = 0x10;
        bus.write(0x0000, 0x50);

        cpu.cpx(operand_at(0x0000), &mut bus);

        assert!(!contains(cpu.status, CARRY));
        assert!(!contains(cpu.status, ZERO));
    }

    #[test]
    fn cpx_sets_negative_when_result_high_bit_set() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.x = 0x10;
        bus.write(0x0000, 0x20);

        cpu.cpx(operand_at(0x0000), &mut bus);

        // 0x10 - 0x20 = -0x10, low byte (0xF0) has bit 7 set.
        assert!(contains(cpu.status, NEGATIVE));
    }

    #[test]
    fn cpx_returns_one_extra_cycle_when_page_crossed() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.x = 0x10;
        bus.write(0x0000, 0x10);

        let operand = Operand::Address {
            address: 0x0000,
            page_crossed: true,
        };
        let extra_cycles = cpu.cpx(operand, &mut bus);

        assert_eq!(extra_cycles, 1);
    }

    #[test]
    fn cpy_sets_carry_without_zero_when_a_is_greater() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.y = 0x50;
        bus.write(0x0000, 0x10);

        cpu.cpy(operand_at(0x0000), &mut bus);

        assert!(contains(cpu.status, CARRY));
        assert!(!contains(cpu.status, ZERO));
    }

    #[test]
    fn cpy_clears_carry_when_a_is_less_than_memory() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.y = 0x10;
        bus.write(0x0000, 0x50);

        cpu.cpy(operand_at(0x0000), &mut bus);

        assert!(!contains(cpu.status, CARRY));
        assert!(!contains(cpu.status, ZERO));
    }

    #[test]
    fn cpy_sets_negative_when_result_high_bit_set() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.y = 0x10;
        bus.write(0x0000, 0x20);

        cpu.cpy(operand_at(0x0000), &mut bus);

        // 0x10 - 0x20 = -0x10, low byte (0xF0) has bit 7 set.
        assert!(contains(cpu.status, NEGATIVE));
    }

    #[test]
    fn cpy_returns_one_extra_cycle_when_page_crossed() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.y = 0x10;
        bus.write(0x0000, 0x10);

        let operand = Operand::Address {
            address: 0x0000,
            page_crossed: true,
        };
        let extra_cycles = cpu.cpx(operand, &mut bus);

        assert_eq!(extra_cycles, 1);
    }

    #[test]
    fn dec_subtracts_one_from_memory() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0x10);

        let extra_cycles = cpu.dec(operand_at(0x0000), &mut bus);

        assert_eq!(bus.peek(0x0000), 0x0F);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn dec_wraps_from_zero_to_0xff() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0x00);

        cpu.dec(operand_at(0x0000), &mut bus);

        // Decrementing 0x00 must wrap to 0xFF.
        assert_eq!(bus.peek(0x0000), 0xFF);
        assert!(contains(cpu.status, NEGATIVE)); // 0xFF has bit 7 set
        assert!(!contains(cpu.status, ZERO));
    }

    #[test]
    fn dec_sets_zero_when_result_is_zero() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0x01);

        cpu.dec(operand_at(0x0000), &mut bus);

        assert_eq!(bus.peek(0x0000), 0x00);
        assert!(contains(cpu.status, ZERO));
        assert!(!contains(cpu.status, NEGATIVE));
    }

    #[test]
    fn dec_sets_negative_when_result_high_bit_set() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0x00); // will wrap to 0xFF, bit 7 set

        cpu.dec(operand_at(0x0000), &mut bus);

        assert!(contains(cpu.status, NEGATIVE));
    }

    #[test]
    fn dex_subtracts_one_from_x() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.x = 0x10;

        let extra_cycles = cpu.dex(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.x, 0x0F);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn dex_wraps_from_zero_to_0xff() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.x = 0x00;

        cpu.dex(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.x, 0xFF);
        assert!(contains(cpu.status, NEGATIVE));
        assert!(!contains(cpu.status, ZERO));
    }

    #[test]
    fn dex_sets_zero_when_result_is_zero() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.x = 0x01;

        cpu.dex(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.x, 0x00);
        assert!(contains(cpu.status, ZERO));
    }

    #[test]
    fn dey_subtracts_one_from_y() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.y = 0x10;

        let extra_cycles = cpu.dey(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.y, 0x0F);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn dey_wraps_from_zero_to_0xff() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.y = 0x00;

        cpu.dey(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.y, 0xFF);
        assert!(contains(cpu.status, NEGATIVE));
        assert!(!contains(cpu.status, ZERO));
    }

    #[test]
    fn dey_sets_zero_when_result_is_zero() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.y = 0x01;

        cpu.dey(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.y, 0x00);
        assert!(contains(cpu.status, ZERO));
    }

    #[test]
    fn eor_xors_accumulator_with_memory() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0b1100_1100;
        bus.write(0x0000, 0b1010_1010);

        let extra_cycles = cpu.eor(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.a, 0b0110_0110);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn eor_with_0xff_inverts_all_bits() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0b1010_0101;
        bus.write(0x0000, 0xFF);

        cpu.eor(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.a, 0b0101_1010);
    }

    #[test]
    fn eor_sets_zero_when_operands_are_identical() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x77;
        bus.write(0x0000, 0x77);

        cpu.eor(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.a, 0x00);
        assert!(contains(cpu.status, ZERO));
    }

    #[test]
    fn eor_sets_negative_when_result_high_bit_set() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x00;
        bus.write(0x0000, 0x80);

        cpu.eor(operand_at(0x0000), &mut bus);

        assert!(contains(cpu.status, NEGATIVE));
    }

    #[test]
    fn eor_returns_one_extra_cycle_when_page_crossed() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x0F;
        bus.write(0x0000, 0xF0);

        let operand = Operand::Address {
            address: 0x0000,
            page_crossed: true,
        };
        let extra_cycles = cpu.eor(operand, &mut bus);

        assert_eq!(extra_cycles, 1);
    }

    #[test]
    fn inc_adds_one_to_memory() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0x10);

        let extra_cycles = cpu.inc(operand_at(0x0000), &mut bus);

        assert_eq!(bus.peek(0x0000), 0x11);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn inc_wraps_from_0xff_to_zero() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0xFF);

        cpu.inc(operand_at(0x0000), &mut bus);

        assert_eq!(bus.peek(0x0000), 0x00);
        assert!(contains(cpu.status, ZERO));
        assert!(!contains(cpu.status, NEGATIVE));
    }

    #[test]
    fn inx_adds_one_to_x() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.x = 0x10;

        let extra_cycles = cpu.inx(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.x, 0x11);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn inx_wraps_from_0xff_to_zero() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.x = 0xFF;

        cpu.inx(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.x, 0x00);
        assert!(contains(cpu.status, ZERO));
    }

    #[test]
    fn iny_adds_one_to_y() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.y = 0x10;

        let extra_cycles = cpu.iny(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.y, 0x11);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn iny_wraps_from_0xff_to_zero() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.y = 0xFF;

        cpu.iny(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.y, 0x00);
        assert!(contains(cpu.status, ZERO));
    }
}
