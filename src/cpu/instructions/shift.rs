use crate::{
    bus::Bus,
    cpu::{Cpu, addressing::Operand, flags::*},
};

impl Cpu {
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

    /// Logical Shift Right
    /// value = value >> 1
    ///
    /// LSR shifts all the bits of a memory value or the accumulator one position to the right,
    /// moving the value of each bit into the next bit. 0 is shifted into bit 7, and bit 0 is
    /// shifted into the carry flag. This is equivalent to dividing an unsigned value by 2 and
    /// rounding down, with the remainder carry.
    /// This is a read-modify-write instruction, meaning that its addressing modes that operate on
    /// memory first write the original value back to memory before the modified value. This extra
    /// write can matter if targeting a hardware register.
    pub fn lsr(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let value = match operand {
            Operand::Accumulator => self.a,
            Operand::Address { address, .. } => bus.read(address),
        };

        set(&mut self.status, CARRY, value & 0x01 != 0);

        let result = value >> 1;
        self.update_zn(result);
        match operand {
            Operand::Accumulator => self.a = result,
            Operand::Address { address, .. } => bus.write(address, result),
        };

        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::instructions::test_utils::operand_at;

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
    fn lsr_shifts_memory_value_right() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0b0000_0010);

        cpu.lsr(operand_at(0x0000), &mut bus);

        // The shifted result must be written back to the same address it was read from.
        assert_eq!(bus.peek(0x0000), 0b0000_0001);
    }

    #[test]
    fn lsr_sets_carry_when_bit_0_shifts_out() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0b0000_0001); // bit 0 set

        cpu.lsr(operand_at(0x0000), &mut bus);

        assert_eq!(bus.peek(0x0000), 0b0000_0000);
        assert!(contains(cpu.status, CARRY));
        assert!(contains(cpu.status, ZERO));
    }

    #[test]
    fn lsr_does_not_set_carry_when_bit_0_is_clear() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0b0000_0010); // bit 0 clear, bit 1 set

        cpu.lsr(operand_at(0x0000), &mut bus);

        assert_eq!(bus.peek(0x0000), 0b0000_0001);
        assert!(!contains(cpu.status, CARRY));
    }

    #[test]
    fn lsr_never_sets_negative() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0b1111_1111); // bit 7 set going in

        cpu.lsr(operand_at(0x0000), &mut bus);

        // 0 is always shifted into bit 7 on a right shift, so NEGATIVE
        // can never be set as a result of LSR, regardless of the input.
        assert_eq!(bus.peek(0x0000), 0b0111_1111);
        assert!(!contains(cpu.status, NEGATIVE));
    }

    #[test]
    fn lsr_accumulator_mode_shifts_register_not_memory() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0b0000_0110;
        bus.write(0x0000, 0xFF); // must remain untouched

        cpu.lsr(Operand::Accumulator, &mut bus);

        // No bus access should happen at all.
        assert_eq!(cpu.a, 0b0000_0011);
        assert_eq!(bus.peek(0x0000), 0xFF);
    }

    #[test]
    fn lsr_accumulator_mode_sets_carry_correctly() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0b0000_0011;

        cpu.lsr(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.a, 0b0000_0001);
        assert!(contains(cpu.status, CARRY));
    }
}
