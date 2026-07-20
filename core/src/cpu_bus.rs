use crate::cartridge::Cartridge;

pub struct CpuBus {
    ram: [u8; 0x0800],
    cartridge: Cartridge,
    ppu: [u8; 8],
}

impl Default for CpuBus {
    fn default() -> Self {
        Self {
            ram: [0; 0x0800],
            cartridge: Cartridge::new(),
            ppu: [0; 8],
        }
    }
}

impl CpuBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cartridge(cartridge: Cartridge) -> Self {
        Self {
            ram: [0; 0x0800],
            cartridge,
            ppu: [0; 8],
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram[(address & 0x07FF) as usize], // RAM
            0x2000..=0x3FFF => self.ppu[(address & 0x0007) as usize], // PPU
            0x4000..=0x4017 => 0x00,                                  // APU + I/O
            0x4018..=0x401F => 0x00,                                  // APU + I/O (test mode)
            0x4020..=0x5FFF => 0x00,                                  // Cartridge expansion
            0x6000..=0x7FFF => 0x00,                                  // Cartridge SRAM/PGR-RAM
            0x8000..=0xFFFF => self.cartridge.cpu_read(address).unwrap_or(0x00), // PRG-ROM
        }
    }

    pub fn peek(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram[(address & 0x07FF) as usize], // RAM
            0x2000..=0x3FFF => self.ppu[(address & 0x0007) as usize], // PPU
            0x4000..=0x4017 => 0x00,                                  // APU + I/O
            0x4018..=0x401F => 0x00,                                  // APU + I/O (test mode)
            0x4020..=0x5FFF => 0x00,                                  // Cartridge expansion
            0x6000..=0x7FFF => 0x00,                                  // Cartridge SRAM/PGR-RAM
            0x8000..=0xFFFF => self.cartridge.cpu_read(address).unwrap_or(0x00), // PRG-ROM
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => self.ram[(address & 0x07FF) as usize] = data, // RAM
            0x2000..=0x3FFF => self.ppu[(address & 0x0007) as usize] = data, // PPU
            0x4000..=0x4017 => {}                                            // APU + I/O
            0x4018..=0x401F => {} // APU + I/O (test mode)
            0x4020..=0x5FFF => {} // Cartridge expansion
            0x6000..=0x7FFF => {} // Cartridge SRAM/PGR-RAM
            0x8000..=0xFFFF => self.cartridge.cpu_write(address, data), // PRG-ROM
        }
    }
}
