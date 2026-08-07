//! Mapper 004 (Nintendo MMC3).
//!
//!The Nintendo MMC3 is a mapper ASIC used in Nintendo's TxROM Game Pak boards. Most common TxROM
//!boards, along with the NES-HKROM board (which uses the Nintendo MMC6), are assigned to iNES
//!Mapper 004.
//!
//! For more information, see the [nesdev.org MMC3 wiki page](https://www.nesdev.org/wiki/MMC3).

use crate::cartridge::{Mirroring, mapper::Mapper};

/// Size of a switchable 8 KiB PRG-ROM bank.
const PRG_BANK_SIZE: usize = 8 * 1024;

/// Size of a switchable 1 KiB CHR-ROM bank.
const CHR_BANK_SIZE: usize = 1024;

/// Internal PRG-RAM size.
const PRG_RAM_SIZE: usize = 8 * 1024;

pub struct Mapper004 {
    /// PRG-ROM image.
    prg_rom: Vec<u8>,
    /// CHR-ROM image. Writable when CHR-RAM is used.
    chr_rom: Vec<u8>,
    /// Cartridge PRG-RAM.
    prg_ram: Vec<u8>,

    /// Register (0-7) targeted by the next write to an odd $8001-$9FFF address.
    bank_select: u8,
    /// The eight bank registers R0-R7 programmed via $8001/$9001/etc.
    bank_regs: [u8; 8],
    /// PRG-ROM bank mode: false = $8000 swappable / $C000 fixed, true = swapped.
    prg_mode_fix_first: bool,
    /// CHR-ROM A12 inversion, swaps the two 1 KiB/2 KiB CHR halves.
    chr_inversion: bool,

    /// Mirroring, controlled by the mapper itself (via $A000) rather than the header.
    mirroring: Mirroring,
    /// PRG-RAM enable, from $A001 bit 7.
    prg_ram_enabled: bool,
    /// PRG-RAM write protect, from $A001 bit 6.
    prg_ram_write_protect: bool,

    /// Reload value for the IRQ counter, set via even $C000-$DFFE writes.
    irq_latch: u8,
    /// Current IRQ scanline counter.
    irq_counter: u8,
    /// Set by a write to odd $C001-$DFFF; forces a reload on the next A12 clock.
    irq_reload: bool,
    /// Whether the counter reaching zero should assert the IRQ line.
    irq_enabled: bool,
    /// Latched IRQ line, polled by the CPU via `Mapper::irq_pending`.
    irq_pending: bool,
    /// Last-observed state of PPU address bit 12, used to detect the rising edge that clocks
    /// the counter.
    last_a12: bool,
}

impl Mapper004 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        Self {
            prg_rom,
            chr_rom,
            prg_ram: vec![0; PRG_RAM_SIZE],
            bank_select: 0,
            bank_regs: [0; 8],
            prg_mode_fix_first: false,
            chr_inversion: false,
            mirroring,
            prg_ram_enabled: true,
            prg_ram_write_protect: false,
            irq_latch: 0,
            irq_counter: 0,
            irq_reload: false,
            irq_enabled: false,
            irq_pending: false,
            last_a12: false,
        }
    }

    fn prg_bank_count(&self) -> usize {
        (self.prg_rom.len() / PRG_BANK_SIZE).max(1)
    }

    fn chr_bank_count(&self) -> usize {
        (self.chr_rom.len() / CHR_BANK_SIZE).max(1)
    }

    /// Resolve which 8 KiB PRG bank is mapped into a given $8000-$FFFF quadrant (0-3).
    fn prg_bank_for_quadrant(&self, quadrant: usize) -> usize {
        let bank_count = self.prg_bank_count();
        let second_last = bank_count.saturating_sub(2);
        let last = bank_count.saturating_sub(1);

        let bank = match quadrant {
            // $8000-$9FFF: swappable (R6), unless mode fixes it to second-to-last.
            0 if self.prg_mode_fix_first => second_last,
            0 => (self.bank_regs[6] & 0x3F) as usize,
            // $A000-$BFFF: always swappable (R7).
            1 => (self.bank_regs[7] & 0x3F) as usize,
            // $C000-$DFFF: fixed to second-to-last, unless mode swaps it in (R6).
            2 if self.prg_mode_fix_first => (self.bank_regs[6] & 0x3F) as usize,
            2 => second_last,
            // $E000-$FFFF: always fixed to the last bank.
            3 => last,
            _ => unreachable!("quadrant is masked to 2 bits"),
        };

        bank % bank_count
    }

    fn prg_offset(&self, address: u16) -> usize {
        let quadrant = ((address - 0x8000) as usize) / PRG_BANK_SIZE;
        let window_offset = (address as usize - 0x8000) % PRG_BANK_SIZE;
        self.prg_bank_for_quadrant(quadrant) * PRG_BANK_SIZE + window_offset
    }

    /// Resolve which 1 KiB CHR bank is mapped into a given $0000-$1FFF slot (0-7).
    fn chr_bank_for_slot(&self, slot: usize) -> usize {
        let slot = if self.chr_inversion {
            slot ^ 0b100
        } else {
            slot
        };

        let bank = match slot {
            0 => (self.bank_regs[0] & 0xFE) as usize,
            1 => (self.bank_regs[0] | 0x01) as usize,
            2 => (self.bank_regs[1] & 0xFE) as usize,
            3 => (self.bank_regs[1] | 0x01) as usize,
            4 => self.bank_regs[2] as usize,
            5 => self.bank_regs[3] as usize,
            6 => self.bank_regs[4] as usize,
            7 => self.bank_regs[5] as usize,
            _ => unreachable!("slot is masked to 3 bits"),
        };

        bank % self.chr_bank_count()
    }

    fn chr_offset(&self, address: u16) -> usize {
        let slot = (address as usize) / CHR_BANK_SIZE;
        let window_offset = (address as usize) % CHR_BANK_SIZE;
        self.chr_bank_for_slot(slot) * CHR_BANK_SIZE + window_offset
    }

    /// Watch PPU address bit 12 for the rising edge that clocks the scanline counter (this
    /// fires roughly once per scanline while rendering, since the PPU's background/sprite
    /// pattern-table fetches toggle A12).
    fn watch_a12(&mut self, address: u16) {
        let a12 = address & 0x1000 != 0;
        if a12 && !self.last_a12 {
            self.clock_irq_counter();
        }
        self.last_a12 = a12;
    }

    fn clock_irq_counter(&mut self) {
        if self.irq_counter == 0 || self.irq_reload {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter -= 1;
        }

        if self.irq_counter == 0 && self.irq_enabled {
            self.irq_pending = true;
        }
    }
}

