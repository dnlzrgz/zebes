use crate::{bus::Bus, cpu::Cpu};

/// The 6502's addressing modes: each describes a way of computing
/// the effective address (or lack thereof) that an instruction operates on.
pub enum AddressingMode {
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
pub struct Operand {
    pub address: u16,
    pub page_crossed: bool,
}

impl Cpu {
    pub fn resolve_address(&mut self, mode: AddressingMode, bus: &mut Bus) -> Operand {
        match mode {
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

    fn addr_implied(&mut self) -> Operand {
        todo!()
    }

    fn addr_immediate(&mut self) -> Operand {
        todo!()
    }

    fn addr_zero_page(&mut self, bus: &mut Bus) -> Operand {
        todo!()
    }

    fn addr_zero_page_x(&mut self, bus: &mut Bus) -> Operand {
        todo!()
    }

    fn addr_zero_page_y(&mut self, bus: &mut Bus) -> Operand {
        todo!()
    }

    fn addr_relative(&mut self, bus: &mut Bus) -> Operand {
        todo!()
    }

    fn addr_absolute(&mut self, bus: &mut Bus) -> Operand {
        todo!()
    }

    fn addr_absolute_x(&mut self, bus: &mut Bus) -> Operand {
        todo!()
    }

    fn addr_absolute_y(&mut self, bus: &mut Bus) -> Operand {
        todo!()
    }

    fn addr_indirect(&mut self, bus: &mut Bus) -> Operand {
        todo!()
    }

    fn addr_indirect_x(&mut self, bus: &mut Bus) -> Operand {
        todo!()
    }

    fn addr_indirect_y(&mut self, bus: &mut Bus) -> Operand {
        todo!()
    }
}
