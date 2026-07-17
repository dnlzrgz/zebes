use crate::{
    cpu::{Cpu, addressing::Operand, flags::*},
    cpu_bus::CpuBus,
};

impl Cpu {
    /// Branch if Carry Clear
    /// PC = PC + 2 memory (signed)
    ///
    /// If the carry flag is clear, BCC branches to a nearby location by adding the relative offset
    /// to the program counter. The offset is signed and has a range of [-128, 127] relative to the
    /// first byte *after* the branch instruction.
    /// The carry flag has different meanings depending on the context. BCC can be used after a
    /// compare to branch if the register is less than the memory value, so it is sometimes called
    /// BLT for Branch if Less Than. It can also be used after SBC to branch if the unsigned value
    /// underflowed or after ADC to branch if it did not overflow.
    pub fn bcc(&mut self, operand: Operand, _: &mut CpuBus) -> u8 {
        self.branch_if(!contains(self.status, CARRY), operand)
    }

    /// Branch if Carry Set
    /// PC = PC + 2 + memory (signed)
    ///
    /// If the carry flag is set, BCS branches to a nearby location by adding the branch offset to
    /// the program counter. The offset is signed and has a range of [-128, 127] relative to the
    /// first byte *after* the branch instruction.
    /// The carry flag has different meanings depending on the context. BCS can be used after a
    /// compare to branch if the register is greater than or equal to the memory value, so it is
    /// sometimes called BGE for Branch if Greater Than or Equal. It can also be used after ADC to
    /// branch if the usigned value overflowed or after SBC to branch if it did not underflow.
    pub fn bcs(&mut self, operand: Operand, _: &mut CpuBus) -> u8 {
        self.branch_if(contains(self.status, CARRY), operand)
    }

    /// Branch if Equal
    /// PC = PC + 2 + memory (signed)
    ///
    /// If the zero flag is set, BEQ branches to a nearby location by adding the branch offset to
    /// the program counter. The offset is signed and has a range of [-128, 127] relative to the
    /// first byte *after* the branch instruction.
    /// Comparison uses this flag to indicate if the compared values are equal. All instructions
    /// that change A, X, or Y also implicitly set or clear the zero flag depending on whether the
    /// register becomes 0.
    pub fn beq(&mut self, operand: Operand, _: &mut CpuBus) -> u8 {
        self.branch_if(contains(self.status, ZERO), operand)
    }

    /// Branch if Not Equal
    /// PC = PC + 2 + memory (signed)
    ///
    /// If the zero flag is clear, BNE branches to a nearby location by adding the branch offset to
    /// the program counter. The offset is signed and has a range of [-128, 127] relative to the
    /// first byte *after* the branch instruction.
    /// Comparison uses this flag to indicate if the compared values are equal. All instructions
    /// that change A, X, or Y also implicitly set or clear the zero flag depending on whether the
    /// register becomes 0.
    pub fn bne(&mut self, operand: Operand, _: &mut CpuBus) -> u8 {
        self.branch_if(!contains(self.status, ZERO), operand)
    }

    /// Branch if Plus
    /// PC = PC + 2 + memory (signed)
    ///
    /// If the negative flag is clear, BPL branches to a nearby location by adding the branch offset
    /// to the program counter. The offset is signed and has a range of [-128, 127] relative to the
    /// first byte *after* the branch instruction.
    /// All instructions that change A, X, or Y implicitly set or clear the negative flag based on
    /// bit 7 (the sign bit).
    pub fn bpl(&mut self, operand: Operand, _: &mut CpuBus) -> u8 {
        self.branch_if(!contains(self.status, NEGATIVE), operand)
    }

    /// Branch if Minus
    /// PC = PC + 2 + memory (signed)
    ///
    /// If the negative flag is set, BMI branches to a nearby location by adding the branch offset
    /// to the program counter. The offset is signed and has a range of [-128, 127] relative to the
    /// first byte *after* the branch instructions.
    /// All instructions that change A, X, or Y implicitly set or clear the negative flag based on
    /// bit 7 (the sign bit).
    pub fn bmi(&mut self, operand: Operand, _: &mut CpuBus) -> u8 {
        self.branch_if(contains(self.status, NEGATIVE), operand)
    }

    /// Branch if Overflow Clear
    /// PC = PC + 2 + memory (signed)
    ///
    /// If the overflowing flag is clear, BVC branches to a nearby location by adding the branch
    /// offset to the program counter. The offset is signed and has a range of [-128, 127] relative
    /// to the first byte after the branch instruction.
    /// Unlike zero, negative, and even carry, overflow is modified by very few instructions. It is
    /// most often used with the BIT instruction, particularly for polling hardware registers. It is
    /// also sometimes used for signed overflow with ADC and SBC. The standard 6502 chip allows an
    /// external device to set overflow usign a pin, enabling software to poll for that event, but
    /// this is not present on the NES' 2A03.
    pub fn bvc(&mut self, operand: Operand, _: &mut CpuBus) -> u8 {
        self.branch_if(!contains(self.status, OVERFLOW), operand)
    }

