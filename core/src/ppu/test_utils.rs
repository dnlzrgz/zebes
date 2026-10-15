use crate::{
    cartridge::{Cartridge, Mirroring, SharedCartridge},
    ppu::{Ppu, ppu_bus::PpuBus},
};
use std::{cell::RefCell, rc::Rc};

/// Shared cartridge whose CHR ROM is pre-populated with the given bytes.
#[cfg(test)]
pub fn cartridge_with_chr_rom(chr_rom: &[u8], mirroring: Mirroring) -> SharedCartridge {
    let mut data = vec![0u8; 16 + 16384 + 8192];
    data[0..4].copy_from_slice(b"NES\x1A");
    data[4] = 1; // one PRG bank
    data[5] = 1; // one CHR bank
    data[6] = if mirroring == Mirroring::Vertical {
        0x01
    } else {
        0x00
    };
    let chr_start = 16 + 16384;
    data[chr_start..chr_start + chr_rom.len()].copy_from_slice(chr_rom);

    let cartridge = Cartridge::try_from_ines(&data).unwrap();
    Rc::new(RefCell::new(cartridge))
}

/// Build a Ppu which CHR ROM is pre-populated with the given bytes.
#[cfg(test)]
pub fn ppu_with_chr_rom(chr_rom: &[u8], mirroring: Mirroring) -> Ppu {
    let bus = PpuBus::new().with_cartridge(cartridge_with_chr_rom(chr_rom, mirroring));
    Ppu::new().with_bus(bus)
}
