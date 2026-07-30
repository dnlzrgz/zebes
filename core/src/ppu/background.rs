use crate::{
    bits::{contains, set},
    ppu::{
        Ppu, SCREEN_HEIGHT, SCREEN_WIDTH,
        flags::{
            MASK_SHOW_BACKGROUND, MASK_SHOW_BACKGROUND_LEFT, STATUS_SPRITE_ZERO_HIT,
            background_pattern_table,
        },
        palette::PALETTE,
    },
};

impl Ppu {
    /// Fetch the next tile number from the current nametable.
    ///
    /// During rendering the PPU continously perform an 8-cycle fetch sequence. The first memory
    /// access gets the tile index from the selected nametable by the current VRAM address. The low
    /// 12 bits of `v` encode the current scroll position. Combining these bits with $2000 produces
    /// the address of the nametable entry for the tile that is being rendered.
    fn fetch_nametable_byte(&mut self) {
        let address = 0x2000 | (self.v & 0x0FFF);
        self.next_tile_id = self.bus.read(address);
    }

    /// Fetch the attribute byte corresponding to the current tile.
    ///
    /// Attribute tables begin at $23C0 within each nametable and assign one of the four background
    /// palettes to every 32x32 pixel regions (4x4 tile area.) Each attribute byte contains four
    /// 2-bit palette selectors. The coaarse X and coarse Y scroll determine which quadran is being
    /// rendered. The selected 2-bit value is expanded later into dedicated attribute shift
    /// registers so it can be shifted alongside the pattern bits.
    fn fetch_attribute_byte(&mut self) {
        let address = 0x23C0 | (self.v & 0x0C00) | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07);
        let byte = self.bus.read(address);

