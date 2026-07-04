//! Processor Status Registers bit masks for the 6502/2A03 CPU.
//! Layout (bit 7 -> bit 0): N V - B D I Z C

/// Indicates that an arithmetic operation produced a carry or borrow.
/// It also stores the bit shifted out by shift and rotate instructions.
pub const CARRY: u8 = 1 << 0;

/// Indicates that the result of the last operation was zero.
pub const ZERO: u8 = 1 << 1;

/// Controls whether IRQ interrupts are accepted.
/// The NMI (Non-Maskable Interrupt) ignores this flag.
pub const INTERRUPT_DISABLE: u8 = 1 << 2;

/// Decimal mode flag. Ignored by the NES 2A03.
/// ADC and SBC always perform binary arithmetic.
pub const DECIMAL: u8 = 1 << 3;

/// Only used in the copy of the status register when pushed onto
/// the stack by certain operations.
pub const BREAK: u8 = 1 << 4;

/// Reserved bit.
/// Always set in status values pushed onto the stack.
pub const UNUSED: u8 = 1 << 5;

/// Indicates signed arithmetic overflow.
pub const OVERFLOW: u8 = 1 << 6;

/// Copies bit 7 of the result.
pub const NEGATIVE: u8 = 1 << 7;

/// Reset status value.
pub const RESET_STATUS: u8 = INTERRUPT_DISABLE | UNUSED;

#[inline]
pub fn contains(status: u8, flag: u8) -> bool {
    status & flag != 0
}

#[inline]
pub fn set(status: &mut u8, flag: u8, value: bool) {
    if value {
        *status |= flag;
    } else {
        *status &= !flag;
    }
}

/// Status byte as it should be pushed to the stack for PHP/BRK:
/// B forced high, unused forced high.
#[inline]
pub fn to_pushed_byte(status: u8) -> u8 {
    status | BREAK | UNUSED
}

/// Status byte as it should be pushed to the stack for a hardware
/// IRQ/NMI: B forced low, unused forced high.
#[inline]
pub fn to_interrupt_pushed_byte(status: u8) -> u8 {
    (status & !BREAK) | UNUSED
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_masks_are_correct() {
        assert_eq!(CARRY, 0b0000_0001);
        assert_eq!(ZERO, 0b0000_0010);
        assert_eq!(INTERRUPT_DISABLE, 0b0000_0100);
        assert_eq!(DECIMAL, 0b0000_1000);
        assert_eq!(BREAK, 0b0001_0000);
        assert_eq!(UNUSED, 0b0010_0000);
        assert_eq!(OVERFLOW, 0b0100_0000);
        assert_eq!(NEGATIVE, 0b1000_0000);
    }

    #[test]
    fn reset_state_is_correct() {
        assert!(contains(RESET_STATUS, INTERRUPT_DISABLE));
        assert!(contains(RESET_STATUS, UNUSED));
        assert!(!contains(RESET_STATUS, CARRY));
        assert!(!contains(RESET_STATUS, ZERO));
        assert!(!contains(RESET_STATUS, DECIMAL));
        assert!(!contains(RESET_STATUS, BREAK));
        assert!(!contains(RESET_STATUS, OVERFLOW));
        assert!(!contains(RESET_STATUS, NEGATIVE));
    }

    #[test]
    fn contains_returns_true_when_flag_is_set() {
        let status = CARRY | ZERO;

        assert!(contains(status, CARRY));
        assert!(contains(status, ZERO));
        assert!(!contains(status, NEGATIVE))
    }

    #[test]
    fn contains_returns_false_when_flag_is_clear() {
        assert!(!contains(0, CARRY));
    }

    #[test]
    fn set_sets_flag() {
        let mut status = 0;
        set(&mut status, CARRY, true);
        assert_eq!(status, CARRY);
    }

    #[test]
    fn set_clears_flag() {
        let mut status = CARRY;
        set(&mut status, CARRY, false);
        assert_eq!(status, 0);
    }

    #[test]
    fn set_does_not_affect_other_flags() {
        let mut status = ZERO;
        set(&mut status, CARRY, true);
        assert_eq!(status, ZERO | CARRY);
    }

    #[test]
    fn set_idempotency() {
        let mut status = CARRY;
        set(&mut status, CARRY, true);
        assert_eq!(status, CARRY);

        let mut status = ZERO;
        set(&mut status, CARRY, false);
        assert_eq!(status, ZERO);
    }

    #[test]
    fn to_pushed_byte_forces_break_and_unused_high() {
        let status = CARRY | ZERO;
        let pushed = to_pushed_byte(status);
        assert!(contains(pushed, BREAK));
        assert!(contains(pushed, UNUSED));
    }

    #[test]
    fn to_pushed_byte_preserves_other_flags() {
        let status = NEGATIVE | OVERFLOW | DECIMAL | INTERRUPT_DISABLE | CARRY | ZERO;
        let pushed = to_pushed_byte(status);
        assert!(contains(pushed, NEGATIVE));
        assert!(contains(pushed, OVERFLOW));
        assert!(contains(pushed, DECIMAL));
        assert!(contains(pushed, INTERRUPT_DISABLE));
        assert!(contains(pushed, CARRY));
        assert!(contains(pushed, ZERO));
    }

    #[test]
    fn to_interrupt_pushed_byte_forces_break_low() {
        let status = CARRY | BREAK;
        let pushed = to_interrupt_pushed_byte(status);
        assert!(!contains(pushed, BREAK));
    }

    #[test]
    fn to_interrupt_pushed_byte_preserves_other_flags() {
        let status = NEGATIVE | OVERFLOW | DECIMAL | INTERRUPT_DISABLE | CARRY | ZERO;
        let pushed = to_interrupt_pushed_byte(status);
        assert!(contains(pushed, NEGATIVE));
        assert!(contains(pushed, OVERFLOW));
        assert!(contains(pushed, DECIMAL));
        assert!(contains(pushed, INTERRUPT_DISABLE));
        assert!(contains(pushed, CARRY));
        assert!(contains(pushed, ZERO));
    }

    #[test]
    fn pushed_and_interrupt_pushed_differ_only_in_break_bit() {
        let status = NEGATIVE | ZERO;

        let pushed = to_pushed_byte(status);
        let interrupt_pushed = to_interrupt_pushed_byte(status);

        // Every bit except BREAK should match between the two.
        // BREAK should be set in one, clear in the other.
        assert_eq!(pushed & !BREAK, interrupt_pushed & !BREAK);
        assert_ne!(pushed & BREAK, interrupt_pushed & BREAK);
    }
}
