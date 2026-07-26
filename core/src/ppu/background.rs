use crate::ppu::{Ppu, flags::background_pattern_table};

impl Ppu {
    /// Fetch the next tile index from the current nametable.
    fn fetch_nametable_byte(&mut self) {
        let address = 0x2000 | (self.v & 0x0FFF);
        self.next_tile_id = self.bus.read(address);
    }

    /// Fetch the attribute byte from the current tile.
    fn fetch_attribute_byte(&mut self) {
        let address = 0x23C0 | (self.v & 0x0C00) | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07);
        let byte = self.bus.read(address);

        let shift = ((self.v >> 4) & 0x04) | (self.v & 0x02);
        self.next_tile_attr = (byte >> shift) & 0x03;
    }

    /// Fetch the low pattern byte from the current tile.
    fn fetch_pattern_lo(&mut self) {
        let fine_y = (self.v >> 12) & 0x07;
        let address =
            background_pattern_table(self.ctrl) + (self.next_tile_id as u16 * 16) + fine_y;
        self.next_tile_lo = self.bus.read(address);
    }

    /// Fetch the high pattern byte from the current tile.
    fn fetch_pattern_hi(&mut self) {
        let fine_y = (self.v >> 12) & 0x07;
        let address =
            background_pattern_table(self.ctrl) + (self.next_tile_id as u16 * 16) + fine_y + 8;
        self.next_tile_hi = self.bus.read(address);
    }

    /// Reload the background shift registers with the tile fetched over the previous 8 PPU cycles.
    fn reload_shift_registers(&mut self) {
        self.bg_pattern_shift_lo = (self.bg_pattern_shift_lo & 0xFF00) | self.next_tile_lo as u16;
        self.bg_pattern_shift_hi = (self.bg_pattern_shift_hi & 0xFF00) | self.next_tile_hi as u16;

        let attr_lo_fill = if self.next_tile_attr & 0b01 != 0 {
            0xFF
        } else {
            0x00
        };

        let attr_hi_fill = if self.next_tile_attr & 0b10 != 0 {
            0xFF
        } else {
            0x00
        };

        self.bg_attr_shift_lo = (self.bg_attr_shift_lo & 0xFF00) | attr_lo_fill;
        self.bg_attr_shift_hi = (self.bg_attr_shift_hi & 0xFF00) | attr_hi_fill;
    }

    /// Advance the background rendering pipeline by one pixel.
    pub fn shift_register(&mut self) {
        self.bg_pattern_shift_lo <<= 1;
        self.bg_pattern_shift_hi <<= 1;
        self.bg_attr_shift_lo <<= 1;
        self.bg_attr_shift_hi <<= 1;
    }

    /// Increment the coarse X component of the VRAM address (`v`).
    fn increment_coarse_x(&mut self) {
        if self.v & 0x001F == 31 {
            self.v &= !0x001F;
            self.v ^= 0x0400;
        } else {
            self.v = self.v.wrapping_add(1);
        }
    }

    /// Increment the vertical scroll component of the VRAM address (`v`).
    pub fn increment_y(&mut self) {
        if self.v & 0x7000 != 0x7000 {
            self.v += 0x1000;
        } else {
            self.v &= !0x7000;
            let mut coarse_y = (self.v & 0x03E0) >> 5;
            if coarse_y == 29 {
                coarse_y = 0;
                self.v ^= 0x0800;
            } else if coarse_y == 31 {
                coarse_y = 0;
            } else {
                coarse_y += 1;
            }

            self.v = (self.v & !0x03E0) | (coarse_y << 5);
        }
    }

    /// Copy the horizontal scrolling bits from `t` into `v`.
    pub fn copy_horizontal_bits(&mut self) {
        self.v = (self.v & !0x041F) | (self.t & 0x041F);
    }

    /// Copies the vertical scrolling bits from `t` into `v`.
    pub fn copy_vertical_bits(&mut self) {
        self.v = (self.v & !0x7BE0) | (self.t & 0x7BE0);
    }

    /// Execute one step of the PPU's repeating 8-cycle background fetch pipeline.
    pub fn fetch_background_tile(&mut self) {
        match self.cycle % 8 {
            1 => self.fetch_nametable_byte(),
            3 => self.fetch_attribute_byte(),
            5 => self.fetch_pattern_lo(),
            7 => self.fetch_pattern_hi(),
            0 => {
                self.reload_shift_registers();
                self.increment_coarse_x();
            }
            _ => {}
        }
    }
}
