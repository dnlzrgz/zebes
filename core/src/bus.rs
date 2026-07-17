const RAM_SIZE: usize = 0x2000; // 8kb, covers 0x0000..=0x1FFF

pub struct Bus {
    ram: [u8; RAM_SIZE],
}

impl Default for Bus {
    fn default() -> Self {
        Self { ram: [0; RAM_SIZE] }
    }
}

impl Bus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&mut self, address: u16) -> u8 {
        self.ram[(address & 0x07FF) as usize]
    }

    pub fn peek(&self, address: u16) -> u8 {
        self.ram[(address & 0x07FF) as usize]
    }

    pub fn write(&mut self, address: u16, data: u8) {
        self.ram[(address & 0x07FF) as usize] = data;
    }

    /// Temporary helper for loading bytes into memory for debugging purposes.
    pub fn load_bytes(&mut self, data: &[u8], start: u16) {
        for (i, &byte) in data.iter().enumerate() {
            self.write(start.wrapping_add(i as u16), byte);
        }
    }
}
