use crate::{cpu::Cpu, cpu_bus::CpuBus};

/// The 6502's addressing modes: each describes a way of computing
/// the effective address (or lack thereof) that an instruction operates on.
#[derive(Clone, Copy, Debug)]
pub enum AddressingMode {
    /// Operand is the accumulator itself.
    Accumulator,

    /// There is no additional data required for this instruction.
    Implied,

    /// Expects the next byte to be used as a value, so it's important
    /// to prepare the read address to point to the next byte.
    Immediate,

    /// To save program bytes, zero page addressing allows us to absolutely address
    /// a location in first 0xFF bytes of address range.
    ZeroPage,

    /// The same as `ZeroPage` but the contents of the `x` register is added to the
    /// supplied single byte address.
    ZeroPageX,

    /// Same as `ZeroPageX` but using the `y` register.
    ZeroPageY,

    /// This address mode is exclusive to branch instructions. The address must reside within
    /// -128 to +127 (signed 8-bit offset) of the branch instruction.
    Relative,

    /// A full 16-bit address is loaded and used.
    Absolute,

    /// The same as absolute addressing but the contents of the `x` register
    /// are added to the supplied two byte address. If the resulting address
    /// changes the page, an additional clock cycle is required.
    AbsoluteX,

    /// Same as `AbsoluteX` but using the `y` register.
    AbsoluteY,

    /// The supplied 16-bit address is read to get the actual 16-bit address. This
    /// instruction is unusual in the sense that is a literal bug in the hardware.
    ///
    /// If the low byte of the supplied address is 0xFF, then to read the high byte of
    /// the actual address we need to cross a page boundary. Since this doesn't work on the chip
    /// as designed, it wraps back around in the same page, yielding an invalid actual address.
    Indirect,

    /// The supplied 8-bit address is offset by the `x` register to index
    /// a location in page 0x00. The actual 16-bit address is read from this location.
    IndirectX,

    /// The supplied 8-bit address is offset by the `y` register to index
    /// a location in page 0x00. From here the actual 16-bit address is read, and the
    /// contents of the `y` register is added to the offset.
    /// If the offset causes a change in page, an additional clock cycle is required.
    IndirectY,
}

/// Result of solving an addressing mode.
#[derive(Clone, Copy, Debug)]
pub enum Operand {
    /// Operand lives in a bus address.
    Address { address: u16, page_crossed: bool },
    /// Operand is the accumulator register itself.
    Accumulator,
}

impl Operand {
    /// Panics if called on Operand::Accumulator. Only valid for instructions whose opcodes never
    /// use accumulator addressing.
    pub fn expect_address(&self) -> (u16, bool) {
        match self {
            Operand::Address {
                address,
                page_crossed,
            } => (*address, *page_crossed),
            Operand::Accumulator => {
                panic!("this instruction does not support accumulator addressing")
            }
        }
    }
}

impl Cpu {
    pub fn resolve_address(&mut self, mode: AddressingMode, bus: &mut CpuBus) -> Operand {
        match mode {
            AddressingMode::Accumulator => self.addr_accumulator(),
            AddressingMode::Implied => self.addr_implied(),
            AddressingMode::Immediate => self.addr_immediate(),
            AddressingMode::ZeroPage => self.addr_zero_page(bus),
            AddressingMode::ZeroPageX => self.addr_zero_page_x(bus),
            AddressingMode::ZeroPageY => self.addr_zero_page_y(bus),
            AddressingMode::Relative => self.addr_relative(bus),
            AddressingMode::Absolute => self.addr_absolute(bus),
            AddressingMode::AbsoluteX => self.addr_absolute_x(bus),
            AddressingMode::AbsoluteY => self.addr_absolute_y(bus),
            AddressingMode::Indirect => self.addr_indirect(bus),
            AddressingMode::IndirectX => self.addr_indirect_x(bus),
            AddressingMode::IndirectY => self.addr_indirect_y(bus),
        }
    }

