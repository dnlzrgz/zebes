use crate::cartridge::mapper::Mapper;

/// Also known as NROM is the simplest cartridge wiring, typically used by early and small NES
/// games. It has no bank-switching hardware at all.
pub struct Mapper000 {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
}

impl Mapper000 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Self { prg_rom, chr_rom }
    }
}

impl Mapper for Mapper000 {
    fn cpu_read(&self, address: u16) -> Option<u8> {
        match address {
            0x8000..=0xFFFF => {
                let idx = (address - 0x8000) as usize % self.prg_rom.len();
                Some(self.prg_rom[idx])
            }
            _ => None,
        }
    }

    fn cpu_write(&mut self, _: u16, _: u8) {}

    fn ppu_read(&self, address: u16) -> Option<u8> {
        match address {
            0x0000..=0x1FFF => self.chr_rom.get(address as usize).copied(),
            _ => None,
        }
    }

    fn ppu_write(&mut self, _: u16, _: u8) {}
}
