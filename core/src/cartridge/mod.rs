mod mapper;
mod mapper000;

pub use mapper::Mapper;
use mapper000::Mapper000;
use std::{cell::RefCell, rc::Rc};

/// Every valid iNES file should start with this exact 4-byte signature.
const INES_MAGIC: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

const HEADER_SIZE: usize = 16;

/// Some old ROMs include this legacy 512-byte block.
const TRAINER_SIZE: usize = 512;

/// PRG-ROM and CHR-ROM sizes are given in the header as a bank count instead of raw bytes.
const PRG_BANK_SIZE: usize = 16 * 1024;
const CHR_BANK_SIZE: usize = 8 * 1024;

/// Represents a loaded NES cartridge.
pub struct Cartridge {
    mapper: Box<dyn Mapper>,
}

impl Default for Cartridge {
    fn default() -> Self {
        Self {
            mapper: Box::new(Mapper000::new(Vec::new(), Vec::new())),
        }
    }
}

impl Cartridge {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses a raw byte buffer from an iNES file into a `Cartridge`.
    pub fn try_from_ines(data: &[u8]) -> Result<Self, String> {
        if data.len() < HEADER_SIZE || data[0..4] != INES_MAGIC {
            return Err("invalid iNES file".to_string());
        }

        let prg_banks = data[4] as usize;
        let chr_banks = data[5] as usize;
        let flags6 = data[6];
        let flags7 = data[7];
        let has_trainer_data = flags6 & 0b0000_0100 != 0;
        let mapper_id = (flags7 & 0xF0) | (flags6 >> 4);

        let mut offset = HEADER_SIZE;
        if has_trainer_data {
            offset += TRAINER_SIZE;
        }

        let prg_size = prg_banks * PRG_BANK_SIZE;
        let prg_rom = data[offset..offset + prg_size].to_vec();
        offset += prg_size;

        let chr_size = chr_banks * CHR_BANK_SIZE;
        let chr_rom = data[offset..offset + chr_size].to_vec();

        let mapper = match mapper_id {
            0 => Box::new(Mapper000::new(prg_rom, chr_rom)),
            _ => return Err(format!("mapper {mapper_id} not supported yet")),
        };

        Ok(Self { mapper })
    }

    pub fn cpu_read(&self, address: u16) -> Option<u8> {
        self.mapper.cpu_read(address)
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        self.mapper.cpu_write(address, data);
    }

    pub fn ppu_read(&self, address: u16) -> Option<u8> {
        self.mapper.ppu_read(address)
    }

    pub fn ppu_write(&mut self, address: u16, data: u8) {
        self.mapper.ppu_write(address, data);
    }
}

/// Shared cartridge handle for the CPU's and PPU's bus.
pub type SharedCartridge = Rc<RefCell<Cartridge>>;
