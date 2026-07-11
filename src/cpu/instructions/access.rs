use crate::{
    bus::Bus,
    cpu::{Cpu, addressing::Operand},
};

impl Cpu {
    /// Load A
    /// A = memory
    ///
    ///LDA loads a memory value into the accumulator.
    pub fn lda(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let (address, page_crossed) = operand.expect_address();
        self.a = bus.read(address);
        self.update_zn(self.a);

        page_crossed as u8
    }

    /// Load X
    /// X = memory
    ///
    ///LDX loads a memory value into the X register.
    pub fn ldx(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let (address, page_crossed) = operand.expect_address();
        self.x = bus.read(address);
        self.update_zn(self.x);

        page_crossed as u8
    }

    /// Load Y
    /// Y = memory
    ///
    ///LDY loads a memory value into the Y register.
    pub fn ldy(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        let (address, page_crossed) = operand.expect_address();
        self.y = bus.read(address);
        self.update_zn(self.y);

        page_crossed as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::instructions::test_utils::operand_at;

    #[test]
    fn lda_loads_memory_into_accumulator() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0x42);

        let extra_cycles = cpu.lda(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.a, 0x42);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn lda_returns_extra_cycle_when_page_crossed() {
        let mut bus = Bus::new();
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
    fn ldx_loads_memory_into_x() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0x42);

        let extra_cycles = cpu.ldx(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.x, 0x42);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn ldy_loads_memory_into_y() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        bus.write(0x0000, 0x42);

        let extra_cycles = cpu.ldy(operand_at(0x0000), &mut bus);

        assert_eq!(cpu.y, 0x42);
        assert_eq!(extra_cycles, 0);
    }
}
