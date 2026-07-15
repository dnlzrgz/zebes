use crate::{
    bus::Bus,
    cpu::{Cpu, addressing::Operand, flags::*},
};

impl Cpu {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::instructions::test_utils::operand_at;

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
}