    fn addr_accumulator(&mut self) -> Operand {
        Operand::Accumulator
    }

    fn addr_implied(&mut self) -> Operand {
        Operand::Address {
            address: 0,
            page_crossed: false,
        }
    }

    fn addr_immediate(&mut self) -> Operand {
        let address = self.pc;
        self.pc = self.pc.wrapping_add(1);
        Operand::Address {
            address,
            page_crossed: false,
        }
    }

    fn addr_zero_page(&mut self, bus: &mut CpuBus) -> Operand {
        let address = bus.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        Operand::Address {
            address,
            page_crossed: false,
        }
    }

    fn addr_zero_page_x(&mut self, bus: &mut CpuBus) -> Operand {
        let base = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        let address = base.wrapping_add(self.x) as u16;
        Operand::Address {
            address,
            page_crossed: false,
        }
    }

    fn addr_zero_page_y(&mut self, bus: &mut CpuBus) -> Operand {
        let base = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        let address = base.wrapping_add(self.y) as u16;
        Operand::Address {
            address,
            page_crossed: false,
        }
    }

    fn addr_relative(&mut self, bus: &mut CpuBus) -> Operand {
        let offset = bus.read(self.pc) as i8;
        self.pc = self.pc.wrapping_add(1);

        // `pc` now points past the branch instruction completely.
        let base = self.pc;
        let address = base.wrapping_add(offset as i16 as u16);

        Operand::Address {
            address,
            page_crossed: page_crossed(base, address),
        }
    }

    fn addr_absolute(&mut self, bus: &mut CpuBus) -> Operand {
        let address = u16::from_le_bytes([bus.read(self.pc), bus.read(self.pc.wrapping_add(1))]);
        self.pc = self.pc.wrapping_add(2);
        Operand::Address {
            address,
            page_crossed: false,
        }
    }

    fn addr_absolute_x(&mut self, bus: &mut CpuBus) -> Operand {
        let base = u16::from_le_bytes([bus.read(self.pc), bus.read(self.pc.wrapping_add(1))]);
        self.pc = self.pc.wrapping_add(2);
        let address = base.wrapping_add(self.x as u16);

        Operand::Address {
            address,
            page_crossed: page_crossed(base, address),
        }
    }

    fn addr_absolute_y(&mut self, bus: &mut CpuBus) -> Operand {
        let base = u16::from_le_bytes([bus.read(self.pc), bus.read(self.pc.wrapping_add(1))]);
        self.pc = self.pc.wrapping_add(2);
        let address = base.wrapping_add(self.y as u16);

        Operand::Address {
            address,
            page_crossed: page_crossed(base, address),
        }
    }

    fn addr_indirect(&mut self, bus: &mut CpuBus) -> Operand {
        let pointer = u16::from_le_bytes([bus.read(self.pc), bus.read(self.pc.wrapping_add(1))]);
        self.pc = self.pc.wrapping_add(2);

        let lo = bus.read(pointer);

        // Hardware bug: if the pointer's low byte is 0xFF, then the high byte
        // of the real address is NOT read from pointer + 1 (which would
        // cross the page). Instead, it wraps back to the start of the same page.
        let hi_addr = if pointer & 0x00FF == 0x00FF {
            pointer & 0xFF00 // Same page, low byte forced to 0x00
        } else {
            pointer.wrapping_add(1)
        };
        let hi = bus.read(hi_addr);

        let address = u16::from_le_bytes([lo, hi]);

        Operand::Address {
            address,
            page_crossed: false,
        }
    }

    fn addr_indirect_x(&mut self, bus: &mut CpuBus) -> Operand {
        let zp_base = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        let pointer = zp_base.wrapping_add(self.x) as u16;
        let lo = bus.read(pointer);

        let hi_addr = zp_base.wrapping_add(self.x).wrapping_add(1) as u16;
        let hi = bus.read(hi_addr);

        let address = u16::from_le_bytes([lo, hi]);
        Operand::Address {
            address,
            page_crossed: false,
        }
    }

