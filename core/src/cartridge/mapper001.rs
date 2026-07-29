//! Mapper 001 (Nintendo NMC1).
//!
//! MMC1 provides a serially-programmed bank switching for PRG-ROM and CHR-ROM, selectable nametable
//! mirroring, and an optional PRG-RAM.
//!
//! For more information, see the [nesdev.org MMC1 wiki page](https://www.nesdev.org/wiki/MMC1).

use crate::cartridge::{Mirroring, mapper::Mapper};

/// Size of the switchable PRG-ROM bank.
const PRG_BANK_SIZE: usize = 16 * 1024;

/// Size of the switchable CHR-ROM bank.
const CHR_BANK_SIZE: usize = 4 * 1024;

/// Internal PRG-RAM size.
const PRG_RAM_SIZE: usize = 8 * 1024;

/// PRG-ROM banking modes selected by the control register.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PrgBankMode {
    /// Switch a single 32 KiB PRG window.
    Switch32Kb,
    /// Fix the first 16 KiB bank at $8000 and switch the upper bank.
    FixFirst,
    /// Fix the last 16 KiB bank at $C000 and switch the lower bank.
    FixLast,
}

/// CHR-ROM banking modes selected by the control register.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ChrBankMode {
    /// Switch one 8 KiB CHR bank.
    Switch8Kb,
    /// Independently switch two 4 KiB CHR banks.
    Switch4Kb,
}

pub struct Mapper001 {
    /// PRG-ROM image.
    prg_rom: Vec<u8>,
    /// CHR-ROM image. Writable when CHR-RAM is used.
    chr_rom: Vec<u8>,
    /// Cartridge PRG-RAM.
    prg_ram: Vec<u8>,
    /// Serial shift registers used to program the MMC1 registers.
    shift_register: u8,
    /// Number of bits currently loaded in the shift register.
    shift_count: u8,
    /// NMC1 control register.
    control: u8,
    /// CHR bank register 0.
    chr_bank_0: u8,
    /// CHR bank register 1.
    chr_bank_1: u8,
    /// PRG bank register.
    prg_bank: u8,
}

impl Mapper001 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, _: Mirroring) -> Self {
        Self {
            prg_rom,
            chr_rom,
            prg_ram: vec![0; PRG_RAM_SIZE],
            shift_register: 0,
            shift_count: 0,
            control: 0b0_11_00,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
        }
    }

    /// Decode the PRG banking mode from the control register.
    fn prg_bank_mode(&self) -> PrgBankMode {
        match (self.control >> 2) & 0b11 {
            0 | 1 => PrgBankMode::Switch32Kb,
            2 => PrgBankMode::FixFirst,
            3 => PrgBankMode::FixLast,
            _ => unreachable!("masked to 2 bits"),
        }
    }

    /// Decode the CHR banking mode from the control register.
    fn chr_bank_mode(&self) -> ChrBankMode {
        if self.control & 0b1_00_00 != 0 {
            ChrBankMode::Switch4Kb
        } else {
            ChrBankMode::Switch8Kb
        }
    }

    /// Convert a CPU PRG-ROM address into an offset within the cartridge image, applying the
    /// current MMC1 banking mode.
    fn prg_offset(&self, address: u16) -> usize {
        let bank_count = (self.prg_rom.len() / PRG_BANK_SIZE).max(1);
        let window_offset = (address & 0x3FFF) as usize;

        let bank = match self.prg_bank_mode() {
            PrgBankMode::Switch32Kb => {
                let base = (self.prg_bank & 0b1110) as usize;
                base + usize::from(address >= 0xC000)
            }
            PrgBankMode::FixFirst => {
                if address < 0xC000 {
                    0
                } else {
                    (self.prg_bank & 0x0F) as usize
                }
            }
            PrgBankMode::FixLast => {
                if address < 0xC000 {
                    (self.prg_bank & 0x0F) as usize
                } else {
                    bank_count - 1
                }
            }
        };

        (bank % bank_count) * PRG_BANK_SIZE + window_offset
    }

    /// Convert a PPU pattern-table address into an offset within the CHR image applying the current
    /// CHR banking mode.
    fn chr_offset(&self, address: u16) -> usize {
        let bank_count = (self.chr_rom.len() / CHR_BANK_SIZE).max(1);
        let window_offset = (address & 0x0FFF) as usize;

        let bank = match self.chr_bank_mode() {
            ChrBankMode::Switch8Kb => {
                let base = (self.chr_bank_0 & 0b1_1110) as usize;
                base + usize::from(address >= 0x1000)
            }
            ChrBankMode::Switch4Kb => {
                if address < 0x1000 {
                    self.chr_bank_0 as usize
                } else {
                    self.chr_bank_1 as usize
                }
            }
        };

        (bank % bank_count) * CHR_BANK_SIZE + window_offset
    }
}

impl Mapper for Mapper001 {
    fn cpu_read(&self, address: u16) -> Option<u8> {
        match address {
            0x6000..=0x7FFF => self.prg_ram.get((address - 0x6000) as usize).copied(),
            0x8000..=0xFFFF => {
                let idx = self.prg_offset(address) % self.prg_rom.len().max(1);
                self.prg_rom.get(idx).copied()
            }
            _ => None,
        }
    }

    fn cpu_write(&mut self, address: u16, data: u8) {
        if let 0x6000..=0x7FFF = address {
            if let Some(byte) = self.prg_ram.get_mut((address - 0x6000) as usize) {
                *byte = data;
            }
            return;
        }

        if !(0x8000..=0xFFFF).contains(&address) {
            return;
        }

        // Bit 7 immediately resets the serial loader.
        if data & 0x80 != 0 {
            self.shift_register = 0;
            self.shift_count = 0;
            self.control |= 0b0_11_00;
            return;
        }

        self.shift_register = (self.shift_register >> 1) | ((data & 1) << 4);
        self.shift_count += 1;

        if self.shift_count == 5 {
            let value = self.shift_register;
            match (address >> 13) & 0b11 {
                0b00 => self.control = value,
                0b01 => self.chr_bank_0 = value,
                0b10 => self.chr_bank_1 = value,
                0b11 => self.prg_bank = value,
                _ => unreachable!("masked to 2 bits"),
            }

            self.shift_register = 0;
            self.shift_count = 0;
        }
    }

    fn ppu_read(&self, address: u16) -> Option<u8> {
        match address {
            0x0000..=0x1FFF if !self.chr_rom.is_empty() => {
                let idx = self.chr_offset(address) % self.chr_rom.len();
                Some(self.chr_rom[idx])
            }
            _ => None,
        }
    }

    fn ppu_write(&mut self, address: u16, data: u8) {
        if (0x0000..=0x1FFF).contains(&address) && !self.chr_rom.is_empty() {
            let idx = self.chr_offset(address) % self.chr_rom.len();
            self.chr_rom[idx] = data;
        }
    }

    fn mirroring(&self) -> Mirroring {
        match self.control & 0b11 {
            0b00 => Mirroring::SingleScreenLower,
            0b01 => Mirroring::SingleScreenUpper,
            0b10 => Mirroring::Vertical,
            0b11 => Mirroring::Horizontal,
            _ => unreachable!("masked to 2 bits"),
        }
    }
}
