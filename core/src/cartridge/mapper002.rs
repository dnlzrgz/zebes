//! Mapper 002 (UxROM).
//!
//! UxROM provides simple PRG-ROM bank switching. A single 16 KiB PRG bank is
//! mapped at $8000-$BFFF, while the last 16 KiB PRG bank remains permanently
//! mapped at $C000-$FFFF.
//!
//! CHR uses a fixed 8 KiB ROM (or RAM on some boards) with no bank switching.
//!
//! For more information, see the [nesdev.org UxROm wiki page](https://www.nesdev.org/wiki/UxROM).

use crate::cartridge::{Mirroring, mapper::Mapper};

/// Size of a switchable PRG-ROM bank.
const PRG_BANK_SIZE: usize = 16 * 1024;

pub struct Mapper002 {
    /// PRG-ROM image.
    prg_rom: Vec<u8>,

    /// CHR-ROM image. Writable when CHR-RAM is used.
    chr_rom: Vec<u8>,

    /// Mirroring mode from the cartridge header.
    mirroring: Mirroring,

    /// Currently selected PRG bank mapped at $8000-$BFFF.
    prg_bank: u8,
}

impl Mapper002 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        Self {
            prg_rom,
            chr_rom,
            mirroring,
            prg_bank: 0,
        }
    }

    /// Converts a CPU PRG-ROM address into an offset within the cartridge image.
    fn prg_offset(&self, address: u16) -> usize {
        let bank_count = (self.prg_rom.len() / PRG_BANK_SIZE).max(1);
        let offset = (address & 0x3FFF) as usize;

        let bank = if address < 0xC000 {
            self.prg_bank as usize
        } else {
            bank_count - 1
        };

        (bank % bank_count) * PRG_BANK_SIZE + offset
    }
}

impl Mapper for Mapper002 {
    fn cpu_read(&self, address: u16) -> Option<u8> {
        match address {
            0x8000..=0xFFFF => {
                let idx = self.prg_offset(address);
                self.prg_rom.get(idx).copied()
            }
            _ => None,
        }
    }

    fn cpu_write(&mut self, address: u16, data: u8) {
        if (0x8000..=0xFFFF).contains(&address) {
            self.prg_bank = data;
        }
    }

    fn ppu_read(&self, address: u16) -> Option<u8> {
        match address {
            0x0000..=0x1FFF if !self.chr_rom.is_empty() => {
                self.chr_rom.get(address as usize).copied()
            }
            _ => None,
        }
    }

    fn ppu_write(&mut self, address: u16, data: u8) {
        if (0x0000..=0x1FFF).contains(&address) && !self.chr_rom.is_empty() {
            self.chr_rom[address as usize] = data;
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
