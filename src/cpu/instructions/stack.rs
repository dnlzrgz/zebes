use crate::{
    bus::Bus,
    cpu::{Cpu, addressing::Operand, flags::to_pushed_byte},
};

impl Cpu {
    /// Push A
    /// ($0100 + SP) = A
    /// SP = SP - 1
    ///
    /// PHA stores the value of A to the current stack position and then decrements the stack
    /// pointer.
    pub fn pha(&mut self, _: Operand, bus: &mut Bus) -> u8 {
        self.push_byte(bus, self.a);
        0
    }

    /// Push Processor Status
    /// ($0100 + SP) = NV11DIZC
    ///
    /// PHP stores a byte to the stack containing the 6 status flags and B flag and then decrements
    /// the stack pointer. The B flag and extra bit are both pushed as 1. The bit order is NV1BDIZC
    /// (high to low).
    /// SP = SP - 1
    pub fn php(&mut self, _: Operand, bus: &mut Bus) -> u8 {
        let pushed_status = to_pushed_byte(self.status);
        self.push_byte(bus, pushed_status);
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::flags::*;

    #[test]
    fn pha_pushes_accumulator_onto_stack() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x42;
        cpu.sp = 0xFD;

        let extra_cycles = cpu.pha(Operand::Accumulator, &mut bus);

        assert_eq!(bus.peek(0x01FD), 0x42);
        assert_eq!(cpu.sp, 0xFC, "sp must decrement by 1");
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn pha_does_not_modify_accumulator_or_flags() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x42;
        let status_before = cpu.status;

        cpu.pha(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.status, status_before);
    }

    #[test]
    fn php_pushes_status_with_break_and_unused_forced_high() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0xFD;
        set(&mut cpu.status, CARRY, true);
        set(&mut cpu.status, NEGATIVE, true);
        set(&mut cpu.status, INTERRUPT_DISABLE, false);

        let extra_cycles = cpu.php(Operand::Accumulator, &mut bus);

        let pushed = bus.peek(0x01FD);
        assert!(contains(pushed, BREAK), "pushed status must have BREAK set");
        assert!(
            contains(pushed, UNUSED),
            "pushed status must have UNUSED set"
        );
        assert!(
            contains(pushed, CARRY),
            "pre-existing flags must be preserved"
        );
        assert!(
            contains(pushed, NEGATIVE),
            "pre-existing flags must be preserved"
        );
        assert_eq!(cpu.sp, 0xFC);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn php_does_not_set_break_on_the_live_status_register() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();

        cpu.php(Operand::Accumulator, &mut bus);

        assert!(!contains(cpu.status, BREAK));
    }
}