        let shift = ((self.v >> 4) & 0x04) | (self.v & 0x02);
        self.next_tile_attr = (byte >> shift) & 0x03;
    }

    /// Fetch the low pattern byte for the current tile.
    ///
    /// Pattern tables store each tile as two bitplanes. The first plane contains the
    /// least-significan bit of every pixel in one tile row. The tile number selects a 16-byte
    /// pattern, while the fine Y scroll selects one of its eight rows.
    fn fetch_pattern_lo(&mut self) {
        let fine_y = (self.v >> 12) & 0x07;
        let address =
            background_pattern_table(self.ctrl) + (self.next_tile_id as u16 * 16) + fine_y;
        self.next_tile_lo = self.bus.read(address);
    }

    /// Fetch the high pattern byte for the current tile.
    ///
    /// The second bitplace immediately follows the first 8-bytes of a tile. Together, the low and
    /// high pattern bytes form the 2-bit color index for each pixel in the tile row.
    fn fetch_pattern_hi(&mut self) {
        let fine_y = (self.v >> 12) & 0x07;
        let address =
            background_pattern_table(self.ctrl) + (self.next_tile_id as u16 * 16) + fine_y + 8;
        self.next_tile_hi = self.bus.read(address);
    }

    /// Load the tile fetched during the previous 8-cycle sequence into the background shift
    /// registers.
    ///
    /// The PPU renders one pixel every cycle while simultaneously fetching the next tile. To
    /// achieve this overlap, the current tile occupies the upper half of each 16-bit shift register
    /// while the newly fetched tile is loaded into the lower half. While pattern bytes are copied
    /// directly, the attribute bits are expanded from a 2-bit palette number into bytes of either
    /// $00 or $FF. This allows the palette bits to shift in parallel with the pattern data so that
    /// a single bit from each register can be sampled every single pixel.
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

    /// Advance the background rendering by one pixel.
    ///
    /// The background shift registers shift left once every visible PPU cycle. After each shift,
    /// the next pixel is exposed at the most significant bit. Fine X scrolling does not change the
    /// registers themselves. Instead, it just changes which bit position is sampled when the pixel
    /// is generated.
    pub fn shift_register(&mut self) {
        self.bg_pattern_shift_lo <<= 1;
        self.bg_pattern_shift_hi <<= 1;
        self.bg_attr_shift_lo <<= 1;
        self.bg_attr_shift_hi <<= 1;
    }

    /// Increment the coarse X component of the VRAM address.
    ///
    /// Coarse X selects one of the 32 tile columns within a nametable. After the last column (31)
    /// the PPU goes back to column 0 and toggles the horizontal nametable bit, allowing rendering
    /// to continue into the adjacent nametable.
    fn increment_coarse_x(&mut self) {
        if self.v & 0x001F == 31 {
            self.v &= !0x001F;
            self.v ^= 0x0400;
        } else {
            self.v = self.v.wrapping_add(1);
        }
    }

    /// Increment the vertical scroll past of the VRAM address.
    ///
    /// Vertical scrolling is split into two fields: Fine Y (bits 12-14) and Coarse Y (bits 5-9).
    /// Fine Y selects one of the eight rows within the current tile and once it reaches row 7 it
    /// goes back to zero while Coarse Y selects the tile row itself. When Coarse Y reaches the tile
    /// row 29, the vertical nametable bit is toggled so rendering continues into the next
    /// nametable. Rows 30 and 31 are not displayed and wrap without changing nametables.
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

    /// Copy the horizontal scrolling bits from the temporary VRAM address.
    ///
    /// During rendering the CPU writes scroll values into the temporary register. At dot 257 of
    /// every visible scanline the PPU copies the horizontal scrolling field from `t` into the
    /// current VRAM address (`v`). This updates Coarse X and the horizontal nametable bit. The
    /// vertical scrolling reamins unchanged.
    pub fn copy_horizontal_bits(&mut self) {
        self.v = (self.v & !0x041F) | (self.t & 0x041F);
    }

    /// Copies the vertical scrolling bits from the temporary VRAM address.
    ///
    /// During the pre-render scanline (261), dots 280-304, the PPU restores the vertical scrolling
    /// fields from `t` into `v`. This reloads Fine Y, Coarse Y, and the vertical nametable bit.
    /// Together with the horizontal copy performed at dot 257, this prepares the VRAM address for
    /// the next frame.
    pub fn copy_vertical_bits(&mut self) {
        self.v = (self.v & !0x7BE0) | (self.t & 0x7BE0);
    }

    /// Execute one iteration of the background fetch process.
    ///
    /// While the background rendering is enabled, the PPU repeats the same sequence every eight cycles.
    pub fn fetch_background_tile(&mut self) {
        match self.cycle % 8 {
            1 => self.fetch_nametable_byte(), // Fetch nametable byte.
            3 => self.fetch_attribute_byte(), // Fetch attribute byte.
            5 => self.fetch_pattern_lo(),     // Fetch pattern low byte.
            7 => self.fetch_pattern_hi(),     // Fetch pattern high byte.
            0 => {
                self.reload_shift_registers(); // Reload shift registers.
                self.increment_coarse_x(); // Increment Coarse X.
            }
            _ => {}
        }
    }

    /// Render the necessary pixel (background + sprite) for the current screen position.
    pub fn render_pixel(&mut self) {
        let x = (self.cycle - 1) as usize;
        let y = self.scanline as usize;

        let bit_mux = 0x8000 >> self.x;

        let pattern_lo = ((self.bg_pattern_shift_lo & bit_mux) != 0) as u8;
        let pattern_hi = ((self.bg_pattern_shift_hi & bit_mux) != 0) as u8;
        let bg_pixel = (pattern_hi << 1) | pattern_lo;

        let attr_lo = ((self.bg_attr_shift_lo & bit_mux) != 0) as u8;
        let attr_hi = ((self.bg_attr_shift_hi & bit_mux) != 0) as u8;
        let bg_palette = (attr_hi << 1) | attr_lo;

        let bg_opaque = contains(self.mask, MASK_SHOW_BACKGROUND)
            && bg_pixel != 0
            && (x >= 8 || contains(self.mask, MASK_SHOW_BACKGROUND_LEFT));

        let sprite = self.sprite_pixel(x);

        let address = match (&sprite, bg_opaque) {
            (Some(s), false) => 0x3F10 + s.palette as u16 * 4 + s.pixel as u16,
            (None, false) => 0x3F00,
            (None, true) => 0x3F00 + bg_palette as u16 * 4 + bg_pixel as u16,
            (Some(s), true) => {
                if s.is_sprite_zero && x != 255 {
                    set(&mut self.status, STATUS_SPRITE_ZERO_HIT, true);
                }

                if s.hidden {
                    0x3F00 + bg_palette as u16 * 4 + bg_pixel as u16
                } else {
                    0x3F10 + s.palette as u16 * 4 + s.pixel as u16
                }
            }
        };

        let color_index = self.bus.read(address) & 0x3F;

        if x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
            self.frame_buffer[y * SCREEN_WIDTH + x] = PALETTE[color_index as usize];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment_coarse_x_wraps_at_31_and_toggles_nametable() {
        let mut ppu = Ppu::new();
        ppu.v = 0x001F; // coarse X = 31

        ppu.increment_coarse_x();

        assert_eq!(ppu.v & 0x001F, 0); // coarse X wrapped to 0
        assert_eq!(ppu.v & 0x0400, 0x0400); // horizontal nametable bit toggled
    }

    #[test]
    fn increment_coarse_x_does_not_toggle_nametable_below_31() {
        let mut ppu = Ppu::new();
        ppu.v = 0x0005;

        ppu.increment_coarse_x();

        assert_eq!(ppu.v & 0x001F, 6);
        assert_eq!(ppu.v & 0x0400, 0);
    }

    #[test]
    fn increment_y_bumps_fine_y_when_not_at_max() {
        let mut ppu = Ppu::new();
        ppu.v = 0x0000; // fine Y = 0

        ppu.increment_y();

        assert_eq!((ppu.v >> 12) & 0x07, 1);
    }

    #[test]
    fn increment_y_wraps_coarse_y_at_29_and_toggles_nametable() {
        let mut ppu = Ppu::new();
        ppu.v = 0x7000 | (29 << 5); // fine Y = 7, coarse Y = 29

        ppu.increment_y();

        assert_eq!((ppu.v >> 12) & 0x07, 0); // fine Y wrapped
        assert_eq!((ppu.v & 0x03E0) >> 5, 0); // coarse Y wrapped
        assert_eq!(ppu.v & 0x0800, 0x0800); // vertical nametable toggled
    }

    #[test]
    fn increment_y_wraps_coarse_y_at_31_without_toggling_nametable() {
        let mut ppu = Ppu::new();
        ppu.v = 0x7000 | (31 << 5);

        ppu.increment_y();

        assert_eq!((ppu.v & 0x03E0) >> 5, 0);
        assert_eq!(ppu.v & 0x0800, 0);
    }

    #[test]
    fn copy_horizontal_bits_copies_coarse_x_and_nametable_only() {
        let mut ppu = Ppu::new();
        ppu.v = 0x7BE0; // everything except horizontal bits set
        ppu.t = 0x041F; // only horizontal bits set

        ppu.copy_horizontal_bits();

        assert_eq!(ppu.v, 0x7BE0 | 0x041F);
    }

    #[test]
    fn copy_vertical_bits_copies_fine_y_coarse_y_and_nametable_only() {
        let mut ppu = Ppu::new();
        ppu.v = 0x041F; // horizontal bits set
        ppu.t = 0x7BE0; // vertical bits set

        ppu.copy_vertical_bits();

        assert_eq!(ppu.v, 0x041F | 0x7BE0);
    }

    #[test]
    fn reload_shift_registers_loads_low_byte_only() {
        let mut ppu = Ppu::new();
        ppu.bg_pattern_shift_lo = 0xAB00;
        ppu.bg_pattern_shift_hi = 0xCD00;
        ppu.next_tile_lo = 0x11;
        ppu.next_tile_hi = 0x22;
        ppu.next_tile_attr = 0b10;

        ppu.reload_shift_registers();

        assert_eq!(ppu.bg_pattern_shift_lo, 0xAB11);
        assert_eq!(ppu.bg_pattern_shift_hi, 0xCD22);
        assert_eq!(ppu.bg_attr_shift_lo & 0x00FF, 0x00); // attr bit 0 clear -> 0x00 fill
        assert_eq!(ppu.bg_attr_shift_hi & 0x00FF, 0xFF); // attr bit 1 set -> 0xFF fill
    }

    #[test]
    fn shift_register_shifts_all_four_registers_left() {
        let mut ppu = Ppu::new();
        ppu.bg_pattern_shift_lo = 0x8001;
        ppu.bg_pattern_shift_hi = 0x0001;
        ppu.bg_attr_shift_lo = 0x0001;
        ppu.bg_attr_shift_hi = 0x0001;

        ppu.shift_register();

        assert_eq!(ppu.bg_pattern_shift_lo, 0x0002);
        assert_eq!(ppu.bg_pattern_shift_hi, 0x0002);
        assert_eq!(ppu.bg_attr_shift_lo, 0x0002);
        assert_eq!(ppu.bg_attr_shift_hi, 0x0002);
    }

    #[test]
    fn fetch_nametable_byte_reads_from_current_v_address() {
        let mut ppu = Ppu::new();
        ppu.v = 0x0005; // low 12 bits -> nametable offset 5
        ppu.bus.write(0x2005, 0x42);

        ppu.fetch_nametable_byte();

        assert_eq!(ppu.next_tile_id, 0x42);
    }

    #[test]
    fn fetch_attribute_byte_selects_top_left_quadrant() {
        let mut ppu = Ppu::new();
        // coarse X = 0, coarse Y = 0 -> bit1 of both clear -> shift 0
        ppu.v = 0 | (0 << 5);
        let address = 0x23C0 | (ppu.v & 0x0C00) | ((ppu.v >> 4) & 0x38) | ((ppu.v >> 2) & 0x07);
        ppu.bus.write(address, 0b11_10_01_00);

        ppu.fetch_attribute_byte();

        assert_eq!(ppu.next_tile_attr, 0b00); // bits 1:0 of the byte
    }

    #[test]
    fn fetch_attribute_byte_selects_top_right_quadrant() {
        let mut ppu = Ppu::new();
        // coarse X = 2 (bit1 set), coarse Y = 0 (bit1 clear) -> shift 2
        ppu.v = 2 | (0 << 5);
        let address = 0x23C0 | (ppu.v & 0x0C00) | ((ppu.v >> 4) & 0x38) | ((ppu.v >> 2) & 0x07);
        ppu.bus.write(address, 0b11_10_01_00);

        ppu.fetch_attribute_byte();

        assert_eq!(ppu.next_tile_attr, 0b01); // bits 3:2
    }

    #[test]
    fn fetch_attribute_byte_selects_bottom_left_quadrant() {
        let mut ppu = Ppu::new();
        // coarse X = 0 (bit1 clear), coarse Y = 2 (bit1 set) -> shift 4
        ppu.v = 0 | (2 << 5);
        let address = 0x23C0 | (ppu.v & 0x0C00) | ((ppu.v >> 4) & 0x38) | ((ppu.v >> 2) & 0x07);
        ppu.bus.write(address, 0b11_10_01_00);

        ppu.fetch_attribute_byte();

        assert_eq!(ppu.next_tile_attr, 0b10); // bits 5:4
    }

    #[test]
    fn fetch_attribute_byte_selects_bottom_right_quadrant() {
        let mut ppu = Ppu::new();
        // coarse X = 2, coarse Y = 2 -> both bit1 set -> shift 6
        ppu.v = 2 | (2 << 5);
        let address = 0x23C0 | (ppu.v & 0x0C00) | ((ppu.v >> 4) & 0x38) | ((ppu.v >> 2) & 0x07);
        ppu.bus.write(address, 0b11_10_01_00);

        ppu.fetch_attribute_byte();

        assert_eq!(ppu.next_tile_attr, 0b11); // bits 7:6
    }
}
