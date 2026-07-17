use crate::{
    cpu::{Cpu, addressing::Operand},
    cpu_bus::CpuBus,
};

impl Cpu {
    /// Transfer A to X
    /// X = A
    ///
    /// TAX copies the accumulator value to the X register.
    pub fn tax(&mut self, _: Operand, _: &mut CpuBus) -> u8 {
        self.x = self.a;
        self.update_zn(self.x);
        0
    }

    /// Transfer X to A
    /// A = X
    ///
    /// TXA copies the X register value to the accumulator.
    pub fn txa(&mut self, _: Operand, _: &mut CpuBus) -> u8 {
        self.a = self.x;
        self.update_zn(self.a);
        0
    }

    /// Transfer A to Y
    /// Y = A
    ///
    /// TAY copies the accumulator value to the Y register.
    pub fn tay(&mut self, _: Operand, _: &mut CpuBus) -> u8 {
        self.y = self.a;
        self.update_zn(self.y);
        0
    }

    /// Transfer Y to A
    /// A = Y
    ///
    /// TYA copies the Y register value to the accumulator.
    pub fn tya(&mut self, _: Operand, _: &mut CpuBus) -> u8 {
        self.a = self.y;
        self.update_zn(self.a);
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::flags::*;

    #[test]
    fn tax_copies_accumulator_into_x() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x42;

        let extra_cycles = cpu.tax(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.x, 0x42);
        assert_eq!(cpu.a, 0x42);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn tax_sets_negative_when_accumulator_has_high_bit_set() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x80;

        cpu.tax(Operand::Accumulator, &mut bus);

        assert!(contains(cpu.status, NEGATIVE));
    }

    #[test]
    fn txa_copies_x_into_accumulator() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.x = 0x42;

        let extra_cycles = cpu.txa(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.x, 0x42);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn txa_flags_reflect_the_copied_value() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.x = 0x00;
        cpu.a = 0xFF;

        cpu.txa(Operand::Accumulator, &mut bus);

        assert!(contains(cpu.status, ZERO),);
    }

    #[test]
    fn tay_copies_accumulator_into_y() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x42;

        let extra_cycles = cpu.tay(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.y, 0x42);
        assert_eq!(cpu.a, 0x42);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn tay_sets_negative_when_accumulator_has_high_bit_set() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x80;

        cpu.tay(Operand::Accumulator, &mut bus);

        assert!(contains(cpu.status, NEGATIVE));
    }

    #[test]
    fn tya_copies_y_into_accumulator() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.y = 0x42;

        let extra_cycles = cpu.tya(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.y, 0x42);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn tya_flags_reflect_the_copied_value() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.y = 0x00;
        cpu.a = 0xFF;

        cpu.tya(Operand::Accumulator, &mut bus);

        assert!(contains(cpu.status, ZERO),);
    }
}
