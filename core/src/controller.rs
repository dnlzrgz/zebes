//! Standard NES controlller button bits, in the order they are supposed to be shifted out on read.
pub const BUTTON_A: u8 = 1 << 0;
pub const BUTTON_B: u8 = 1 << 1;
pub const BUTTON_SELECT: u8 = 1 << 2;
pub const BUTTON_START: u8 = 1 << 3;
pub const BUTTON_UP: u8 = 1 << 4;
pub const BUTTON_DOWN: u8 = 1 << 5;
pub const BUTTON_LEFT: u8 = 1 << 6;
pub const BUTTON_RIGHT: u8 = 1 << 7;
