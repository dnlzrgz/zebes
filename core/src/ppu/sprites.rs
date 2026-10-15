use crate::{
    bits::{contains, set},
    ppu::{
        Ppu,
        flags::{
            MASK_SHOW_SPRITES, MASK_SHOW_SPRITES_LEFT, STATUS_SPRITE_OVERFLOW, sprite_height,
            sprite_pattern_table,
        },
    },
};

/// Maximum number of sprites that can be displayed on a single scanline.
const MAX_SPRITES_PER_SCANLINE: usize = 8;

/// Highest-priority, non-transparent sprite pixel for the current screen position.
pub struct SpritePixel {
    // 2-bit color index from the sprite's palette. Transparent pixels are filtered out.
    pub pixel: u8,
    // Which of the four sprite palettes (0-3) this pixel comes from.
    pub palette: u8,
    // True if the sprite should be drawn behind a background pixel.
    pub hidden: bool,
    // Used for sprite-zero-hit.
    pub is_sprite_zero: bool,
}

impl Ppu {
    /// Evaluate sprites for the next scanline.
    ///
    /// The primary OAM is scanned in order and up to eight visible sprites are copied into the
    /// secondary OAM if necessary. If there are more than 8 visible sprites, the sprite overflow
    /// flag is set.
    pub fn check_sprites(&mut self) {
        // Scanline that is about to be rendered. The pre-render 261 evaluates for scanline 0 of the
        // next frame.
        let target_scanline = if self.scanline == 261 {
            0
        } else {
            self.scanline + 1
        };
        let height = sprite_height(self.ctrl) as u16;

        self.secondary_oam = [0xFF; 32];
        self.secondary_oam_count = 0;
        self.sprite_zero_in_range = false;

        for sprite_index in 0..64usize {
            let base = sprite_index * 4;
            let sprite_y = self.oam[base] as u16;

            let row_in_sprite = target_scanline.wrapping_sub(sprite_y + 1);
            if row_in_sprite >= height {
                continue; // The sprite isn't visible on the target scaline.
            }

            if self.secondary_oam_count < MAX_SPRITES_PER_SCANLINE as u8 {
                let dest = self.secondary_oam_count as usize * 4;
                self.secondary_oam[dest..dest + 4].copy_from_slice(&self.oam[base..base + 4]);

                if sprite_index == 0 {
                    self.sprite_zero_in_range = true;
                }

                self.secondary_oam_count += 1;
            } else {
                // The Ppu can only display 8 sprites per scaline so the rest are dropped and the
                // overflow flag is set.
                set(&mut self.status, STATUS_SPRITE_OVERFLOW, true);
                break;
            }
        }
    }

    /// Fetch pattern data for the sprites selected during the evaluation.
    ///
    /// Pattern bytes are laoded into the internal sprite shift registers, applying 8x16 addressing
    /// and horizontal or vertical flipping if required.
    pub fn fetch_sprites(&mut self) {
        let target_scanline = if self.scanline == 261 {
            0
        } else {
            self.scanline + 1
        };
        let height = sprite_height(self.ctrl) as u16;

        for slot in 0..MAX_SPRITES_PER_SCANLINE {
            if slot >= self.secondary_oam_count as usize {
                // Free slot for the current scanline.
                self.sprite_pattern_lo[slot] = 0;
                self.sprite_pattern_hi[slot] = 0;
                self.sprite_attr[slot] = 0;
                self.sprite_x_counter[slot] = 0xFF;
                continue;
            }

            let base = slot * 4;
            let sprite_y = self.secondary_oam[base] as u16;
            let tile_index = self.secondary_oam[base + 1];
            let attr = self.secondary_oam[base + 2];
            let sprite_x = self.secondary_oam[base + 3];

            let flip_horizontal = attr & 0x40 != 0;
            let flip_vertical = attr & 0x80 != 0;

            let mut row = target_scanline.wrapping_sub(sprite_y + 1);
            if flip_vertical {
                row = height - 1 - row;
            }

            let address = if height == 16 {
                let table = if tile_index & 0x01 != 0 {
                    0x1000
                } else {
                    0x0000
                };
                let mut tile = (tile_index & 0xFE) as u16;
                let mut row = row;
                if row >= 8 {
                    tile += 1;
                    row -= 8;
                }
                table + tile * 16 + row
            } else {
                sprite_pattern_table(self.ctrl) + tile_index as u16 * 16 + row
            };

            let mut lo = self.bus.read(address);
            let mut hi = self.bus.read(address + 8);

            if flip_horizontal {
                lo = lo.reverse_bits();
                hi = hi.reverse_bits();
            }

            self.sprite_pattern_lo[slot] = lo;
            self.sprite_pattern_hi[slot] = hi;
            self.sprite_attr[slot] = attr;
            self.sprite_x_counter[slot] = sprite_x;
        }
    }

