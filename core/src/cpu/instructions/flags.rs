use crate::{
    cpu::{Cpu, addressing::Operand, flags::*},
    cpu_bus::CpuBus,
};

impl Cpu {
    /// Clear Carry
    /// C = 0
    ///
    /// CLC clears the carry flag. In particular, this is usually done before adding the low byte of
    /// a value with ADC to avoid adding an extra 1.
    pub fn clc(&mut self, _: Operand, _: &mut CpuBus) -> u8 {
        set(&mut self.status, CARRY, false);
        0
    }

    /// Set Carry
    /// C = 1
    ///
    /// SEC sets the carry flag. In particular, this is usually done before subtracting the low byte
    /// of a value with SBC to avoid subtracting an extra 1.
    pub fn sec(&mut self, _: Operand, _: &mut CpuBus) -> u8 {
        set(&mut self.status, CARRY, true);
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
    pub fn cli(&mut self, _: Operand, _: &mut CpuBus) -> u8 {
        set(&mut self.status, INTERRUPT_DISABLE, false);
        0
    }

    /// Set Interrupt Disable
    /// I = 1
    ///
    /// SEI sets the interrupt disable flag, preventing the CPU from handling hardware IRQs. The
    /// effect of changing this flag is delayed one instruction because the flag is changed after
    /// IRQ is polled, allowing an IRQ to be serviced between this and the next instruction if the
    /// flag was previously 0.
    pub fn sei(&mut self, _: Operand, _: &mut CpuBus) -> u8 {
        set(&mut self.status, INTERRUPT_DISABLE, true);
        0
    }

    /// Clear Decimal
    /// D = 0
    ///
    /// CLD clears the decimal flag. The decimal flag normally controls whether binary-coded decimal
    /// mode (BCD) is enabled, but this mode is permanently disabled on the NES' 2A03 CPU. However,
    /// the flag itself still functions and can be used to store state.
    pub fn cld(&mut self, _: Operand, _: &mut CpuBus) -> u8 {
        set(&mut self.status, DECIMAL, false);
        0
    }

    /// Set Decimal
    /// D = 1
    ///
    /// SED sets the decimal flag. The decimal flag normally controls whether binary-coded decimal
    /// mode (BCD) is enabled, but this mode is permanently disabled on the NES' 2A03 CPU. However,
    /// the flag itself still functions and can be used to store state.
    pub fn sed(&mut self, _: Operand, _: &mut CpuBus) -> u8 {
        set(&mut self.status, DECIMAL, true);
        0
    }

    /// Clear Overflow
    /// V = 0
    ///
    /// CLV clears the overflow flag. There is no corresponding SEV instruction; instead, setting
    /// overflow is exposed on the 6502 CPU as a pin controller by external hardware, and not
    /// exposed at all on the NES' 2A03 CPU.
    pub fn clv(&mut self, _: Operand, _: &mut CpuBus) -> u8 {
        set(&mut self.status, OVERFLOW, false);
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clc_clears_carry_flag() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        set(&mut cpu.status, CARRY, true);

        let extra_cycles = cpu.clc(Operand::Accumulator, &mut bus);

        assert!(!contains(cpu.status, CARRY));
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn sec_sets_carry_flag() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        set(&mut cpu.status, CARRY, false);

        let extra_cycles = cpu.sec(Operand::Accumulator, &mut bus);

        assert!(contains(cpu.status, CARRY));
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn cli_clears_interrupt_disable_flag() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        set(&mut cpu.status, INTERRUPT_DISABLE, true);

        let extra_cycles = cpu.cli(Operand::Accumulator, &mut bus);

        assert!(!contains(cpu.status, INTERRUPT_DISABLE));
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn sei_sets_interrupt_disable_flag() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        set(&mut cpu.status, INTERRUPT_DISABLE, false);

        let extra_cycles = cpu.sei(Operand::Accumulator, &mut bus);

        assert!(contains(cpu.status, INTERRUPT_DISABLE));
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn cld_clears_decimal_flag() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        set(&mut cpu.status, DECIMAL, true);

        let extra_cycles = cpu.cld(Operand::Accumulator, &mut bus);

        assert!(!contains(cpu.status, DECIMAL));
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn sed_sets_decimal_flag() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        set(&mut cpu.status, DECIMAL, false);

        let extra_cycles = cpu.sed(Operand::Accumulator, &mut bus);

        assert!(contains(cpu.status, DECIMAL));
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn clv_clears_overflow_flag() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        set(&mut cpu.status, OVERFLOW, true);

        let extra_cycles = cpu.clv(Operand::Accumulator, &mut bus);

        assert!(!contains(cpu.status, OVERFLOW));
        assert_eq!(extra_cycles, 0);
    }
}
