const RAM_SIZE: usize = 0x10000; // 64kb, covers 0x0000..=0xFFFF

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

    pub fn read(&mut self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    pub fn peek(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }

    /// Temporary helper for loading bytes into memory for debugging purposes.
    pub fn load_bytes(&mut self, data: &[u8], start: u16) {
        for (i, &byte) in data.iter().enumerate() {
            self.write(start.wrapping_add(i as u16), byte);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peek_and_read_return_same_value() {
        let mut bus = Bus::new();
        bus.write(0x0010, 0x42);
        assert_eq!(bus.read(0x0010), 0x42);
        assert_eq!(bus.peek(0x0010), 0x42);
    }
}
