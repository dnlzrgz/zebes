/// A "Mapper" implements the memory-mapping logic specific for a cartridge.
pub trait Mapper {
    /// Converts a CPU-visible address into a PRG-ROM/PRG-RAM byte.
    fn cpu_read(&self, address: u16) -> Option<u8>;

    /// Handles a CPU "write"" into the mapper's space. Usually the "data" written is not being
    /// written at all and is instead used to tell the mapper what do to next.
    fn cpu_write(&mut self, address: u16, data: u8);

    /// Same idea as the `cpu_read` but for the PPU's separate address space.
    fn ppu_read(&self, address: u16) -> Option<u8>;

    /// Handles PPU-side write into the mapper's space.
    fn ppu_write(&mut self, address: u16, data: u8);
}
