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

    #[test]
    fn check_sprites_finds_sprite_in_range() {
        let mut ppu = Ppu::new();
        ppu.scanline = 10; // evaluating for scanline 11
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
        let mut ppu = Ppu::new();
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
    fn check_sprites_ignores_sprite_outside_vertical_range() {
        let mut ppu = Ppu::new();
        ppu.scanline = 10;
        ppu.oam[0] = 100; // way below target scanline (11)

        ppu.check_sprites();

        assert_eq!(ppu.secondary_oam_count, 0);
    }
}
