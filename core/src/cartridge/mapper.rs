/// A "Mapper" implements the memory-mapping logic specific for a cartridge.
pub trait Mapper {
    /// Convert a CPU-visible address into a PRG-ROM/PRG-RAM byte.
    fn cpu_read(&mut self, address: u16) -> Option<u8>;

    /// Handle a CPU "write"" into the mapper's space. Usually the "data" written is not being
    /// written at all and is instead used to tell the mapper what do to next.
    fn cpu_write(&mut self, address: u16, data: u8);

    /// Same idea as the `cpu_read` but for the PPU's separate address space.
    fn ppu_read(&mut self, address: u16) -> Option<u8>;

    /// Handle PPU-side write into the mapper's space.
    fn ppu_write(&mut self, address: u16, data: u8);

    /// Returns the mirroring configuration.
    fn mirroring(&self) -> Mirroring;

    /// Return whether the mapper has an IRQ request.
    fn irq_pending(&self) -> bool {
        false
    }

    /// Acknowledge a pending IRQ.
    fn irq_clear(&mut self) {}
}

/// Nametable mirroring configuration.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreenLower,
    SingleScreenUpper,
}