impl Mapper for Mapper004 {
    fn cpu_read(&mut self, address: u16) -> Option<u8> {
        match address {
            0x6000..=0x7FFF if self.prg_ram_enabled => {
                self.prg_ram.get((address - 0x6000) as usize).copied()
            }
            0x8000..=0xFFFF if !self.prg_rom.is_empty() => {
                let idx = self.prg_offset(address) % self.prg_rom.len();
                Some(self.prg_rom[idx])
            }
            _ => None,
        }
    }

    fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            0x6000..=0x7FFF => {
                if self.prg_ram_enabled && !self.prg_ram_write_protect {
                    if let Some(byte) = self.prg_ram.get_mut((address - 0x6000) as usize) {
                        *byte = data;
                    }
                }
            }
            // Bank select ($8000-$9FFE, even).
            0x8000..=0x9FFF if address.is_multiple_of(2) => {
                self.bank_select = data & 0x07;
                self.prg_mode_fix_first = data & 0x40 != 0;
                self.chr_inversion = data & 0x80 != 0;
            }
            // Bank data ($8001-$9FFF, odd).
            0x8000..=0x9FFF => {
                self.bank_regs[self.bank_select as usize] = data;
            }
            // Mirroring ($A000-$BFFE, even).
            0xA000..=0xBFFF if address.is_multiple_of(2) => {
                self.mirroring = if data & 0x01 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
            // PRG-RAM protect ($A001-$BFFF, odd).
            0xA000..=0xBFFF => {
                self.prg_ram_write_protect = data & 0x40 != 0;
                self.prg_ram_enabled = data & 0x80 != 0;
            }
            // IRQ latch ($C000-$DFFE, even).
            0xC000..=0xDFFF if address.is_multiple_of(2) => {
                self.irq_latch = data;
            }
            // IRQ reload ($C001-$DFFF, odd).
            0xC000..=0xDFFF => {
                self.irq_reload = true;
            }
            // IRQ disable + acknowledge ($E000-$FFFE, even).
            0xE000..=0xFFFF if address.is_multiple_of(2) => {
                self.irq_enabled = false;
                self.irq_pending = false;
            }
            // IRQ enable ($E001-$FFFF, odd).
            0xE000..=0xFFFF => {
                self.irq_enabled = true;
            }
            _ => {}
        }
    }

    fn ppu_read(&mut self, address: u16) -> Option<u8> {
        if (0x0000..=0x1FFF).contains(&address) {
            self.watch_a12(address);
        }

        match address {
            0x0000..=0x1FFF if !self.chr_rom.is_empty() => {
                let idx = self.chr_offset(address) % self.chr_rom.len();
                Some(self.chr_rom[idx])
            }
            _ => None,
        }
    }

    fn ppu_write(&mut self, address: u16, data: u8) {
        if (0x0000..=0x1FFF).contains(&address) {
            self.watch_a12(address);
        }

        if (0x0000..=0x1FFF).contains(&address) && !self.chr_rom.is_empty() {
            let idx = self.chr_offset(address) % self.chr_rom.len();
            self.chr_rom[idx] = data;
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    fn irq_clear(&mut self) {
        self.irq_pending = false;
    }
}
