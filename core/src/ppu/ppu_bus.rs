use crate::cartridge::{Mirroring, SharedCartridge};

/// Represents the PPU's address space.
pub struct PpuBus {
    /// Nametable RAM.
    nametables: [u8; 0x0800],

    /// Palette RAM.
    palette: [u8; 32],

    /// Shared handle to the cartridge.
    cartridge: SharedCartridge,
}

impl Default for PpuBus {
    fn default() -> Self {
        Self {
            nametables: [0; 0x0800],
            palette: [0; 32],
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
            0x0000..=0x1FFF => self.cartridge.borrow_mut().ppu_read(address).unwrap_or(0),
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
        let physical_table = match self.cartridge.borrow().mirroring() {
            Mirroring::Horizontal => table / 2,
            Mirroring::Vertical => table % 2,
            Mirroring::SingleScreenLower => 0,
            Mirroring::SingleScreenUpper => 1,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ppu::test_utils::cartridge_with_chr_rom;

    #[test]
    fn read_nametable_roundtrips_with_horizontal_mirroring() {
        let mut bus = PpuBus::new();

        bus.write(0x2000, 0x11); // logical table 0 -> physical table 0
        bus.write(0x2800, 0x22); // logical table 2 -> physical table 1

        assert_eq!(bus.read(0x2000), 0x11);
        assert_eq!(bus.read(0x2400), 0x11); // aliases table 0
        assert_eq!(bus.read(0x2800), 0x22);
        assert_eq!(bus.read(0x2C00), 0x22); // aliases table 1
    }

    #[test]
    fn read_nametable_roundtrips_with_vertical_mirroring() {
        // Vertical mirroring folds logical tables 0/2 and 1/3 onto physical tables 0 and 1.
        let mut bus =
            PpuBus::new().with_cartridge(cartridge_with_chr_rom(&[], Mirroring::Vertical));

        bus.write(0x2000, 0x11); // table 0 -> physical 0
        bus.write(0x2400, 0x22); // table 1 -> physical 1

        assert_eq!(bus.read(0x2000), 0x11);
        assert_eq!(bus.read(0x2800), 0x11); // table 2 aliases table 0
        assert_eq!(bus.read(0x2400), 0x22);
        assert_eq!(bus.read(0x2C00), 0x22); // table 3 aliases table 1
    }

    #[test]
    fn palette_read_and_write_mirror_background_palette() {
        let mut bus = PpuBus::new();

        bus.write(0x3F00, 0xAB);

        // $3F10 and $3F20 are mirrors of $3F00 because addresses whose low 5 bits
        // are 0, 4, 8, or 12 alias the corresponding background palette entry.
        assert_eq!(bus.read(0x3F00), 0xAB);
        assert_eq!(bus.read(0x3F10), 0xAB);
        assert_eq!(bus.read(0x3F20), 0xAB);
    }

    #[test]
    fn nametable_mirroring_preserves_offset_within_table() {
        let mut bus = PpuBus::new();

        bus.write(0x23FF, 0x77); // table 0, offset $3FF

        assert_eq!(bus.read(0x23FF), 0x77);
        assert_eq!(bus.read(0x27FF), 0x77); // table 1 aliases table 0 at the same offset
    }

    #[test]
    fn palette_entries_are_independent_outside_mirrored_slots() {
        let mut bus = PpuBus::new();

        bus.write(0x3F01, 0x11);
        bus.write(0x3F11, 0x22);

        assert_eq!(bus.read(0x3F01), 0x11);
        assert_eq!(bus.read(0x3F11), 0x22); // sprite palette entries are not mirrors
    }

    #[test]
    fn read_cartridge_address_passes_through_to_chr_rom() {
        let bus =
            PpuBus::new().with_cartridge(cartridge_with_chr_rom(&[0x5A], Mirroring::Horizontal));

        assert_eq!(bus.read(0x0000), 0x5A);
        assert_eq!(bus.read(0x0001), 0x00);
    }
}
