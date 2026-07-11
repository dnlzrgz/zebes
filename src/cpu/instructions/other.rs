use crate::{
    bus::Bus,
    cpu::{Cpu, addressing::Operand},
};

impl Cpu {
    /// No Operation
    ///
    /// NOP has no effect; it merely wastes space and CPU cycles. This instruction can be useful
    /// when writing timed code to delay for a desired amount of time, as padding to ensure that
    /// something does or does not cross a page, or to disable code in a binary.
    #[inline]
    pub fn nop(&mut self, _: Operand, _: &mut Bus) -> u8 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nop_does_nothing_and_returns_zero_cycles() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x11;
        cpu.x = 0x22;
        cpu.y = 0x33;
        cpu.pc = 0x1000;
        let status_before = cpu.status;
        bus.write(0x0000, 0xAB);

        let extra_cycles = cpu.nop(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.a, 0x11);
        assert_eq!(cpu.x, 0x22);
        assert_eq!(cpu.y, 0x33);
        assert_eq!(cpu.pc, 0x1000);
        assert_eq!(cpu.status, status_before);
        assert_eq!(bus.peek(0x0000), 0xAB);
        assert_eq!(extra_cycles, 0);
    }
}