    fn addr_indirect_y(&mut self, bus: &mut CpuBus) -> Operand {
        let zp_base = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        let lo = bus.read(zp_base as u16);
        let hi = bus.read(zp_base.wrapping_add(1) as u16);
        let base = u16::from_le_bytes([lo, hi]);

        let address = base.wrapping_add(self.y as u16);
        Operand::Address {
            address,
            page_crossed: page_crossed(base, address),
        }
    }
}

#[inline]
fn page_crossed(base: u16, address: u16) -> bool {
    (address & 0xFF00) != (base & 0xFF00)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implied_returns_placeholder_without_advancing_pc() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        let operand = cpu.addr_implied();
        let (_, page_crossed) = operand.expect_address();
        assert_eq!(cpu.pc, 0x1000);
        assert!(!page_crossed);
    }

    #[test]
    fn immediate_reads_next_byte_address_and_advances_pc() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        let operand = cpu.addr_immediate();
        let (address, page_crossed) = operand.expect_address();
        assert_eq!(address, 0x1000);
        assert_eq!(cpu.pc, 0x1001);
        assert!(!page_crossed);
    }

    #[test]
    fn zero_page_reads_low_byte_and_advances_pc() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        bus.write(0x1000, 0x42);

        let operand = cpu.addr_zero_page(&mut bus);
        let (address, page_crossed) = operand.expect_address();

        assert_eq!(address, 0x0042);
        assert_eq!(cpu.pc, 0x1001);
        assert!(!page_crossed);
    }

    #[test]
    fn zero_page_x_wraps_within_page_zero() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        cpu.x = 0x02;
        bus.write(0x1000, 0xFF);

        let operand = cpu.addr_zero_page_x(&mut bus);
        let (address, _) = operand.expect_address();

        assert_eq!(address, 0x0001, "must wrap within page 0x00");
    }

    #[test]
    fn zero_page_y_wraps_within_page_zero() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        cpu.y = 0x10;
        bus.write(0x1000, 0xFF);

        let operand = cpu.addr_zero_page_y(&mut bus);
        let (address, _) = operand.expect_address();

        assert_eq!(address, 0x000F, "must wrap within page 0x00");
    }

    #[test]
    fn relative_positive_offset() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        bus.write(0x1000, 0x10); // +16

        let operand = cpu.addr_relative(&mut bus);
        let (address, page_crossed) = operand.expect_address();

        // pc after consuming the offset byte is 0x1001; target is 0x1001 + 16
        assert_eq!(address, 0x1011);
        assert!(!page_crossed);
    }

    #[test]
    fn relative_negative_offset() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        bus.write(0x1000, 0xFE); // -2 as i8

        let operand = cpu.addr_relative(&mut bus);
        let (address, page_crossed) = operand.expect_address();

        // pc after operand byte is 0x1001; target is 0x1001 - 2 = 0x0FFF
        assert_eq!(address, 0x0FFF);
        assert!(page_crossed, "0x1001 -> 0x0FFF crosses a page");
    }

    #[test]
    fn absolute_reads_little_endian_address() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        bus.write(0x1000, 0x00);
        bus.write(0x1001, 0x80);

        let operand = cpu.addr_absolute(&mut bus);
        let (address, page_crossed) = operand.expect_address();

        assert_eq!(address, 0x8000);
        assert_eq!(cpu.pc, 0x1002);
        assert!(!page_crossed);
    }

    #[test]
    fn absolute_x_no_page_cross() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        cpu.x = 0x05;
        bus.write(0x1000, 0x00);
        bus.write(0x1001, 0x20); // base = 0x2000

        let operand = cpu.addr_absolute_x(&mut bus);
        let (address, page_crossed) = operand.expect_address();

        assert_eq!(address, 0x2005);
        assert!(!page_crossed);
    }

    #[test]
    fn absolute_x_page_cross() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        cpu.x = 0x20;
        bus.write(0x1000, 0xF0);
        bus.write(0x1001, 0x20); // base = 0x20F0

        let operand = cpu.addr_absolute_x(&mut bus);
        let (address, page_crossed) = operand.expect_address();

        assert_eq!(address, 0x2110);
        assert!(page_crossed, "0x20F0 + 0x20 crosses into page 0x21");
    }

    #[test]
    fn absolute_y_page_cross() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        cpu.y = 0xFF;
        bus.write(0x1000, 0x01);
        bus.write(0x1001, 0x20); // base = 0x2001

        let operand = cpu.addr_absolute_y(&mut bus);
        let (address, page_crossed) = operand.expect_address();

        assert_eq!(address, 0x2100);
        assert!(page_crossed);
    }

    #[test]
    fn indirect_normal_case() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        bus.write(0x1000, 0x00);
        bus.write(0x1001, 0x02); // pointer = 0x0200
        bus.write(0x0200, 0x34);
        bus.write(0x0201, 0x12); // real address = 0x1234

        let operand = cpu.addr_indirect(&mut bus);
        let (address, _) = operand.expect_address();

        assert_eq!(address, 0x1234);
    }

    #[test]
    fn indirect_page_wrap_bug() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        bus.write(0x1000, 0xFF);
        bus.write(0x1001, 0x02); // pointer = 0x02FF, triggers the bug

        bus.write(0x02FF, 0x34); // low byte, read normally
        bus.write(0x0200, 0x12); // high byte, buggy source (wraps to page start)
        bus.write(0x0300, 0x99); // what a "correct" implementation would wrongly read

        let operand = cpu.addr_indirect(&mut bus);
        let (address, _) = operand.expect_address();

        assert_eq!(
            address, 0x1234,
            "high byte must come from 0x0200, not 0x0300"
        );
    }

    #[test]
    fn indirect_x_uses_index_before_pointer_lookup() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        cpu.x = 0x04;
        bus.write(0x1000, 0x20); // zp_base
        // pointer = zp_base + x = 0x24
        bus.write(0x0024, 0x00);
        bus.write(0x0025, 0x03); // real address = 0x0300

        let operand = cpu.addr_indirect_x(&mut bus);
        let (address, _) = operand.expect_address();

        assert_eq!(address, 0x0300);
    }

    #[test]
    fn indirect_x_wraps_pointer_within_zero_page() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        cpu.x = 0x10;
        bus.write(0x1000, 0xF8); // zp_base + x = 0x108, wraps to 0x08
        bus.write(0x0008, 0x00);
        bus.write(0x0009, 0x04); // real address = 0x0400

        let operand = cpu.addr_indirect_x(&mut bus);
        let (address, _) = operand.expect_address();

        assert_eq!(address, 0x0400);
    }

    #[test]
    fn indirect_y_adds_index_after_pointer_lookup() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        cpu.y = 0x10;
        bus.write(0x1000, 0x20); // zp_base, no index applied here
        bus.write(0x0020, 0x00);
        bus.write(0x0021, 0x03); // base address = 0x0300

        let operand = cpu.addr_indirect_y(&mut bus);
        let (address, page_crossed) = operand.expect_address();

        assert_eq!(address, 0x0310); // 0x0300 + y
        assert!(!page_crossed);
    }

    #[test]
    fn indirect_y_page_cross() {
        let mut bus = CpuBus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        cpu.y = 0xFF;
        bus.write(0x1000, 0x20);
        bus.write(0x0020, 0x01);
        bus.write(0x0021, 0x03); // base address = 0x0301

        let operand = cpu.addr_indirect_y(&mut bus);
        let (address, page_crossed) = operand.expect_address();

        assert_eq!(address, 0x0400);
        assert!(page_crossed);
    }

    #[test]
    fn page_crossed_helper_detects_difference() {
        assert!(page_crossed(0x20F0, 0x2110));
        assert!(!page_crossed(0x2000, 0x2010));
    }
}
