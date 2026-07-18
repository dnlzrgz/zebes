pub struct CpuBus {
    ram: [u8; 0x1FFF],     // 2kb, covers 0x0000..=0x1FFF
    pgr_rom: [u8; 0x8000], // 32kb
    ppu: [u8; 8],
}

impl Default for CpuBus {
    fn default() -> Self {
        Self {
            ram: [0; 0x1FFF],
            pgr_rom: [0; 0x8000],
            ppu: [0; 8],
        }
    }
}

impl CpuBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram[(address & 0x07FF) as usize], // RAM
            0x2000..=0x3FFF => self.ppu[(address & 0x0007) as usize], // PPU
            0x4000..=0x4017 => todo!(),                               // APU + I/O
            0x4018..=0x401F => todo!(),                               // APU + I/O (test mode)
            0x4020..=0x5FFF => todo!(),                               // Cartridge expansion
            0x6000..=0x7FFF => todo!(),                               // Cartridge SRAM/PGR-RAM
            0x8000..=0xFFFF => self.pgr_rom[(address - 0x8000) as usize], // PRG-ROM
        }
    }

    pub fn peek(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram[(address & 0x07FF) as usize], // RAM
            0x2000..=0x3FFF => self.ppu[(address & 0x0007) as usize], // PPU
            0x4000..=0x4017 => todo!(),                               // APU + I/O
            0x4018..=0x401F => todo!(),                               // APU + I/O (test mode)
            0x4020..=0x5FFF => todo!(),                               // Cartridge expansion
            0x6000..=0x7FFF => todo!(),                               // Cartridge SRAM/PGR-RAM
            0x8000..=0xFFFF => self.pgr_rom[(address - 0x8000) as usize], // PRG-ROM
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => self.ram[(address & 0x07FF) as usize] = data, // RAM
            0x2000..=0x3FFF => self.ppu[(address & 0x0007) as usize] = data, // PPU
            0x4000..=0x4017 => todo!(),                                      // APU + I/O
            0x4018..=0x401F => todo!(), // APU + I/O (test mode)
            0x4020..=0x5FFF => todo!(), // Cartridge expansion
            0x6000..=0x7FFF => todo!(), // Cartridge SRAM/PGR-RAM
            0x8000..=0xFFFF => {}       // PRG-ROM
        }
    }

    /// Temporary helper for loading bytes into memory for debugging purposes.
    pub fn load_bytes(&mut self, data: &[u8], start: u16) {
        for (i, &byte) in data.iter().enumerate() {
            self.write(start.wrapping_add(i as u16), byte);
        }
    }
}
