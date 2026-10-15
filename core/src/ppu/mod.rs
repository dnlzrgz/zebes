mod background;
mod flags;
mod palette;
pub mod ppu_bus;
mod sprites;
#[cfg(test)]
mod test_utils;

use crate::bits::{contains, set};
use crate::ppu::flags::*;
use crate::ppu::palette::Rgb;
use crate::ppu::ppu_bus::PpuBus;

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;

pub type Framebuffer = [Rgb; SCREEN_WIDTH * SCREEN_HEIGHT];

/// Models the Ricoh 2C02.
pub struct Ppu {
    /// CPU-visible registers: $2000, $2001, $2002.
    ctrl: u8,
    mask: u8,
    status: u8,

    /// Sprite memory address ($2003) and OAM contents.
    oam_address: u8,
    oam: [u8; 256],

    /// Current VRAM address (15 bits).
    v: u16,

    /// Temporary VRAM address (15 bits).
    t: u16,

    /// Fine X scroll (3 bits).
    x: u8,

    /// Shred write toggle for $2005 and $2006.
    /// False = first write; true = second write.
    w: bool,

    /// Buffer for PPUDATA.
    read_buffer: u8,

    /// Set when the PPU enters vertical blank and NMI is requested.
    nmi_requested: bool,

    /// Current scanline (0..=239 = visible, 240 = post-render, 241..=260 =
    /// vertical blank, 261 = pre-render).
    scanline: u16,

    /// Current cycle within scanline (0..=340)
    cycle: u16,

    /// Number of frames rendered.
    frame: u64,

    bg_pattern_shift_lo: u16,
    bg_pattern_shift_hi: u16,
    bg_attr_shift_lo: u16,
    bg_attr_shift_hi: u16,

    next_tile_id: u8,
    next_tile_attr: u8,
    next_tile_lo: u8,
    next_tile_hi: u8,

    secondary_oam: [u8; 32],
    secondary_oam_count: u8,

    sprite_zero_in_range: bool,
    sprite_pattern_lo: [u8; 8],
    sprite_pattern_hi: [u8; 8],
    sprite_attr: [u8; 8],
    sprite_x_counter: [u8; 8],

    frame_buffer: Box<Framebuffer>,

    // PPU-owned bus.
    bus: PpuBus,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_address: 0,
            oam: [0; 256],
            v: 0,
            t: 0,
            x: 0,
            w: false,
            read_buffer: 0,
            nmi_requested: false,
            scanline: 0,
            cycle: 0,
            frame: 0,
            bg_pattern_shift_lo: 0,
            bg_pattern_shift_hi: 0,
            bg_attr_shift_lo: 0,
            bg_attr_shift_hi: 0,
            next_tile_id: 0,
            next_tile_attr: 0,
            next_tile_lo: 0,
            next_tile_hi: 0,
            secondary_oam: [0xFF; 32],
            secondary_oam_count: 0,
            sprite_zero_in_range: false,
            sprite_pattern_lo: [0; 8],
            sprite_pattern_hi: [0; 8],
            sprite_attr: [0; 8],
            sprite_x_counter: [0xFF; 8],
            frame_buffer: Box::new([Rgb::default(); SCREEN_WIDTH * SCREEN_HEIGHT]),
            bus: PpuBus::new(),
        }
    }
}

