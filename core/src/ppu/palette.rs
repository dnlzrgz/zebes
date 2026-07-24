/// RGB color, produced by mapping a 6-bit NES palette index throught the hardware color table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub const PALETTE: [Rgb; 64] = [
    // 0x00
    Rgb {
        r: 0x54,
        g: 0x54,
        b: 0x54,
    },
    Rgb {
        r: 0x00,
        g: 0x1E,
        b: 0x74,
    },
    Rgb {
        r: 0x08,
        g: 0x10,
        b: 0x90,
    },
    Rgb {
        r: 0x30,
        g: 0x00,
        b: 0x88,
    }, // $00-$03
    Rgb {
        r: 0x44,
        g: 0x00,
        b: 0x64,
    },
    Rgb {
        r: 0x5C,
        g: 0x00,
        b: 0x30,
    },
    Rgb {
        r: 0x54,
        g: 0x04,
        b: 0x00,
    },
    Rgb {
        r: 0x3C,
        g: 0x18,
        b: 0x00,
    }, // $04-$07
    Rgb {
        r: 0x20,
        g: 0x2A,
        b: 0x00,
    },
    Rgb {
        r: 0x08,
        g: 0x3A,
        b: 0x00,
    },
    Rgb {
        r: 0x00,
        g: 0x40,
        b: 0x00,
    },
    Rgb {
        r: 0x00,
        g: 0x3C,
        b: 0x00,
    }, // $08-$0B
    Rgb {
        r: 0x00,
        g: 0x32,
        b: 0x3C,
    },
    Rgb {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Rgb {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Rgb {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }, // $0C-$0F
    // 0x10
    Rgb {
        r: 0x98,
        g: 0x96,
        b: 0x98,
    },
    Rgb {
        r: 0x08,
        g: 0x4C,
        b: 0xC4,
    },
    Rgb {
        r: 0x30,
        g: 0x32,
        b: 0xEC,
    },
    Rgb {
        r: 0x5C,
        g: 0x1E,
        b: 0xE4,
    }, // $10-$13
    Rgb {
        r: 0x88,
        g: 0x14,
        b: 0xB0,
    },
    Rgb {
        r: 0xA0,
        g: 0x14,
        b: 0x64,
    },
    Rgb {
        r: 0x98,
        g: 0x22,
        b: 0x20,
    },
    Rgb {
        r: 0x78,
        g: 0x3C,
        b: 0x00,
    }, // $14-$17
    Rgb {
        r: 0x54,
        g: 0x5A,
        b: 0x00,
    },
    Rgb {
        r: 0x28,
        g: 0x72,
        b: 0x00,
    },
    Rgb {
        r: 0x08,
        g: 0x7C,
        b: 0x00,
    },
    Rgb {
        r: 0x00,
        g: 0x76,
        b: 0x28,
    }, // $18-$1B
    Rgb {
        r: 0x00,
        g: 0x66,
        b: 0x78,
    },
    Rgb {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Rgb {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Rgb {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }, // $1C-$1F
    // 0x20
    Rgb {
        r: 0xEC,
        g: 0xEE,
        b: 0xEC,
    },
    Rgb {
        r: 0x4C,
        g: 0x9A,
        b: 0xEC,
    },
    Rgb {
        r: 0x78,
        g: 0x7C,
        b: 0xEC,
    },
    Rgb {
        r: 0xB0,
        g: 0x62,
        b: 0xEC,
    }, // $20-$23
    Rgb {
        r: 0xE4,
        g: 0x54,
        b: 0xEC,
    },
    Rgb {
        r: 0xEC,
        g: 0x58,
        b: 0xB4,
    },
    Rgb {
        r: 0xEC,
        g: 0x6A,
        b: 0x64,
    },
    Rgb {
        r: 0xD4,
        g: 0x88,
        b: 0x20,
    }, // $24-$27
    Rgb {
        r: 0xA0,
        g: 0xAA,
        b: 0x00,
    },
    Rgb {
        r: 0x74,
        g: 0xC4,
        b: 0x00,
    },
    Rgb {
        r: 0x4C,
        g: 0xD0,
        b: 0x20,
    },
    Rgb {
        r: 0x38,
        g: 0xCC,
        b: 0x6C,
    }, // $28-$2B
    Rgb {
        r: 0x38,
        g: 0xB4,
        b: 0xCC,
    },
    Rgb {
        r: 0x3C,
        g: 0x3C,
        b: 0x3C,
    },
    Rgb {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Rgb {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }, // $2C-$2F
    // 0x30
    Rgb {
        r: 0xEC,
        g: 0xEE,
        b: 0xEC,
    },
    Rgb {
        r: 0xA8,
        g: 0xCC,
        b: 0xEC,
    },
    Rgb {
        r: 0xBC,
        g: 0xBC,
        b: 0xEC,
    },
    Rgb {
        r: 0xD4,
        g: 0xB2,
        b: 0xEC,
    }, // $30-$33
    Rgb {
        r: 0xEC,
        g: 0xAE,
        b: 0xEC,
    },
    Rgb {
        r: 0xEC,
        g: 0xAE,
        b: 0xD4,
    },
    Rgb {
        r: 0xEC,
        g: 0xB4,
        b: 0xB0,
    },
    Rgb {
        r: 0xE4,
        g: 0xC4,
        b: 0x90,
    }, // $34-$37
    Rgb {
        r: 0xCC,
        g: 0xD2,
        b: 0x78,
    },
    Rgb {
        r: 0xB4,
        g: 0xDE,
        b: 0x78,
    },
    Rgb {
        r: 0xA8,
        g: 0xE2,
        b: 0x90,
    },
    Rgb {
        r: 0x98,
        g: 0xE2,
        b: 0xB4,
    }, // $38-$3B
    Rgb {
        r: 0xA0,
        g: 0xD6,
        b: 0xE4,
    },
    Rgb {
        r: 0xA0,
        g: 0xA2,
        b: 0xA0,
    },
    Rgb {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Rgb {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }, // $3C-$3F
];
