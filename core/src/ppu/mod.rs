mod flags;
mod ppu_bus;

/// Models the core of the Ricoh 2C02.
pub struct Ppu {}

impl Default for Ppu {
    fn default() -> Self {
        Self {}
    }
}

impl Ppu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        todo!()
    }

    pub fn cpu_peek(&self, address: u16) -> u8 {
        todo!()
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        todo!()
    }
}