impl Ppu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bus(mut self, bus: PpuBus) -> Self {
        self.bus = bus;
        self
    }

    /// Advance the PPU by one cycle.
    pub fn clock(&mut self) {
        if rendering_enabled(self.mask) && (self.scanline < 240 || self.scanline == 261) {
            if (1..=256).contains(&self.cycle) || (321..=336).contains(&self.cycle) {
                if self.scanline < 240 && (1..=256).contains(&self.cycle) {
                    self.render_pixel();
                    self.tick_sprites();
                }
                self.shift_register();
                self.fetch_background_tile();
            }

            if self.cycle == 256 {
                self.increment_y();
                self.check_sprites();
            }

            if self.cycle == 257 {
                self.copy_horizontal_bits();
            }

            if (257..=320).contains(&self.cycle) {
                self.oam_address = 0;
            }

            if self.cycle == 320 {
                self.fetch_sprites();
            }

            if self.scanline == 261 && (280..=304).contains(&self.cycle) {
                self.copy_vertical_bits();
            }
        }

        if self.scanline == 241 && self.cycle == 1 {
            set(&mut self.status, STATUS_VBLANK, true);
            if contains(self.ctrl, CTRL_NMI_ENABLE) {
                self.nmi_requested = true;
            }
        }

        if self.scanline == 261 && self.cycle == 1 {
            set(&mut self.status, STATUS_VBLANK, false);
            set(&mut self.status, STATUS_SPRITE_ZERO_HIT, false);
            set(&mut self.status, STATUS_SPRITE_OVERFLOW, false);
        }

        // NTSC odd-frame skip
        if self.scanline == 261
            && self.cycle == 339
            && rendering_enabled(self.mask)
            && !self.frame.is_multiple_of(2)
        {
            self.cycle = 0;
            self.scanline = 0;
            self.frame = self.frame.wrapping_add(1);
            return;
        }

        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
                self.frame = self.frame.wrapping_add(1);
            }
        }
    }

    /// Return and clear a pending NMI request.
    pub fn take_nmi(&mut self) -> bool {
        let requested = self.nmi_requested;
        self.nmi_requested = false;
        requested
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address & 0x0007 {
            flags::REG_PPUCTRL => 0, // write-only
            flags::REG_PPUMASK => 0, // write-only
            flags::REG_PPUSTATUS => {
                let value = self.status;
                set(&mut self.status, STATUS_VBLANK, false);
                self.w = false;
                value
            }
            flags::REG_OAMADDR => 0, // write-only
            flags::REG_OAMDATA => self.oam[self.oam_address as usize],
            flags::REG_PPUSCROLL => 0, // write-only
            flags::REG_PPUADDR => 0,   // write-only
            flags::REG_PPUDATA => {
                let result = if self.v >= 0x3F00 {
                    // Bypass the buffer and returns immediately.
                    self.read_buffer = self.bus.read(self.v - 0x1000);
                    self.bus.read(self.v)
                } else {
                    let buffered = self.read_buffer;
                    self.read_buffer = self.bus.read(self.v);
                    buffered
                };

                self.v = self.v.wrapping_add(vram_increment(self.ctrl));
                result
            }
            _ => unreachable!("address & 0x0007 is always in 0..=7"),
        }
    }

    pub fn cpu_peek(&self, address: u16) -> u8 {
        match address & 0x0007 {
            REG_PPUSTATUS => self.status,
            REG_OAMDATA => self.oam[self.oam_address as usize],
            _ => 0,
        }
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address & 0x0007 {
            REG_PPUCTRL => {
                self.ctrl = data;
                self.t = (self.t & !0x0C00) | ((data as u16 & 0x03) << 10);
            }
            REG_PPUMASK => self.mask = data,
            REG_PPUSTATUS => {} // read-only
            REG_OAMADDR => self.oam_address = data,
            REG_OAMDATA => {
                self.oam[self.oam_address as usize] = data;
                self.oam_address = self.oam_address.wrapping_add(1);
            }
            REG_PPUSCROLL => {
                if !self.w {
                    // First write
                    self.x = data & 0x07;
                    self.t = (self.t & !0x001F) | (data as u16 >> 3);
                } else {
                    // Second write
                    self.t = (self.t & !0x73E0)
                        | ((data as u16 & 0x07) << 12)
                        | ((data as u16 & 0xF8) << 2);
                }

                self.w = !self.w;
            }
            REG_PPUADDR => {
                if !self.w {
                    // First write
                    self.t = (self.t & 0x00FF) | ((data as u16 & 0x3F) << 8);
                } else {
                    // Second write
                    self.t = (self.t & 0xFF00) | data as u16;
                    self.v = self.t;
                }

                self.w = !self.w;
            }
            REG_PPUDATA => {
                self.bus.write(self.v, data);
                self.v = self.v.wrapping_add(vram_increment(self.ctrl));
            }
            _ => unreachable!("address & 0x0007 is always in 0..=7"),
        }
    }

    pub fn scanline(&self) -> u16 {
        self.scanline
    }

    pub fn cycle(&self) -> u16 {
        self.cycle
    }

    pub fn frame(&self) -> u64 {
        self.frame
    }

    pub fn framebuffer(&self) -> &Framebuffer {
        &self.frame_buffer
    }

    pub fn ctrl(&self) -> u8 {
        self.ctrl
    }

    pub fn mask(&self) -> u8 {
        self.mask
    }

    pub fn status(&self) -> u8 {
        self.status
    }

    pub fn v(&self) -> u16 {
        self.v
    }

    pub fn t(&self) -> u16 {
        self.t
    }

    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn w(&self) -> bool {
        self.w
    }

    pub fn oam(&self) -> &[u8; 256] {
        &self.oam
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_nmi_returns_and_clears_pending_request() {
        let mut ppu = Ppu::new();
        ppu.nmi_requested = true;

        assert!(ppu.take_nmi());
        assert!(!ppu.take_nmi()); // must return cleared flag
    }

    #[test]
    fn cpu_read_ppustatus_clears_vblank_and_w_toggle() {
        let mut ppu = Ppu::new();
        ppu.status = STATUS_VBLANK | STATUS_SPRITE_OVERFLOW;
        ppu.w = true;

        let value = ppu.cpu_read(REG_PPUSTATUS);

        assert_eq!(value, STATUS_VBLANK | STATUS_SPRITE_OVERFLOW);
        assert!(!contains(ppu.status, STATUS_VBLANK));
        assert!(contains(ppu.status, STATUS_SPRITE_OVERFLOW));
        assert!(!ppu.w);
    }

    #[test]
    fn cpu_read_mirrors_registers_every_eight_bytes() {
        let mut ppu = Ppu::new();
        ppu.status = STATUS_VBLANK;
        ppu.w = true;

        let value = ppu.cpu_read(0x3FFA); // 0x3FFA & 7 == 2 -> PPUSTATUS

        assert_eq!(value, STATUS_VBLANK);
        assert!(!contains(ppu.status, STATUS_VBLANK));
        assert!(!ppu.w);
    }

    #[test]
    fn cpu_read_oamdata_returns_current_oam_entry_without_incrementing() {
        let mut ppu = Ppu::new();
        ppu.oam_address = 7;
        ppu.oam[7] = 0x5A;

        assert_eq!(ppu.cpu_read(REG_OAMDATA), 0x5A);
        assert_eq!(ppu.oam_address, 7);
    }

    #[test]
    fn cpu_read_ppudata_buffers_and_increments_by_one() {
        let mut ppu = Ppu::new();
        ppu.ctrl = 0;
        ppu.read_buffer = 0xAA;
        ppu.v = 0x2000;
        ppu.bus.write(0x2000, 0xBB);

        let value = ppu.cpu_read(REG_PPUDATA);

        assert_eq!(value, 0xAA); // Buffered value
        assert_eq!(ppu.read_buffer, 0xBB); // Next byte
        assert_eq!(ppu.v, 0x2001);
    }

    #[test]
    fn cpu_read_ppudata_increments_by_32_when_ctrl_bit_set() {
        let mut ppu = Ppu::new();
        ppu.ctrl = CTRL_VRAM_INCREMENT;
        ppu.v = 0x2000;

        ppu.cpu_read(REG_PPUDATA);

        assert_eq!(ppu.v, 0x2020);
    }

    #[test]
    fn cpu_read_ppudata_palette_address_bypasses_buffer() {
        let mut ppu = Ppu::new();
        ppu.ctrl = 0;
        ppu.v = 0x3F00;
        ppu.bus.write(0x2F00, 0xAA);
        ppu.bus.write(0x3F00, 0xCC);

        let value = ppu.cpu_read(REG_PPUDATA);

        assert_eq!(value, 0xCC);
        assert_eq!(ppu.read_buffer, 0xAA);
        assert_eq!(ppu.v, 0x3F01);
    }

    #[test]
    fn cpu_read_ppudata_bypasses_buffer_across_palette_range() {
        for v in [0x3F00u16, 0x3F01, 0x3F10, 0x3F11, 0x3F20, 0x3F1F] {
            let mut ppu = Ppu::new();
            ppu.ctrl = 0;
            ppu.v = v;
            ppu.bus.write(v - 0x1000, 0xAA);
            ppu.bus.write(v, 0xCC);

            let value = ppu.cpu_read(REG_PPUDATA);

            assert_eq!(value, 0xCC, "v = ${v:04X}");
            assert_eq!(ppu.read_buffer, 0xAA, "v = ${v:04X}");
            assert_eq!(ppu.v, v + 1, "v = ${v:04X}");
        }
    }

    #[test]
    fn cpu_peek_ppustatus_does_not_clear_flags_or_w_toggle() {
        let mut ppu = Ppu::new();
        ppu.status = STATUS_VBLANK | STATUS_SPRITE_ZERO_HIT;
        ppu.w = true;

        let value = ppu.cpu_peek(REG_PPUSTATUS);

        assert_eq!(value, STATUS_VBLANK | STATUS_SPRITE_ZERO_HIT);
        assert!(contains(ppu.status, STATUS_VBLANK));
        assert!(contains(ppu.status, STATUS_SPRITE_ZERO_HIT));
        assert!(ppu.w);
    }

    #[test]
    fn cpu_peek_oamdata_reads_current_oam_entry() {
        let mut ppu = Ppu::new();
        ppu.oam_address = 3;
        ppu.oam[3] = 0x99;

        assert_eq!(ppu.cpu_peek(REG_OAMDATA), 0x99);
    }

    #[test]
    fn cpu_write_ppuctrl_updates_ctrl_and_t_nametable_bits() {
        let mut ppu = Ppu::new();
        ppu.t = 0;

        ppu.cpu_write(REG_PPUCTRL, 0b11);

        assert_eq!(ppu.ctrl, 0b11);
        assert_eq!(ppu.t & 0x0C00, 0x0C00);
    }

    #[test]
    fn cpu_write_ppuctrl_masks_nametable_bits_and_preserves_other_t_bits() {
        let mut ppu = Ppu::new();
        ppu.t = 0x7FFF; // nametable bits and unrelated scroll bits set

        ppu.cpu_write(REG_PPUCTRL, 0x81); // only low 2 bits feed the nametable field

        assert_eq!(ppu.ctrl, 0x81); // the full byte is still stored in ctrl
        assert_eq!(ppu.t & 0x0C00, 0x0400); // 0b01 -> horizontal nametable bit
        assert_eq!(ppu.t & !0x0C00, 0x73FF); // unrelated bits are keep
    }

    #[test]
    fn cpu_write_oamdata_writes_and_increments_oam_address() {
        let mut ppu = Ppu::new();
        ppu.oam_address = 0x10;

        ppu.cpu_write(REG_OAMDATA, 0xAB);

        assert_eq!(ppu.oam[0x10], 0xAB);
        assert_eq!(ppu.oam_address, 0x11);
    }

    #[test]
    fn cpu_write_oamdata_wraps_oam_address() {
        let mut ppu = Ppu::new();
        ppu.oam_address = 0xFF;

        ppu.cpu_write(REG_OAMDATA, 0xCD);

        assert_eq!(ppu.oam[0xFF], 0xCD);
        assert_eq!(ppu.oam_address, 0x00);
    }

    #[test]
    fn cpu_write_ppuscroll_first_write_sets_fine_x_and_coarse_x() {
        let mut ppu = Ppu::new();
        ppu.t = 0x7FE0;

        ppu.cpu_write(REG_PPUSCROLL, 0b0010_1011);

        assert_eq!(ppu.x, 0b011); // low 3 bits
        assert_eq!(ppu.t, 0x7FE5); // high 5 bits become coarse X
        assert!(ppu.w);
    }

    #[test]
    fn cpu_write_ppuscroll_second_write_sets_fine_y_and_coarse_y() {
        let mut ppu = Ppu::new();
        ppu.t = 0x041F;
        ppu.w = true;

        ppu.cpu_write(REG_PPUSCROLL, 0b0100_1000);

        assert_eq!((ppu.t >> 12) & 0x07, 0b000); // fine Y = low 3 bits
        assert_eq!((ppu.t & 0x03E0) >> 5, 0b01001); // coarse Y = high 5 bits
        assert_eq!(ppu.t & 0x041F, 0x041F); // unrelated horizontal bits are keep
        assert!(!ppu.w);
    }

    #[test]
    fn cpu_write_ppuscroll_does_not_touch_v() {
        let mut ppu = Ppu::new();
        ppu.v = 0x1234;
        ppu.t = 0;

        ppu.cpu_write(REG_PPUSCROLL, 0xFF);
        assert_eq!(ppu.v, 0x1234);

        ppu.cpu_write(REG_PPUSCROLL, 0xFF);
        assert_eq!(ppu.v, 0x1234);
    }

    #[test]
    fn cpu_write_scroll_then_addr_uses_shared_w_latch() {
        let mut ppu = Ppu::new();

        ppu.cpu_write(REG_PPUSCROLL, 0xAB); // first scroll write -> w = true
        ppu.cpu_write(REG_PPUADDR, 0x0F); // treated as the second PPUADDR write

        assert_eq!(ppu.v, 0x000F);
        assert!(!ppu.w);
    }

    #[test]
    fn cpu_write_ppuaddr_first_write_sets_t_high_byte() {
        let mut ppu = Ppu::new();
        ppu.t = 0x00FF;

        ppu.cpu_write(REG_PPUADDR, 0x3F);

        assert_eq!(ppu.t, 0x3FFF);
        assert!(ppu.w);
    }

    #[test]
    fn cpu_write_ppuaddr_first_write_masks_high_address_bits() {
        let mut ppu = Ppu::new();
        ppu.t = 0x00FF;

        ppu.cpu_write(REG_PPUADDR, 0xFF);

        assert_eq!(ppu.t, 0x3FFF); // bits 6-7 are ignored
        assert!(ppu.w);
    }

    #[test]
    fn cpu_write_ppuaddr_second_write_sets_low_byte_and_copies_t_to_v() {
        let mut ppu = Ppu::new();
        ppu.t = 0x2000;
        ppu.w = true;

        ppu.cpu_write(REG_PPUADDR, 0x0F);

        assert_eq!(ppu.t, 0x200F);
        assert_eq!(ppu.v, 0x200F);
        assert!(!ppu.w);
    }

    #[test]
    fn cpu_write_ppudata_writes_and_increments_v_by_one() {
        let mut ppu = Ppu::new();
        ppu.ctrl = 0;
        ppu.v = 0x2000;

        ppu.cpu_write(REG_PPUDATA, 0xAB);

        assert_eq!(ppu.bus.read(0x2000), 0xAB);
        assert_eq!(ppu.v, 0x2001);
    }

    #[test]
    fn cpu_write_ppudata_increments_v_by_32_when_ctrl_bit_set() {
        let mut ppu = Ppu::new();
        ppu.ctrl = CTRL_VRAM_INCREMENT;
        ppu.v = 0x2000;

        ppu.cpu_write(REG_PPUDATA, 0xAB);

        assert_eq!(ppu.v, 0x2020);
    }

    #[test]
    fn clock_sets_vblank_and_requests_nmi_when_enabled() {
        let mut ppu = Ppu::new();
        ppu.scanline = 241;
        ppu.cycle = 1;
        ppu.mask = 0; // keeps test focused on VBlank
        ppu.ctrl = CTRL_NMI_ENABLE;

        ppu.clock();

        assert!(contains(ppu.status, STATUS_VBLANK));
        assert!(ppu.take_nmi());
    }

    #[test]
    fn clock_does_not_request_nmi_when_disabled() {
        let mut ppu = Ppu::new();
        ppu.scanline = 241;
        ppu.cycle = 1;
        ppu.mask = 0;
        ppu.ctrl = 0;

        ppu.clock();

        assert!(contains(ppu.status, STATUS_VBLANK));
        assert!(!ppu.take_nmi());
    }

    #[test]
    fn clock_clears_status_flags_at_pre_render_start() {
        let mut ppu = Ppu::new();
        ppu.scanline = 261;
        ppu.cycle = 1;
        ppu.mask = 0;
        ppu.status = STATUS_VBLANK | STATUS_SPRITE_ZERO_HIT | STATUS_SPRITE_OVERFLOW;

        ppu.clock();

        assert!(!contains(ppu.status, STATUS_VBLANK));
        assert!(!contains(ppu.status, STATUS_SPRITE_ZERO_HIT));
        assert!(!contains(ppu.status, STATUS_SPRITE_OVERFLOW));
    }

    #[test]
    fn clock_skips_last_cycle_on_odd_frames_when_rendering() {
        let mut ppu = Ppu::new();
        ppu.scanline = 261;
        ppu.cycle = 339;
        ppu.frame = 1;
        ppu.mask = MASK_SHOW_BACKGROUND; // rendering enabled

        ppu.clock();

        assert_eq!(ppu.cycle, 0);
        assert_eq!(ppu.scanline, 0);
        assert_eq!(ppu.frame, 2);
    }

    #[test]
    fn clock_does_not_skip_last_cycle_on_even_frames() {
        let mut ppu = Ppu::new();
        ppu.scanline = 261;
        ppu.cycle = 339;
        ppu.frame = 0;
        ppu.mask = MASK_SHOW_BACKGROUND;

        ppu.clock();

        assert_eq!(ppu.cycle, 340);
        assert_eq!(ppu.scanline, 261);
        assert_eq!(ppu.frame, 0);
    }

    #[test]
    fn clock_advances_cycle_within_scanline() {
        let mut ppu = Ppu::new();
        ppu.scanline = 0;
        ppu.cycle = 0;
        ppu.mask = 0; // keeps test focused on cycle bookkeeping

        ppu.clock();

        assert_eq!(ppu.cycle, 1);
        assert_eq!(ppu.scanline, 0);
        assert_eq!(ppu.frame, 0);
    }

    #[test]
    fn clock_wraps_cycle_to_next_scanline() {
        let mut ppu = Ppu::new();
        ppu.scanline = 0;
        ppu.cycle = 340;
        ppu.mask = 0;

        ppu.clock();

        assert_eq!(ppu.cycle, 0);
        assert_eq!(ppu.scanline, 1);
    }

    #[test]
    fn clock_wraps_scanline_and_increments_frame() {
        let mut ppu = Ppu::new();
        ppu.scanline = 261;
        ppu.cycle = 340;
        ppu.mask = 0;

        ppu.clock();

        assert_eq!(ppu.cycle, 0);
        assert_eq!(ppu.scanline, 0);
        assert_eq!(ppu.frame, 1);
    }

    #[test]
    fn clock_increments_vertical_scroll_at_cycle_256() {
        let mut ppu = Ppu::new();
        ppu.scanline = 0;
        ppu.cycle = 256;
        ppu.mask = MASK_SHOW_BACKGROUND;
        ppu.v = 0x0000; // fine Y = 0

        ppu.clock();

        assert_eq!((ppu.v >> 12) & 0x07, 1); // fine Y bumped
    }

    #[test]
    fn clock_copies_horizontal_bits_at_cycle_257() {
        let mut ppu = Ppu::new();
        ppu.scanline = 0;
        ppu.cycle = 257;
        ppu.mask = MASK_SHOW_BACKGROUND;
        ppu.v = 0x7BE0;
        ppu.t = 0x041F;

        ppu.clock();

        assert_eq!(ppu.v, 0x7FFF);
    }

    #[test]
    fn clock_copies_vertical_bits_on_pre_render_line() {
        let mut ppu = Ppu::new();
        ppu.scanline = 261;
        ppu.cycle = 280;
        ppu.mask = MASK_SHOW_BACKGROUND;
        ppu.v = 0x041F;
        ppu.t = 0x7BE0;

        ppu.clock();

        assert_eq!(ppu.v, 0x7FFF);
    }

    #[test]
    fn clock_does_not_update_v_when_rendering_disabled() {
        let cases = [
            (0u16, 256u16, 0x0000u16, 0x7FFF), // bump fine Y
            (0, 257, 0x0000, 0x041F),          // copy horizontal bits
            (261, 280, 0x041F, 0x7BE0),        // copy vertical bits
        ];

        for (scanline, cycle, v, t) in cases {
            let mut ppu = Ppu::new();
            ppu.scanline = scanline;
            ppu.cycle = cycle;
            ppu.mask = 0; // rendering disabled
            ppu.v = v;
            ppu.t = t;

            ppu.clock();

            assert_eq!(ppu.v, v, "scanline {scanline} cycle {cycle}");
        }
    }

    #[test]
    fn frame_has_341_times_262_cycles_when_rendering_disabled() {
        let mut ppu = Ppu::new();
        ppu.mask = 0;

        for _ in 0..(341 * 262) {
            ppu.clock();
        }

        assert_eq!(ppu.frame, 1);
        assert_eq!(ppu.scanline, 0);
        assert_eq!(ppu.cycle, 0);
    }

    #[test]
    fn odd_frame_skips_one_cycle_when_rendering_enabled() {
        let mut ppu = Ppu::new();
        ppu.mask = MASK_SHOW_BACKGROUND;
        ppu.frame = 1; // odd frame skips cycle 339 on the pre-render line

        for _ in 0..(341 * 262 - 1) {
            ppu.clock();
        }

        assert_eq!(ppu.frame, 2);
        assert_eq!(ppu.scanline, 0);
        assert_eq!(ppu.cycle, 0);
    }
}
