mod flags;
pub mod ppu_bus;

use ppu_bus::PpuBus;

use crate::ppu::flags::*;

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

    pub fn clock(&mut self) {
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
}
