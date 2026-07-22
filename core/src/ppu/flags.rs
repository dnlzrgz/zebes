//! Bit masks for the PPU's CPU-visible registers. These sit at $2000 through $2007 in the CPU's
//! address space, but because their addresses are incompletely decoded, they're mirrored every 8 bytes.

//
// Registers offsets ($2000-$2007)
//

pub const REG_PPUCTRL: u16 = 0x0000;
pub const REG_PPUMASK: u16 = 0x0001;
pub const REG_PPUSTATUS: u16 = 0x0002;
pub const REG_OAMADDR: u16 = 0x0003;
pub const REG_OAMDATA: u16 = 0x0004;
pub const REG_PPUSCROLL: u16 = 0x0005;
pub const REG_PPUADDR: u16 = 0x0006;
pub const REG_PPUDATA: u16 = 0x0007;

//
// PPUCTRL - Miscellaneous settings ($2000 write)
//

/// Base nametable selects bit 0.
pub const CTRL_NAMETABLE_LO: u8 = 1 << 0;

/// Base nametable selects bit 1.
pub const CTRL_NAMETABLE_HI: u8 = 1 << 1;

/// VRAM address increment per CPU read/write of PPUDATA.
pub const CTRL_VRAM_INCREMENT: u8 = 1 << 2;

/// Sprite pattern table address for 8x8 sprites.
pub const CTRL_SPRITE_PATTERN_TABLE: u8 = 1 << 3;

/// Background pattern table address (0: $0000; 1: $1000)
pub const CTRL_BACKGROUND_PATTERN_TABLE: u8 = 1 << 4;

/// Sprite size (0: 8x8 pixels; 1: 8x16 pixels).
pub const CTRL_SPRITE_SIZE: u8 = 1 << 5;

/// PPU master/slave select.
pub const CTRL_MASTER_SLAVE: u8 = 1 << 6;

/// Create NMI at the start of a Vblank.
pub const CTRL_NMI_ENABLE: u8 = 1 << 7;

//
// PPUMASK - Rendering settings ($2001 write)
//

/// Greyscale (0: normal color, 1: Greyscale).
pub const MASK_GRAYSCALE: u8 = 1 << 0;

/// Show background in leftmost 8 pixels on screen, 0: hide.
pub const MASK_SHOW_BACKGROUND_LEFT: u8 = 1 << 1;

/// Show sprites in leftmost 8 pixels on screen, 0: hide.
pub const MASK_SHOW_SPRITES_LEFT: u8 = 1 << 2;

/// Enable background rendering.
pub const MASK_SHOW_BACKGROUND: u8 = 1 << 3;

/// Enable sprite rendering.
pub const MASK_SHOW_SPRITES: u8 = 1 << 4;

/// Emphasize red (green on PAL/Dendy).
pub const MASK_EMPHASIZE_RED: u8 = 1 << 5;

/// Emphasize green (red on PAL/Dendy).
pub const MASK_EMPHASIZE_GREEN: u8 = 1 << 6;

/// Emphasize blue.
pub const MASK_EMPHASIZE_BLUE: u8 = 1 << 7;

//
// PPUSTATUS - Rendering events ($2002 read)
//

/// Sprite overflow flag.
pub const STATUS_SPRITE_OVERFLOW: u8 = 1 << 5;

/// Sprite 0 hit flag.
pub const STATUS_SPRITE_ZERO_HIT: u8 = 1 << 6;

/// Vblank flag, cleared on read.
pub const STATUS_VBLANK: u8 = 1 << 7;

#[inline]
pub fn contains(register: u8, flag: u8) -> bool {
    register & flag != 0
}

#[inline]
pub fn set(register: &mut u8, flag: u8, value: bool) {
    if value {
        *register |= flag;
    } else {
        *register &= !flag;
    }
}

#[inline]
pub fn base_nametable(ctrl: u8) -> u8 {
    ctrl & (CTRL_NAMETABLE_LO | CTRL_NAMETABLE_HI)
}

#[inline]
pub fn vram_increment(ctrl: u8) -> u16 {
    if contains(ctrl, CTRL_VRAM_INCREMENT) {
        32
    } else {
        1
    }
}

#[inline]
pub fn background_pattern_table(ctrl: u8) -> u16 {
    if contains(ctrl, CTRL_BACKGROUND_PATTERN_TABLE) {
        0x1000
    } else {
        0x0000
    }
}

#[inline]
pub fn sprite_pattern_table(ctrl: u8) -> u16 {
    if contains(ctrl, CTRL_SPRITE_PATTERN_TABLE) {
        0x1000
    } else {
        0x0000
    }
}

#[inline]
pub fn sprite_height(ctrl: u8) -> u8 {
    if contains(ctrl, CTRL_SPRITE_SIZE) {
        16
    } else {
        8
    }
}

#[inline]
pub fn rendering_enabled(mask: u8) -> bool {
    mask & (MASK_SHOW_BACKGROUND | MASK_SHOW_SPRITES) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_checks_only_the_given_flag() {
        let status = STATUS_VBLANK | STATUS_SPRITE_ZERO_HIT;

        assert!(contains(status, STATUS_VBLANK));
        assert!(contains(status, STATUS_SPRITE_ZERO_HIT));
        assert!(!contains(status, STATUS_SPRITE_OVERFLOW));
    }

    #[test]
    fn set_sets_flags() {
        let mut status = STATUS_SPRITE_ZERO_HIT;

        set(&mut status, STATUS_VBLANK, true);
        assert_eq!(status, STATUS_SPRITE_ZERO_HIT | STATUS_VBLANK);

        set(&mut status, STATUS_SPRITE_ZERO_HIT, false);
        assert_eq!(status, STATUS_VBLANK);
    }

    #[test]
    fn base_nametable_extracts_low_two_bits_only() {
        assert_eq!(base_nametable(0b0000_0000), 0);
        assert_eq!(base_nametable(CTRL_NAMETABLE_LO), 1);
        assert_eq!(base_nametable(CTRL_NAMETABLE_HI), 2);
        assert_eq!(base_nametable(CTRL_NAMETABLE_LO | CTRL_NAMETABLE_HI), 3);
        assert_eq!(base_nametable(CTRL_NMI_ENABLE | CTRL_NAMETABLE_LO), 1);
    }

    #[test]
    fn vram_increment_selects_1_or_32() {
        assert_eq!(vram_increment(0), 1);
        assert_eq!(vram_increment(CTRL_VRAM_INCREMENT), 32);
    }

    #[test]
    fn pattern_table_selectors_choose_0000_or_1000() {
        assert_eq!(background_pattern_table(0), 0x0000);
        assert_eq!(
            background_pattern_table(CTRL_BACKGROUND_PATTERN_TABLE),
            0x1000
        );

        assert_eq!(sprite_pattern_table(0), 0x0000);
        assert_eq!(sprite_pattern_table(CTRL_SPRITE_PATTERN_TABLE), 0x1000);
    }

    #[test]
    fn sprite_height_selects_8_or_16() {
        assert_eq!(sprite_height(0), 8);
        assert_eq!(sprite_height(CTRL_SPRITE_SIZE), 16);
    }

    #[test]
    fn rendering_enabled_true_if_either_layer_on() {
        assert!(!rendering_enabled(0));
        assert!(rendering_enabled(MASK_SHOW_BACKGROUND));
        assert!(rendering_enabled(MASK_SHOW_SPRITES));
        assert!(rendering_enabled(MASK_SHOW_BACKGROUND | MASK_SHOW_SPRITES));
    }
}
