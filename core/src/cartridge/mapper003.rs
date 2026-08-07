//! Mapper 003 (CNROM).
//!
//! CNROM provides simple CHR-ROM bank switching while PRG-ROM is fixed
//! (similar to Mapper 000 / NROM). Writes to $8000-$FFFF select an 8 KiB
//! CHR-ROM bank.
//!
//! For more information, see the [nesdev.org CNROM wiki page](https://www.nesdev.org/wiki/CNROM).

use crate::cartridge::{Mirroring, mapper::Mapper};

/// Size of an 8 KiB CHR-ROM bank.
const CHR_BANK_SIZE: usize = 8 * 1024;

pub struct Mapper003 {
    /// PRG-ROM image.
    prg_rom: Vec<u8>,

    /// CHR-ROM image. Writable when CHR-RAM is used.
    chr_rom: Vec<u8>,

    /// Mirroring mode from the cartridge header.
    mirroring: Mirroring,

    /// Currently selected CHR bank mapped at $0000-$1FFF.
    chr_bank: u8,
}

impl Mapper003 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        Self {
            prg_rom,
            chr_rom,
            mirroring,
            chr_bank: 0,
        }
    }

    /// Converts a PPU pattern-table address into an offset within the CHR image.
    fn chr_offset(&self, address: u16) -> usize {
        let bank_count = (self.chr_rom.len() / CHR_BANK_SIZE).max(1);
        let offset = (address & 0x1FFF) as usize;
        let bank = (self.chr_bank as usize) % bank_count;

        bank * CHR_BANK_SIZE + offset
    }
}

impl Mapper for Mapper003 {
    fn cpu_read(&mut self, address: u16) -> Option<u8> {
        match address {
            0x8000..=0xFFFF if !self.prg_rom.is_empty() => {
                let idx = (address - 0x8000) as usize % self.prg_rom.len();
                Some(self.prg_rom[idx])
            }
            _ => None,
        }
    }

    fn cpu_write(&mut self, address: u16, data: u8) {
        if (0x8000..=0xFFFF).contains(&address) {
            self.chr_bank = data;
        }
    }

    fn ppu_read(&mut self, address: u16) -> Option<u8> {
        match address {
            0x0000..=0x1FFF if !self.chr_rom.is_empty() => {
                let idx = self.chr_offset(address);
                self.chr_rom.get(idx).copied()
            }
            _ => None,
        }
    }

    fn ppu_write(&mut self, address: u16, data: u8) {
        if (0x0000..=0x1FFF).contains(&address) && !self.chr_rom.is_empty() {
            let idx = self.chr_offset(address);
            if let Some(byte) = self.chr_rom.get_mut(idx) {
                *byte = data;
            }
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
