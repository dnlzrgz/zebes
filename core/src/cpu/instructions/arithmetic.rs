use crate::{
    bus::Bus,
    cpu::{Cpu, addressing::Operand, flags::*},
};

impl Cpu {
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

    /// Subtract with Carry
    /// A = A - memory - ~C || A = A + ~memory + C
    ///
    /// SBC subtracts a memory value and the NOT of the carry flag from the accumulator. It does
    /// this by adding the bitwise NOT of the memory value using ADC. This implementation detail
    /// explains the backward nature of carry; SBC subtracts 1 more when carry is clear, not
    /// when it's set, and carry is cleared when it underflows and set otherwise. As with ADC, carry
    /// allows the borrow from one subtraction to be carried into the next subtraction, allowing
    /// subtraction of values larger than 1 byte. It is common to set carry with SEC before
    /// subtracting the first byte to ensure it is a known state, avoiding an off-by-one error.
    /// Overflow works the same as with ADC, except with an inverted memory value. Therefore,
    /// overflow or underflow occur if result's sign is different from A's and the same as the
    /// memory value's.
    pub fn sbc(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let (address, page_crossed) = operand.expect_address();
        let value = bus.read(address);
        let inverted_value = value ^ 0xFF;

        let carry_in = contains(self.status, CARRY) as u16;
        let sum = self.a as u16 + inverted_value as u16 + carry_in;
        let result = sum as u8;

        set(&mut self.status, CARRY, sum > 0xFF);

        let overflow = (!(self.a ^ inverted_value) & (self.a ^ result) & 0x80) != 0;
        set(&mut self.status, OVERFLOW, overflow);

        self.update_zn(result);
        self.a = result;

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

    /// Increment X
    /// X = X + 1
    ///
    /// INX adds 1 from the X register. Note that it does not affect carry nor overflow.
    pub fn inx(&mut self, _: Operand, _: &mut Bus) -> u8 {
        self.x = self.x.wrapping_add(1);
        self.update_zn(self.x);

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

    /// Increment Y
    /// Y = Y + 1
    ///
    /// INY adds 1 from the Y register. Note that it does not affect carry nor overflow.
    pub fn iny(&mut self, _: Operand, _: &mut Bus) -> u8 {
        self.y = self.y.wrapping_add(1);
        self.update_zn(self.y);

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::instructions::test_utils::operand_at;

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
    fn sbc_simple_subtraction_with_carry_set() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x10;
        set(&mut cpu.status, CARRY, true);
        bus.write(0x0000, 0x05);

        let extra_cycles = cpu.sbc(operand_at(0x0000), &mut bus);

        // 16 - 5 = 11 (0x0B)
        assert_eq!(cpu.a, 0x0B);
        assert!(contains(cpu.status, CARRY),);
        assert!(!contains(cpu.status, ZERO));
        assert!(!contains(cpu.status, OVERFLOW));
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn sbc_clear_carry_subtracts_one_extra() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x10;
        set(&mut cpu.status, CARRY, false);
        bus.write(0x0000, 0x05);

        cpu.sbc(operand_at(0x0000), &mut bus);

        // 16 - 5 - 1 (borrowed) = 10 (0x0A)
        assert_eq!(cpu.a, 0x0A);
    }

    #[test]
    fn sbc_clears_carry_on_borrow() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x10;
        set(&mut cpu.status, CARRY, true);
        bus.write(0x0000, 0x20);

        cpu.sbc(operand_at(0x0000), &mut bus);

        // A borrow happened, so CARRY must be cleared.
        assert!(!contains(cpu.status, CARRY));
    }

    #[test]
    fn sbc_sets_zero_when_result_is_zero() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x10;
        set(&mut cpu.status, CARRY, true);
        bus.write(0x0000, 0x10);

        cpu.sbc(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.a, 0x00);
        assert!(contains(cpu.status, ZERO));
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
}
