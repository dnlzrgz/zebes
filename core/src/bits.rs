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

#[cfg(test)]
mod tests {
    use super::*;

    const FLAG_A: u8 = 0b0000_0001;
    const FLAG_B: u8 = 0b0000_0010;
    const FLAG_C: u8 = 0b0000_0100;

    #[test]
    fn contains_checks_only_the_given_flag() {
        let register = FLAG_A | FLAG_B;

        assert!(contains(register, FLAG_A));
        assert!(contains(register, FLAG_B));
        assert!(!contains(register, FLAG_C));
    }

    #[test]
    fn set_sets_flags() {
        let mut register = FLAG_B;

        set(&mut register, FLAG_A, true);
        assert_eq!(register, FLAG_A | FLAG_B);

        set(&mut register, FLAG_B, false);
        assert_eq!(register, FLAG_A);
    }

    #[test]
    fn set_is_idempotent() {
        let mut register = FLAG_A;
        set(&mut register, FLAG_A, true);
        set(&mut register, FLAG_A, true);
        assert_eq!(register, FLAG_A);
    }
}
