use crate::cartridge::SharedCartridge;

/// Nametable mirroring mode set by the cartridge.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
}

/// Represents the PPU's address space.
pub struct PpuBus {
    /// Nametable RAM.
    nametables: [u8; 0x0800],

    /// Palette RAM.
    palette: [u8; 32],

    /// Represents how the two extra logical nametables are wired.
    mirroring: Mirroring,

    /// Shared handle to the cartridge.
    cartridge: SharedCartridge,
}

impl Default for PpuBus {
    fn default() -> Self {
        Self {
            nametables: [0; 0x0800],
            palette: [0; 32],
            mirroring: Mirroring::Horizontal,
            cartridge: SharedCartridge::default(),
        }
    }
}

impl PpuBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cartridge(mut self, cartridge: SharedCartridge) -> Self {
        self.cartridge = cartridge;
        self
    }

    pub fn read(&self, address: u16) -> u8 {
        match address & 0x3FFF {
            0x0000..=0x1FFF => self.cartridge.borrow().ppu_read(address).unwrap_or(0),
            0x2000..=0x3EFF => self.nametables[self.nametable_index(address)],
            0x3F00..=0x3FFF => self.palette[Self::palette_index(address)],
            _ => unreachable!("address & 0x3FFF is always in 0..=0x3FFF"),
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address & 0x3FFF {
            0x0000..=0x1FFF => {
                self.cartridge.borrow_mut().ppu_write(address, data);
            }
            0x2000..=0x3EFF => {
                self.nametables[self.nametable_index(address)] = data;
            }
            0x3F00..=0x3FFF => {
                self.palette[Self::palette_index(address)] = data;
            }
            _ => unreachable!("address & 0x3FFF is always in 0..=0x3FFF"),
        }
    }

    /// Maps a $2000-$3EFF address down to a valid index into the 2KB of nametable RAM while
    /// applying the cartridge's mirroring mode.
    fn nametable_index(&self, address: u16) -> usize {
        let address = (address - 0x2000) % 0x1000;
        let table = address / 0x0400; // 4 logical nametables (0-3)
        let offset = (address % 0x0400) as usize;

        // TODO: check mirroring behavior.
        let physical_table = match self.mirroring {
            Mirroring::Horizontal => table / 2,
            Mirroring::Vertical => table % 2,
        };

        physical_table as usize * 0x0400 + offset
    }

    /// Maps a $3F00-$3FFF address down to a 0..32 index into palette RAM.
    fn palette_index(address: u16) -> usize {
        let mut index = (address - 0x3F00) % 32;
        if index.is_multiple_of(4) && index >= 16 {
            index -= 16;
        }

        index as usize
    }
}
