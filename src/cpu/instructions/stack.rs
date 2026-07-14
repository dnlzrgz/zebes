use crate::{
    bus::Bus,
    cpu::{Cpu, addressing::Operand, flags::*},
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

    /// Pull A
    /// SP = SP + 1
    /// A = ($0100 + SP)
    ///
    /// PLA increments the stack pointer and then loads the value at that stack position into A.
    pub fn pla(&mut self, _: Operand, bus: &mut Bus) -> u8 {
        self.a = self.pull_byte(bus);
        self.update_zn(self.a);
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

    /// Pull Processor Status
    /// SP = SP + 1
    /// NVxxDIZC = ($0100 + SP)
    ///
    /// PLP increments the stack pointer and then loads the value at that stack position into the 6
    /// status flags. The bit order is NVxxDIZC (high to low). The B flag and extra bit are ignored.
    /// Note that the effect of changing I is delayed one instruction because the flag is changed
    /// after IRQ is polled, delaying the effect until IRQ is polled in the next instruction like
    /// with CLI and SEI.
    pub fn plp(&mut self, _: Operand, bus: &mut Bus) -> u8 {
        let pulled = self.pull_byte(bus);
        self.status = (pulled & !BREAK) | UNUSED;
        0
    }

    /// Transfer X to Stack Pointer
    /// SP = X
    ///
    /// TXS copies the X register value to the stack pointer.
    pub fn txs(&mut self, _: Operand, _: &mut Bus) -> u8 {
        self.sp = self.x;
        0
    }

    /// Transfer Stack Pointer to X
    /// X = SP
    ///
    /// TSX copies the stack pointer value to the X register.
    pub fn tsx(&mut self, _: Operand, _: &mut Bus) -> u8 {
        self.x = self.sp;
        self.update_zn(self.x);
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pha_pushes_accumulator_onto_stack() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x42;
        cpu.sp = 0xFD;

        let extra_cycles = cpu.pha(Operand::Accumulator, &mut bus);

        assert_eq!(bus.peek(0x01FD), 0x42);
        assert_eq!(cpu.sp, 0xFC);
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
    fn pla_loads_pulled_value_into_accumulator() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0xFC;
        bus.write(0x01FD, 0x42);

        let extra_cycles = cpu.pla(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.sp, 0xFD);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn pla_mirrors_a_previous_pha() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x77;
        cpu.sp = 0xFD;

        cpu.pha(Operand::Accumulator, &mut bus);
        cpu.a = 0x00;

        cpu.pla(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.a, 0x77);
        assert_eq!(cpu.sp, 0xFD);
    }

    #[test]
    fn pla_sets_zero_when_pulled_value_is_zero() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0xFF;
        cpu.sp = 0xFC;
        bus.write(0x01FD, 0x00);

        cpu.pla(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.a, 0x00);
        assert!(contains(cpu.status, ZERO));
    }

    #[test]
    fn pla_sets_negative_when_pulled_value_has_high_bit_set() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0xFC;
        bus.write(0x01FD, 0x80);

        cpu.pla(Operand::Accumulator, &mut bus);

        assert!(contains(cpu.status, NEGATIVE));
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
        assert!(contains(pushed, BREAK));
        assert!(contains(pushed, UNUSED));
        assert!(contains(pushed, CARRY));
        assert!(contains(pushed, NEGATIVE));
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

    #[test]
    fn plp_restores_status_from_stack() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0xFC;
        bus.write(0x01FD, CARRY | ZERO | NEGATIVE);

        let extra_cycles = cpu.plp(Operand::Accumulator, &mut bus);

        assert!(contains(cpu.status, CARRY));
        assert!(contains(cpu.status, ZERO));
        assert!(contains(cpu.status, NEGATIVE));
        assert_eq!(cpu.sp, 0xFD);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn plp_forces_break_low_regardless_of_pulled_value() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0xFC;
        bus.write(0x01FD, CARRY | BREAK);

        cpu.plp(Operand::Accumulator, &mut bus);

        assert!(!contains(cpu.status, BREAK));
        assert!(contains(cpu.status, CARRY));
    }

    #[test]
    fn plp_forces_unused_high_regardless_of_pulled_value() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0xFC;
        bus.write(0x01FD, CARRY);

        cpu.plp(Operand::Accumulator, &mut bus);

        assert!(contains(cpu.status, UNUSED));
    }

    #[test]
    fn plp_mirrors_a_previous_php() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0xFD;
        set(&mut cpu.status, CARRY, true);
        set(&mut cpu.status, NEGATIVE, true);
        let status_before = cpu.status;

        cpu.php(Operand::Accumulator, &mut bus);
        set(&mut cpu.status, CARRY, false);
        set(&mut cpu.status, NEGATIVE, false);

        cpu.plp(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.status, status_before);
        assert_eq!(cpu.sp, 0xFD);
    }

    #[test]
    fn txs_copies_x_into_stack_pointer() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.x = 0x42;

        let extra_cycles = cpu.txs(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.sp, 0x42);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn txs_does_not_affect_flags() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.x = 0x00;
        let status_before = cpu.status;

        cpu.txs(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.status, status_before);
    }

    #[test]
    fn tsx_copies_stack_pointer_into_x() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0x42;

        let extra_cycles = cpu.tsx(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.x, 0x42);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn tsx_sets_zero_when_stack_pointer_is_zero() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0x00;
        cpu.x = 0xFF;

        cpu.tsx(Operand::Accumulator, &mut bus);

        assert!(contains(cpu.status, ZERO));
    }

    #[test]
    fn tsx_sets_negative_when_stack_pointer_has_high_bit_set() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0x80;

        cpu.tsx(Operand::Accumulator, &mut bus);

        assert!(contains(cpu.status, NEGATIVE));
    }
}
