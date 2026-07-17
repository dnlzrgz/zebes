use crate::{
    cpu::{Cpu, addressing::Operand},
    cpu_bus::CpuBus,
};

impl Cpu {
    /// Load A
    /// A = memory
    ///
    /// LDA loads a memory value into the accumulator.
    pub fn lda(&mut self, operand: Operand, bus: &mut CpuBus) -> u8 {
        let (address, page_crossed) = operand.expect_address();
        self.a = bus.read(address);
        self.update_zn(self.a);

        page_crossed as u8
    }

    /// Store A
    /// memory = A
    ///
    /// STA stores the accumulator value into memory.
    pub fn sta(&mut self, operand: Operand, bus: &mut CpuBus) -> u8 {
        let (address, _) = operand.expect_address();
        bus.write(address, self.a);
        0
    }

    /// Load X
    /// X = memory
    ///
    /// LDX loads a memory value into the X register.
    pub fn ldx(&mut self, operand: Operand, bus: &mut CpuBus) -> u8 {
        let (address, page_crossed) = operand.expect_address();
        self.x = bus.read(address);
        self.update_zn(self.x);

        page_crossed as u8
    }

    /// Store X
    /// memory = X
    ///
    /// STX stores the X register value into memory.
    pub fn stx(&mut self, operand: Operand, bus: &mut CpuBus) -> u8 {
        let (address, _) = operand.expect_address();
        bus.write(address, self.x);
        0
    }

    /// Load Y
    /// Y = memory
    ///
    /// LDY loads a memory value into the Y register.
    pub fn ldy(&mut self, operand: Operand, bus: &mut CpuBus) -> u8 {
        let (address, page_crossed) = operand.expect_address();
        self.y = bus.read(address);
        self.update_zn(self.y);

        page_crossed as u8
    }

    /// Store Y
    /// memory = Y
    ///
    /// STY stores the Y register value into memory.
    pub fn sty(&mut self, operand: Operand, bus: &mut CpuBus) -> u8 {
        let (address, _) = operand.expect_address();
        bus.write(address, self.y);
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::instructions::test_utils::operand_at;

    #[test]
    fn lda_loads_memory_into_accumulator() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0x42);

        let extra_cycles = cpu.lda(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.a, 0x42);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn lda_returns_extra_cycle_when_page_crossed() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0x10);

        let operand = Operand::Address {
            address: 0x0000,
            page_crossed: true,
        };
        let extra_cycles = cpu.lda(operand, &mut bus);

        assert_eq!(extra_cycles, 1);
    }

    #[test]
    fn sta_stores_accumulator_into_memory() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.a = 0x42;

        let extra_cycles = cpu.sta(operand_at(0x0000), &mut bus);

        assert_eq!(bus.peek(0x0000), 0x42);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn ldx_loads_memory_into_x() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0x42);

        let extra_cycles = cpu.ldx(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.x, 0x42);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn stx_stores_x_into_memory() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.x = 0x42;

        let extra_cycles = cpu.stx(operand_at(0x0000), &mut bus);

        assert_eq!(bus.peek(0x0000), 0x42);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn ldy_loads_memory_into_y() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0x42);

        let extra_cycles = cpu.ldy(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.y, 0x42);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn sty_stores_y_into_memory() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.y = 0x42;

        let extra_cycles = cpu.sty(operand_at(0x0000), &mut bus);

        assert_eq!(bus.peek(0x0000), 0x42);
        assert_eq!(extra_cycles, 0);
    }
}