    /// Branch if Overflow Set
    /// PC = PC + 2 + memory (signed)
    ///
    /// If the overflow flag is set, BVS branches to a nearby location by adding the branch offset
    /// to the program counter. The offset is signed and has a range of [-128, 127] relative to the
    /// first byte after the branch instruction.
    /// Unlike zero, negative, and even carry, overflow is modified by very few instructions. It is
    /// most often used with the BIT instruction, particularly for polling hardware registers. It is
    /// also sometimes used for signed overflow with ADC and SBC. The standard 6502 chip allows an
    /// external device to set overflow usign a pin, enabling software to poll for that event, but
    /// this is not present on the NES' 2A03.
    pub fn bvs(&mut self, operand: Operand, _: &mut CpuBus) -> u8 {
        self.branch_if(contains(self.status, OVERFLOW), operand)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::instructions::test_utils::{assert_branch_not_taken, assert_branch_taken};

    #[test]
    fn bcc_does_not_branch_when_carry_is_set() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, CARRY, true);
        assert_branch_not_taken(Cpu::bcc, &mut cpu);
    }

    #[test]
    fn bcc_branches_when_carry_is_clear_same_page() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, CARRY, false);
        assert_branch_taken(Cpu::bcc, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn bcc_branch_taken_adds_two_cycles_when_page_crossed() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, CARRY, false);
        assert_branch_taken(Cpu::bcc, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn bcs_does_not_branch_when_carry_clear() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, CARRY, false);
        assert_branch_not_taken(Cpu::bcs, &mut cpu);
    }

    #[test]
    fn bcs_branches_when_carry_set_same_page() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, CARRY, true);
        assert_branch_taken(Cpu::bcs, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn bcs_branches_across_page_boundary() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, CARRY, true);
        assert_branch_taken(Cpu::bcs, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn beq_does_not_branch_when_zero_clear() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, ZERO, false);
        assert_branch_not_taken(Cpu::beq, &mut cpu);
    }

    #[test]
    fn beq_branches_when_zero_set_same_page() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, ZERO, true);
        assert_branch_taken(Cpu::beq, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn beq_branches_across_page_boundary() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, ZERO, true);
        assert_branch_taken(Cpu::beq, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn bne_branches_when_zero_clear() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, ZERO, false);
        assert_branch_taken(Cpu::bne, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn bne_does_not_branch_when_zero_is_set() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, ZERO, true);
        assert_branch_not_taken(Cpu::bne, &mut cpu);
    }

    #[test]
    fn bne_branches_across_page_boundary() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, ZERO, false);
        assert_branch_taken(Cpu::bne, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn bpl_branches_when_negative_clear() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, NEGATIVE, false);
        assert_branch_taken(Cpu::bpl, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn bpl_does_not_branch_when_negative_is_set() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, NEGATIVE, true);
        assert_branch_not_taken(Cpu::bpl, &mut cpu);
    }

    #[test]
    fn bpl_branches_across_page_boundary() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, NEGATIVE, false);
        assert_branch_taken(Cpu::bpl, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn bmi_does_not_branch_when_negative_clear() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, NEGATIVE, false);
        assert_branch_not_taken(Cpu::bmi, &mut cpu);
    }

    #[test]
    fn bmi_branches_when_negative_set_same_page() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, NEGATIVE, true);
        assert_branch_taken(Cpu::bmi, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn bmi_branches_across_page_boundary() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, NEGATIVE, true);
        assert_branch_taken(Cpu::bmi, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn bvc_branches_when_overflow_clear() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, OVERFLOW, false);
        assert_branch_taken(Cpu::bvc, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn bvc_does_not_branch_when_overflow_is_set() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, OVERFLOW, true);
        assert_branch_not_taken(Cpu::bvc, &mut cpu);
    }

    #[test]
    fn bvc_branches_across_page_boundary() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, OVERFLOW, false);
        assert_branch_taken(Cpu::bvc, &mut cpu, 0x1105, 2);
    }

    #[test]
    fn bvs_branches_when_overflow_is_set() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        set(&mut cpu.status, OVERFLOW, true);
        assert_branch_taken(Cpu::bvs, &mut cpu, 0x1010, 1);
    }

    #[test]
    fn bvs_does_not_branch_when_overflow_is_clear() {
        let mut cpu = Cpu::new();
        set(&mut cpu.status, OVERFLOW, false);
        assert_branch_not_taken(Cpu::bvs, &mut cpu);
    }

    #[test]
    fn bvs_branches_across_page_boundary() {
        let mut cpu = Cpu::new();
        cpu.pc = 0x10F0;
        set(&mut cpu.status, OVERFLOW, true);
        assert_branch_taken(Cpu::bvs, &mut cpu, 0x1105, 2);
    }
}
