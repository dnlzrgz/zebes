use crate::{
    bits::contains,
    ppu::{
        Ppu, SCREEN_HEIGHT, SCREEN_WIDTH,
        flags::{MASK_SHOW_BACKGROUND, background_pattern_table},
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

    /// Produces the background pixel for the current screen position.
    pub fn render_pixel(&mut self) {
        // Fine X scrolling does not physically shift the background registers. Instead, it selects
        // which bit position should be sampled from the 16-bit shift registers.
        //
        // x = 0 samples the most significant bit. x = 7 samples the eight bit.
        let bit_mux = 0x8000 >> self.x;

        // The background pattern is stored in two bitplanes. Each shift register contributes one
        // bit. Together, they produce the 2-bit pixel value within the selected background palette.
        let pattern_lo = ((self.bg_pattern_shift_lo & bit_mux) != 0) as u8;
        let pattern_hi = ((self.bg_pattern_shift_hi & bit_mux) != 0) as u8;
        let pixel = (pattern_hi << 1) | pattern_lo;

        // The attribute bits are shifted in parallel with the pattern bits. These two registers
        // provide the upper two bits of the palette address, selecting one of the four background palettes.
        let attr_lo = ((self.bg_attr_shift_lo & bit_mux) != 0) as u8;
        let attr_hi = ((self.bg_attr_shift_hi & bit_mux) != 0) as u8;
        let palette = (attr_hi << 1) | attr_lo;

        // The pixel value 0 is always transparent with respect to the selected background palette
        // and instead uses the universal background color stored at $3F00.
        let address = if !contains(self.mask, MASK_SHOW_BACKGROUND) || pixel == 0 {
            0x3F00
        } else {
            0x3F00 + (palette as u16 * 4) + pixel as u16
        };

        let color_index = self.bus.read(address) & 0x3F;

        // Convert the current scanline/cycle into a framebuffer index. Visible pixels are generated
        // during cycles 1-256 of visible scanlines (0-239). Cycle 1 corresponds to the leftmost pixel.
        let x = (self.cycle - 1) as usize;
        let y = self.scanline as usize;

        // Translate the NES palette index into a 24-bit RGB color and write the finished pixel into
        // the emulator's framebuffer.
        if x < SCREEN_WIDTH && y < SCREEN_HEIGHT {
            self.frame_buffer[y * SCREEN_WIDTH + x] = PALETTE[color_index as usize];
        }
    }
}