    /// Advance the sprite rendering pipeline by one pixel.
    pub fn tick_sprites(&mut self) {
        for slot in 0..self.secondary_oam_count as usize {
            if self.sprite_x_counter[slot] > 0 {
                self.sprite_x_counter[slot] -= 1;
            } else {
                self.sprite_pattern_lo[slot] <<= 1;
                self.sprite_pattern_hi[slot] <<= 1;
            }
        }
    }

    /// Return the highest-priority visible sprite pixel at the current x position.
    pub fn sprite_pixel(&self, x: usize) -> Option<SpritePixel> {
        if !contains(self.mask, MASK_SHOW_SPRITES) {
            return None;
        }

        if x < 8 && !contains(self.mask, MASK_SHOW_SPRITES_LEFT) {
            return None;
        }

        for slot in 0..self.secondary_oam_count as usize {
            if self.sprite_x_counter[slot] != 0 {
                continue;
            }

            let lo = (self.sprite_pattern_lo[slot] >> 7) & 1;
            let hi = (self.sprite_pattern_hi[slot] >> 7) & 1;
            let pixel = (hi << 1) | lo;

            if pixel == 0 {
                continue; // Transparent
            }

            let attr = self.sprite_attr[slot];
            return Some(SpritePixel {
                pixel,
                palette: attr & 0x03,
                hidden: attr & 0x20 != 0,
                is_sprite_zero: slot == 0 && self.sprite_zero_in_range,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cartridge::Mirroring,
        ppu::{
            flags::{CTRL_SPRITE_PATTERN_TABLE, CTRL_SPRITE_SIZE},
            test_utils::ppu_with_chr_rom,
        },
    };

    fn ppu_with_blank_oam() -> Ppu {
        let mut ppu = Ppu::new();
        ppu.oam = [0xFF; 256];
        ppu
    }

    #[test]
    fn check_sprites_finds_sprite_in_range() {
        let mut ppu = ppu_with_blank_oam();
        ppu.scanline = 10; // testing scanline 11
        ppu.oam[0] = 10; // sprite_y = 10 -> visible rows 11..=18 (8px sprite)
        ppu.oam[1] = 0x42; // tile
        ppu.oam[2] = 0x00; // attr
        ppu.oam[3] = 5; // x

        ppu.check_sprites();

        assert_eq!(ppu.secondary_oam_count, 1);
        assert_eq!(ppu.secondary_oam[1], 0x42);
        assert!(ppu.sprite_zero_in_range);
    }

    #[test]
    fn check_sprites_sets_overflow_flag_past_eight_sprites() {
        let mut ppu = ppu_with_blank_oam();
        ppu.scanline = 10;
        for i in 0..9usize {
            let base = i * 4;
            ppu.oam[base] = 10; // all in range for scanline 11
        }

        ppu.check_sprites();

        assert_eq!(ppu.secondary_oam_count, 8);
        assert!(contains(ppu.status, STATUS_SPRITE_OVERFLOW));
    }

    #[test]
    fn check_sprites_exactly_eight_does_not_set_overflow() {
        let mut ppu = ppu_with_blank_oam();
        ppu.scanline = 10;
        for i in 0..8usize {
            let base = i * 4;
            ppu.oam[base] = 10; // all in range for scanline 11
        }

        ppu.check_sprites();

        assert_eq!(ppu.secondary_oam_count, 8);
        assert!(!contains(ppu.status, STATUS_SPRITE_OVERFLOW));
    }

    #[test]
    fn check_sprites_does_not_mark_sprite_zero_when_out_of_range() {
        let mut ppu = ppu_with_blank_oam();
        ppu.scanline = 10;
        ppu.oam[0] = 100; // sprite 0 is below the target scanline
        ppu.oam[4] = 10; // sprite 1 is in range
        ppu.oam[5] = 0x77; // sprite 1 tile

        ppu.check_sprites();

        assert!(!ppu.sprite_zero_in_range);
        assert_eq!(ppu.secondary_oam_count, 1);
        assert_eq!(ppu.secondary_oam[1], 0x77); // copied sprite 1, not sprite 0
    }

    #[test]
    fn check_sprites_supports_8x16_sprites() {
        let mut ppu = Ppu::new();
        ppu.oam = [100; 256]; // Y=100 is out of range for target scanline 11
        ppu.ctrl = CTRL_SPRITE_SIZE; // 8x16
        ppu.scanline = 10; // testing for scanline 11
        ppu.oam[0] = 0; // visible: row 10 of 16
        ppu.oam[1] = 0x42;
        ppu.oam[2] = 0x00;
        ppu.oam[3] = 5;

        ppu.check_sprites();

        assert_eq!(ppu.secondary_oam_count, 1);
        assert_eq!(ppu.secondary_oam[1], 0x42);
        assert!(ppu.sprite_zero_in_range);
    }

    #[test]
    fn check_sprites_visible_boundary_is_height_minus_one() {
        let mut ppu = ppu_with_blank_oam();
        ppu.scanline = 10; // testing for scanline 11, 8x8

        ppu.oam[0] = 3; // row = 11 - 4 = 7 (visible)
        ppu.oam[4] = 2; // row = 11 - 3 = 8 (not visible)

        ppu.check_sprites();

        assert_eq!(ppu.secondary_oam_count, 1);
        assert_eq!(ppu.secondary_oam[0], 3); // only the visible sprite was copied
    }

    #[test]
    fn fetch_sprites_loads_8x8_sprite_patterns() {
        let mut chr_rom = [0u8; 8192];
        chr_rom[0] = 0x11; // plane 0 of tile 0
        chr_rom[8] = 0x22; // plane 1 of tile 0
        let mut ppu = ppu_with_chr_rom(&chr_rom, Mirroring::Horizontal);

        ppu.ctrl = 0; // 8x8 sprites, sprite pattern table at $0000
        ppu.scanline = 0; // testing for scanline 1
        ppu.secondary_oam_count = 1;
        ppu.secondary_oam[0] = 0; // sprite_y = 0 -> row 0
        ppu.secondary_oam[1] = 0; // tile 0
        ppu.secondary_oam[2] = 0x03; // attr
        ppu.secondary_oam[3] = 7; // x

        ppu.fetch_sprites();

        assert_eq!(ppu.sprite_pattern_lo[0], 0x11);
        assert_eq!(ppu.sprite_pattern_hi[0], 0x22);
        assert_eq!(ppu.sprite_attr[0], 0x03);
        assert_eq!(ppu.sprite_x_counter[0], 7);
    }

    #[test]
    fn fetch_sprites_uses_second_half_of_8x16_tile_for_lower_rows() {
        let mut chr_rom = [0u8; 8192];
        chr_rom[16] = 0x33; // tile 1, plane 0
        chr_rom[24] = 0x44; // tile 1, plane 1
        let mut ppu = ppu_with_chr_rom(&chr_rom, Mirroring::Horizontal);

        ppu.ctrl = CTRL_SPRITE_SIZE; // 8x16 sprites
        ppu.scanline = 8; // testing for scanline 9
        ppu.secondary_oam_count = 1;
        ppu.secondary_oam[0] = 0; // sprite_y = 0 -> row 8, second half of the tile
        ppu.secondary_oam[1] = 0; // tile 0
        ppu.secondary_oam[2] = 0x00;
        ppu.secondary_oam[3] = 0;

        ppu.fetch_sprites();

        assert_eq!(ppu.sprite_pattern_lo[0], 0x33);
        assert_eq!(ppu.sprite_pattern_hi[0], 0x44);
    }

    #[test]
    fn fetch_sprites_uses_second_pattern_table_for_odd_8x16_tiles() {
        let mut chr_rom = [0u8; 8192];
        chr_rom[0x1000] = 0x55;
        let mut ppu = ppu_with_chr_rom(&chr_rom, Mirroring::Horizontal);

        ppu.ctrl = CTRL_SPRITE_SIZE;
        ppu.scanline = 0; // testing for scanline 1
        ppu.secondary_oam_count = 1;
        ppu.secondary_oam[0] = 0; // sprite_y = 0 -> row 0
        ppu.secondary_oam[1] = 1; // odd tile index selects the $1000 pattern table
        ppu.secondary_oam[2] = 0x00;
        ppu.secondary_oam[3] = 0;

        ppu.fetch_sprites();

        assert_eq!(ppu.sprite_pattern_lo[0], 0x55);
    }

    #[test]
    fn fetch_sprites_applies_horizontal_flip() {
        let mut chr_rom = [0u8; 8192];
        chr_rom[0] = 0b1000_0000;
        chr_rom[8] = 0b0000_0001;
        let mut ppu = ppu_with_chr_rom(&chr_rom, Mirroring::Horizontal);

        ppu.ctrl = 0;
        ppu.scanline = 0; // testing for scanline 1
        ppu.secondary_oam_count = 1;
        ppu.secondary_oam[0] = 0;
        ppu.secondary_oam[1] = 0;
        ppu.secondary_oam[2] = 0x40; // horizontal flip
        ppu.secondary_oam[3] = 0;

        ppu.fetch_sprites();

        assert_eq!(ppu.sprite_pattern_lo[0], 0b0000_0001); // reversed low plane
        assert_eq!(ppu.sprite_pattern_hi[0], 0b1000_0000); // reversed high plane
    }

    #[test]
    fn fetch_sprites_applies_vertical_flip() {
        let mut chr_rom = [0u8; 8192];
        chr_rom[0] = 0x00;
        chr_rom[7] = 0x77; // the row used when the sprite is flipped vertically
        let mut ppu = ppu_with_chr_rom(&chr_rom, Mirroring::Horizontal);

        ppu.ctrl = 0;
        ppu.scanline = 0; // testing for scanline 1
        ppu.secondary_oam_count = 1;
        ppu.secondary_oam[0] = 0; // row 0 before flipping
        ppu.secondary_oam[1] = 0;
        ppu.secondary_oam[2] = 0x80; // vertical flip
        ppu.secondary_oam[3] = 0;

        ppu.fetch_sprites();

        assert_eq!(ppu.sprite_pattern_lo[0], 0x77);
    }

    #[test]
    fn fetch_sprites_uses_second_pattern_table_for_8x8_sprites() {
        let mut chr_rom = [0u8; 8192];
        chr_rom[0x1000] = 0x55; // $1000 table, tile 0, row 0
        let mut ppu = ppu_with_chr_rom(&chr_rom, Mirroring::Horizontal);

        ppu.ctrl = CTRL_SPRITE_PATTERN_TABLE; // 8x8 sprites, pattern table at $1000
        ppu.scanline = 0; // testing for scanline 1
        ppu.secondary_oam_count = 1;
        ppu.secondary_oam[0] = 0; // sprite_y = 0 -> row 0
        ppu.secondary_oam[1] = 0; // tile 0
        ppu.secondary_oam[2] = 0x00;
        ppu.secondary_oam[3] = 0;

        ppu.fetch_sprites();

        assert_eq!(ppu.sprite_pattern_lo[0], 0x55);
    }

    #[test]
    fn fetch_sprites_vertical_flip_swaps_8x16_tile_halves() {
        let mut chr_rom = [0u8; 8192];
        chr_rom[0x17] = 0x66; // tile 1, row 7, plane 0
        chr_rom[0x1F] = 0x77; // tile 1, row 7, plane 1
        let mut ppu = ppu_with_chr_rom(&chr_rom, Mirroring::Horizontal);

        ppu.ctrl = CTRL_SPRITE_SIZE; // 8x16
        ppu.scanline = 0; // testing for scanline 1, row 0 before flipping
        ppu.secondary_oam_count = 1;
        ppu.secondary_oam[0] = 0; // sprite_y = 0
        ppu.secondary_oam[1] = 0; // tile 0 -> halves 0 and 1
        ppu.secondary_oam[2] = 0x80; // vertical flip
        ppu.secondary_oam[3] = 0;

        ppu.fetch_sprites();

        assert_eq!(ppu.sprite_pattern_lo[0], 0x66);
        assert_eq!(ppu.sprite_pattern_hi[0], 0x77);
    }

    #[test]
    fn fetch_sprites_clears_unused_slots() {
        let mut ppu = Ppu::new();
        ppu.scanline = 0;
        ppu.secondary_oam_count = 1;
        ppu.secondary_oam[0] = 0;
        ppu.secondary_oam[1] = 0;
        ppu.secondary_oam[2] = 0;
        ppu.secondary_oam[3] = 0;

        // Fill unused slot to ensure it gets reset.
        ppu.sprite_pattern_lo[1] = 0xAA;
        ppu.sprite_pattern_hi[1] = 0xBB;
        ppu.sprite_attr[1] = 0xCC;
        ppu.sprite_x_counter[1] = 0x00;

        ppu.fetch_sprites();

        assert_eq!(ppu.sprite_pattern_lo[1], 0);
        assert_eq!(ppu.sprite_pattern_hi[1], 0);
        assert_eq!(ppu.sprite_attr[1], 0);
        assert_eq!(ppu.sprite_x_counter[1], 0xFF);
    }

    #[test]
    fn tick_sprites_decrements_x_counter_until_zero_then_shifts() {
        let mut ppu = Ppu::new();
        ppu.secondary_oam_count = 1;
        ppu.sprite_x_counter[0] = 2;
        ppu.sprite_pattern_lo[0] = 0x80;
        ppu.sprite_pattern_hi[0] = 0x01;

        ppu.tick_sprites();
        assert_eq!(ppu.sprite_x_counter[0], 1);
        assert_eq!(ppu.sprite_pattern_lo[0], 0x80); // no shift yet

        ppu.tick_sprites();
        assert_eq!(ppu.sprite_x_counter[0], 0);
        assert_eq!(ppu.sprite_pattern_lo[0], 0x80); // still no shift

        ppu.tick_sprites();
        assert_eq!(ppu.sprite_x_counter[0], 0);
        assert_eq!(ppu.sprite_pattern_lo[0], 0x00); // now the pattern shifts
        assert_eq!(ppu.sprite_pattern_hi[0], 0x02);
    }

    #[test]
    fn sprite_pixel_returns_none_when_sprites_disabled() {
        let mut ppu = Ppu::new();
        ppu.mask = 0;
        ppu.secondary_oam_count = 1;
        ppu.sprite_x_counter[0] = 0;
        ppu.sprite_pattern_lo[0] = 0x80;
        ppu.sprite_pattern_hi[0] = 0;

        assert!(ppu.sprite_pixel(8).is_none());
    }

    #[test]
    fn sprite_pixel_hides_leftmost_8_pixels_unless_enabled() {
        let mut ppu = Ppu::new();
        ppu.mask = MASK_SHOW_SPRITES;
        ppu.secondary_oam_count = 1;
        ppu.sprite_x_counter[0] = 0;
        ppu.sprite_pattern_lo[0] = 0x80;
        ppu.sprite_pattern_hi[0] = 0;
        ppu.sprite_attr[0] = 0x00;

        assert!(ppu.sprite_pixel(0).is_none());

        ppu.mask |= MASK_SHOW_SPRITES_LEFT;
        assert!(ppu.sprite_pixel(0).is_some());
    }

    #[test]
    fn sprite_pixel_returns_none_when_all_sprites_transparent() {
        let mut ppu = Ppu::new();
        ppu.mask = MASK_SHOW_SPRITES;
        ppu.secondary_oam_count = 1;
        ppu.sprite_x_counter[0] = 0;
        ppu.sprite_pattern_lo[0] = 0;
        ppu.sprite_pattern_hi[0] = 0;
        ppu.sprite_attr[0] = 0;

        assert!(ppu.sprite_pixel(8).is_none());
    }

    #[test]
    fn sprite_pixel_skips_sprites_not_yet_at_their_x_position() {
        let mut ppu = Ppu::new();
        ppu.mask = MASK_SHOW_SPRITES;
        ppu.secondary_oam_count = 2;
        ppu.sprite_x_counter[0] = 1; // not visible yet
        ppu.sprite_pattern_lo[0] = 0x80;
        ppu.sprite_pattern_hi[0] = 0;
        ppu.sprite_attr[0] = 0x01;
        ppu.sprite_x_counter[1] = 0; // visible now
        ppu.sprite_pattern_lo[1] = 0x00; // pixel index 2 = high plane only
        ppu.sprite_pattern_hi[1] = 0x80;
        ppu.sprite_attr[1] = 0x02;

        let pixel = ppu.sprite_pixel(8).unwrap();

        assert_eq!(pixel.pixel, 2);
        assert_eq!(pixel.palette, 0x02);
    }

    #[test]
    fn sprite_pixel_skips_transparent_pixels_and_returns_first_opaque() {
        let mut ppu = Ppu::new();
        ppu.mask = MASK_SHOW_SPRITES;
        ppu.secondary_oam_count = 2;
        ppu.sprite_x_counter[0] = 0;
        ppu.sprite_x_counter[1] = 0;
        ppu.sprite_pattern_lo[0] = 0x00; // transparent
        ppu.sprite_pattern_hi[0] = 0x00;
        ppu.sprite_attr[0] = 0x00;
        ppu.sprite_pattern_lo[1] = 0x80; // pixel index 1
        ppu.sprite_pattern_hi[1] = 0x00;
        ppu.sprite_attr[1] = 0x03;

        let pixel = ppu.sprite_pixel(8).unwrap();

        assert_eq!(pixel.pixel, 1);
        assert_eq!(pixel.palette, 0x03);
        assert!(!pixel.hidden);
        assert!(!pixel.is_sprite_zero);
    }

    #[test]
    fn sprite_pixel_marks_sprite_zero_when_in_range() {
        let mut ppu = Ppu::new();
        ppu.mask = MASK_SHOW_SPRITES;
        ppu.sprite_zero_in_range = true;
        ppu.secondary_oam_count = 1;
        ppu.sprite_x_counter[0] = 0;
        ppu.sprite_pattern_lo[0] = 0x80;
        ppu.sprite_pattern_hi[0] = 0x00;
        ppu.sprite_attr[0] = 0x00;

        let pixel = ppu.sprite_pixel(8).unwrap();

        assert!(pixel.is_sprite_zero);
    }

    #[test]
    fn sprite_pixel_reports_hidden_attribute() {
        let mut ppu = Ppu::new();
        ppu.mask = MASK_SHOW_SPRITES;
        ppu.secondary_oam_count = 1;
        ppu.sprite_x_counter[0] = 0;
        ppu.sprite_pattern_lo[0] = 0x80;
        ppu.sprite_pattern_hi[0] = 0x00;
        ppu.sprite_attr[0] = 0x20 | 0x01; // behind background, palette 1

        let pixel = ppu.sprite_pixel(8).unwrap();

        assert!(pixel.hidden);
        assert_eq!(pixel.palette, 0x01);
    }
}
