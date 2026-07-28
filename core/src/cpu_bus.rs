use crate::{cartridge::SharedCartridge, ppu::Ppu};

pub struct CpuBus {
    ram: [u8; 0x0800],
    cartridge: SharedCartridge,
    pub ppu: Ppu,
    pub controllers: [u8; 2],
    controller_shift: [u8; 2],
    controller_strobe: bool,
}

impl Default for CpuBus {
    fn default() -> Self {
        Self {
            ram: [0; 0x0800],
            cartridge: SharedCartridge::default(),
            ppu: Ppu::new(),
            controllers: [0; 2],
            controller_shift: [0; 2],
            controller_strobe: false,
        }
    }
}

impl CpuBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cartridge(mut self, cartridge: SharedCartridge) -> Self {
        self.cartridge = cartridge;
        self
    }

    pub fn with_ppu(mut self, ppu: Ppu) -> Self {
        self.ppu = ppu;
        self
    }

    pub fn set_controller_state(&mut self, controller: usize, state: u8) {
        self.controllers[controller] = state;
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram[(address & 0x07FF) as usize], // RAM
            0x2000..=0x3FFF => self.ppu.cpu_read(address),            // PPU
            0x4000..=0x4017 => match address {
                0x4016 => {
                    let bit = self.controller_shift[0] & 1;
                    if !self.controller_strobe {
                        self.controller_shift[0] >>= 1;
                        self.controller_shift[0] |= 0x80;
                    }
                    0x40 | bit
                }
                0x4017 => {
                    let bit = self.controller_shift[1] & 1;
                    if !self.controller_strobe {
                        self.controller_shift[1] >>= 1;
                        self.controller_shift[1] |= 0x80;
                    }

                    0x40 | bit
                }
                _ => 0x00,
            }, // APU + I/O
            0x4018..=0x401F => 0x00,                                  // APU + I/O (test mode)
            0x4020..=0x5FFF => 0x00,                                  // Cartridge expansion
            0x6000..=0x7FFF => 0x00,                                  // Cartridge SRAM/PGR-RAM
            0x8000..=0xFFFF => self.cartridge.borrow().cpu_read(address).unwrap_or(0x00), // PRG-ROM
        }
    }

    pub fn peek(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.ram[(address & 0x07FF) as usize], // RAM
            0x2000..=0x3FFF => self.ppu.cpu_peek(address),            // PPU
            0x4000..=0x4017 => 0x00,                                  // APU + I/O
            0x4018..=0x401F => 0x00,                                  // APU + I/O (test mode)
            0x4020..=0x5FFF => 0x00,                                  // Cartridge expansion
            0x6000..=0x7FFF => 0x00,                                  // Cartridge SRAM/PGR-RAM
            0x8000..=0xFFFF => self.cartridge.borrow().cpu_read(address).unwrap_or(0x00), // PRG-ROM
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => self.ram[(address & 0x07FF) as usize] = data, // RAM
            0x2000..=0x3FFF => self.ppu.cpu_write(address, data),            // PPU
            0x4000..=0x4017 => match address {
                0x4016 => {
                    self.controller_strobe = data & 1 == 1;
                    if self.controller_strobe {
                        self.controller_shift[0] = self.controllers[0];
                        self.controller_shift[1] = self.controllers[1];
                    }
                }
                _ => {}
            }, // APU + I/O
            0x4018..=0x401F => {} // APU + I/O (test mode)
            0x4020..=0x5FFF => {} // Cartridge expansion
            0x6000..=0x7FFF => {} // Cartridge SRAM/PGR-RAM
            0x8000..=0xFFFF => self.cartridge.borrow_mut().cpu_write(address, data), // PRG-ROM
        }
    }

    pub fn tick_ppu(&mut self) {
        self.ppu.clock();
    }

    pub fn take_nmi(&mut self) -> bool {
        self.ppu.take_nmi()
    }
}
