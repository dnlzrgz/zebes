const RAM_SIZE: usize = 0x10000; // 64kb, covers 0x0000..=0xFFFF

pub struct Bus {
    ram: [u8; RAM_SIZE],
}

impl Default for Bus {
    fn default() -> Self {
        Bus { ram: [0; RAM_SIZE] }
    }
}

impl Bus {
    pub fn new() -> Self {
        Bus::default()
    }

    pub fn read(&self, addr: u16, _: bool) -> u8 {
        self.ram[addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }
}
