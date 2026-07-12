use crate::{
    bus::Bus,
    cpu::{Cpu, addressing::Operand, flags::*},
};

impl Cpu {
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

    /// Bitwise OR
    /// A = A | memory
    ///
    /// ORA inclusive-ORs a memory value and the accumulator, bit by bit. If either input bit is 1,
    /// the resulting bit is 1. Otherwise, it is 0.
    pub fn ora(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let (address, page_crossed) = operand.expect_address();
        let value = bus.read(address);

        self.a |= value;
        self.update_zn(self.a);

        page_crossed as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::instructions::test_utils::operand_at;

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
    fn ora_ors_accumulator_with_memory() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0b1100_0000;
        bus.write(0x0000, 0b0000_1100);

        let extra_cycles = cpu.ora(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.a, 0b1100_1100);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn ora_sets_zero_when_both_operands_are_zero() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x00;
        bus.write(0x0000, 0x00);

        cpu.ora(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.a, 0x00);
        assert!(contains(cpu.status, ZERO));
    }

    #[test]
    fn ora_sets_negative_when_result_high_bit_set() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x00;
        bus.write(0x0000, 0x80);

        cpu.ora(operand_at(0x0000), &mut bus);

        assert!(contains(cpu.status, NEGATIVE));
    }

    #[test]
    fn ora_returns_one_extra_cycle_when_page_crossed() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x0F;
        bus.write(0x0000, 0xF0);

        let operand = Operand::Address {
            address: 0x0000,
            page_crossed: true,
        };
        let extra_cycles = cpu.ora(operand, &mut bus);

        assert_eq!(extra_cycles, 1);
    }
}
