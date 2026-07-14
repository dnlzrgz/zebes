use crate::{
    bus::Bus,
    cpu::{Cpu, addressing::Operand, flags::*},
};

impl Cpu {
    /// Jump
    /// PC = memory
    ///
    /// JMP sets the program counter to a new value, allowing code to execute from a new location.
    /// If we wish to be able to return from that location, JSR should normally be used instead.
    /// The indirect addressing modes uses the operand as a poointer, getting the new 2-byte program
    /// counter value from the specified address. Unfortunately, because of a CPU bug, if this
    /// 2-byte variable has an address ending in $FF and thus crossed a page, then the CPU fails to
    /// increment the page when reading the second byte and thus reads the wrong address. For
    /// example, JMP ($03FF) read $03FF and $0300 instead of $0400. Care should be taken to ensure
    /// this variable does not cross a page.
    pub fn jmp(&mut self, operand: Operand, _: &mut Bus) -> u8 {
        let (address, _) = operand.expect_address();
        self.pc = address;
        0
    }

    /// Jump to Subroutine
    /// `push` PC + 2 high byte to stack
    /// `push` PC + 2 low byte to stack
    /// PC = memory
    ///
    /// JSR pushes the current program counter to the stack and then sets the program counter to a
    /// new value. This allows code to call a function and return with RTS back to the instruction
    /// after the JSR.
    /// Notably, the return address on the stack point 1 byte before the start of the next
    /// instruction, rather than directly at the instruction. This is because RTS increments the
    /// program counter before the next instruction is fetched. This differs from the return address
    /// pushed by interrupts and used by RTI, which points directly to the next instruction.
    pub fn jsr(&mut self, operand: Operand, bus: &mut Bus) -> u8 {
        self.pc = self.pc.wrapping_sub(1);

        self.push_byte(bus, (self.pc >> 8) as u8); // high byte
        self.push_byte(bus, (self.pc & 0x00FF) as u8); // low byte

        let (address, _) = operand.expect_address();
        self.pc = address;

        0
    }

    /// Return from Subroutine
    /// `pull` PC low byte from stack
    /// `pull` PC high byte from stack
    /// PC = PC + 1
    ///
    /// RTS pulls an address from the stack into the program counter and then increments it. It is
    /// normally used at the end of a function to return to the instruction after the JSR that
    /// called the function. However, RTS is also sometimes used to implement jump tables.
    pub fn rts(&mut self, _: Operand, bus: &mut Bus) -> u8 {
        let lo = self.pull_byte(bus);
        let hi = self.pull_byte(bus);
        self.pc = u16::from_le_bytes([lo, hi]);
        self.pc = self.pc.wrapping_add(1);

        0
    }

    /// Break (Software IRQ)
    /// `push` PC + 2 high byte to stack
    /// `push` PC + 2 low byte to stack
    /// `push` NV11DIZC flags to stack
    /// PC = ($FFFE)
    ///
    /// BRK triggers an interrupt request (IRQ). IRQs are normally triggered by external hardware,
    /// and BRK is the only way to do it in software. Like a typical IRQ, it pushes the current
    /// program counter and processor flags to the stack, sets the interrupt disable flag, and jumps
    /// to the IRQ handler. Unlike a typical IRQ, it sets the break flag in the flags byte that is
    /// pushed to the stack (like PHP) and it triggers an interrupt even if the interrupt disable
    /// flag is set. Notably, the return address that is pushed to the stack skips the byte after the
    /// BRK opcode. For this reason, BRK is often considered a 2-byte instruction with an unused
    /// immediate.
    /// Unfortunately, a 6502 bug allows the BRK IRQ to be overridden by an NMI occurring at the same
    /// time. In this case, only the NMI handler is called; the IRQ handler is skipped. However, the
    /// break flag is still set in the flags byte pushed to the stack, so the NMI hanlder can detect
    /// that this occurred (albeit slowly) by checking this flag.
    /// Because BRK uses the value $000, any byte in a programmable ROM can be overwritten with a
    /// BRK instruction to send execution to an IRQ handler. This is useful for patching one-time
    /// programmable ROMs. BRK can also be used as a system call mechanism, and the unused byte can
    /// be used by software as an argument (although it is inconvenient to access). In the context
    /// of NES games, BRK is often most useful as a crash handler, where the unused program space is
    /// filled with $00 and the IRQ handler displays debugging information or otherwise handles the
    /// crash in a clean way.
    pub fn brk(&mut self, _: Operand, bus: &mut Bus) -> u8 {
        self.pc = self.pc.wrapping_add(1);

        self.push_byte(bus, (self.pc >> 8) as u8); // high byte
        self.push_byte(bus, (self.pc & 0x00FF) as u8); // low byte

        let pushed_status = to_pushed_byte(self.status);
        self.push_byte(bus, pushed_status);

        set(&mut self.status, INTERRUPT_DISABLE, true);

        self.pc = u16::from_le_bytes([bus.read(0xFFFE), bus.read(0xFFFF)]);

        0
    }

    /// Return from Interrupt
    /// `pull` NVxxDIZC flags from stack
    /// `pull` PC low byte from stack
    /// `pull` PC high byte from stack
    ///
    /// RTI returns from an interrupt handler, first pulling the 6 status flags from the stack and
    /// then pulling the new program counter. The flag pulling behaves like PLP except that changes
    /// to the interrupt disable flag apply immediately instead of being delayed 1 instruction. This
    /// is because the flags change before IRQs are polled for the instruction, not after. The PC
    /// pulling behaves like RTS except that the return address is the exact same address of the
    /// next instruction instead of 1 byte before it.
    pub fn rti(&mut self, _: Operand, bus: &mut Bus) -> u8 {
        let pulled = self.pull_byte(bus);
        self.status = (pulled & !BREAK) | UNUSED;

        let lo = self.pull_byte(bus);
        let hi = self.pull_byte(bus);
        self.pc = u16::from_le_bytes([lo, hi]);

        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::instructions::test_utils::operand_at;

    #[test]
    fn jmp_sets_pc_to_operand_address() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;

        let extra_cycles = cpu.jmp(operand_at(0x8000), &mut bus);

        assert_eq!(cpu.pc, 0x8000);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn jsr_pushes_return_address_minus_one_and_jumps() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1234;
        cpu.sp = 0xFD;

        let extra_cycles = cpu.jsr(operand_at(0x8000), &mut bus);

        // the pushed address should be pc - 1.
        let expected_pushed_addr: u16 = 0x1233;

        assert_eq!(bus.peek(0x01FD), (expected_pushed_addr >> 8) as u8);
        assert_eq!(bus.peek(0x01FC), (expected_pushed_addr & 0x00FF) as u8);
        assert_eq!(cpu.sp, 0xFB);

        assert_eq!(cpu.pc, 0x8000);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn rts_restores_pc_from_low_then_high_byte_plus_one() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0xFB;
        bus.write(0x01FC, 0x33); // pc low byte
        bus.write(0x01FD, 0x12); // pc high byte

        let extra_cycles = cpu.rts(Operand::Accumulator, &mut bus);

        // Pulled address is 0x1233, then +1 gives 0x1234.
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn rts_advances_sp_by_exactly_two() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0xFB;
        bus.write(0x01FC, 0x00);
        bus.write(0x01FD, 0x00);

        cpu.rts(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.sp, 0xFD);
    }

    #[test]
    fn rts_mirrors_a_previous_jsr() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.pc = 0x1234; // already advanced past JSR's operand bytes
        cpu.sp = 0xFD;

        cpu.jsr(operand_at(0x8000), &mut bus);
        assert_eq!(cpu.pc, 0x8000);

        cpu.rts(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0xFD);
    }

    #[test]
    fn brk_pushes_return_address_and_status_then_jumps_to_irq_vector() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();

        cpu.pc = 0x1234;
        cpu.sp = 0xFD;

        // Some arbitrary flags so we can confirm if they have been preserved.
        set(&mut cpu.status, CARRY, true);
        set(&mut cpu.status, NEGATIVE, true);
        set(&mut cpu.status, INTERRUPT_DISABLE, false);

        // Saved status before BRK modifies it.
        let status_before = cpu.status;

        // IRQ/BRK vector: where execution should end up.
        bus.write(0xFFFE, 0x00);
        bus.write(0xFFFF, 0x80); // vector points at 0x8000

        let extra_cycles = cpu.brk(Operand::Accumulator, &mut bus); // operand is unused

        // pc advanced by 1 before pushing, so the return address on the stack should be 0x1235, not 0x1234.
        let expected_return_addr: u16 = 0x1235;

        // Stack pushes happen high byte first, then low byte, then
        // status and sp decrements by 1 after each push. Starting sp
        // was 0xFD, so:
        //   push high byte at 0x0100 + 0xFD = 0x01FD, sp -> 0xFC
        //   push low byte  at 0x0100 + 0xFC = 0x01FC, sp -> 0xFB
        //   push status    at 0x0100 + 0xFB = 0x01FB, sp -> 0xFA
        assert_eq!(bus.peek(0x01FD), (expected_return_addr >> 8) as u8);
        assert_eq!(bus.peek(0x01FC), (expected_return_addr & 0x00FF) as u8);
        assert_eq!(cpu.sp, 0xFA);

        // The pushed status byte must have BREAK and UNUSED forced high while preserving flags
        // already set.
        let pushed_status = bus.peek(0x01FB);
        let expected_status = to_pushed_byte(status_before);
        assert_eq!(pushed_status, expected_status);

        // self.status should not have BREAK set after brk() returns.
        assert!(!contains(cpu.status, BREAK));

        assert!(contains(cpu.status, INTERRUPT_DISABLE));

        // Execution must have jumped to the IRQ/BRK vector.
        assert_eq!(cpu.pc, 0x8000);

        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn rti_restores_status_with_break_forced_low() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0xFA;
        // Stack layout (bottom to top as pulled): status, pc-low, pc-high
        bus.write(0x01FB, CARRY | BREAK); // status, as if pushed with BREAK set
        bus.write(0x01FC, 0x00); // pc low byte
        bus.write(0x01FD, 0x80); // pc high byte

        let extra_cycles = cpu.rti(Operand::Accumulator, &mut bus);

        assert!(!contains(cpu.status, BREAK));
        assert!(contains(cpu.status, CARRY));
        assert!(contains(cpu.status, UNUSED));
        assert_eq!(extra_cycles, 0);
    }

    #[test]
    fn rti_restores_pc_from_low_then_high_byte() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0xFA;
        bus.write(0x01FB, 0x00); // status
        bus.write(0x01FC, 0x34); // pc low byte
        bus.write(0x01FD, 0x12); // pc high byte

        cpu.rti(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.pc, 0x1234);
    }

    #[test]
    fn rti_advances_sp_by_exactly_three() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();
        cpu.sp = 0xFA;
        bus.write(0x01FB, 0x00);
        bus.write(0x01FC, 0x00);
        bus.write(0x01FD, 0x00);

        cpu.rti(Operand::Accumulator, &mut bus);

        assert_eq!(cpu.sp, 0xFD, "sp must advance by 3 (one pull per byte)");
    }

    #[test]
    fn rti_mirrors_a_previous_brk() {
        let mut bus = Bus::new();
        let mut cpu = Cpu::new();

        cpu.pc = 0x1234;
        cpu.sp = 0xFD;
        set(&mut cpu.status, CARRY, true);
        set(&mut cpu.status, NEGATIVE, true);
        set(&mut cpu.status, INTERRUPT_DISABLE, false);
        let status_before_brk = cpu.status;
        let pc_after_brk_advance = cpu.pc.wrapping_add(1); // BRK's own pc+1 before pushing

        // IRQ/BRK vector — brk() will jump here.
        bus.write(0xFFFE, 0x00);
        bus.write(0xFFFF, 0x90);

        cpu.brk(Operand::Accumulator, &mut bus);
        cpu.status = 0x00;
        cpu.pc = 0x0000;

        cpu.rti(Operand::Accumulator, &mut bus);

        assert_eq!(
            cpu.pc, pc_after_brk_advance,
            "RTI must return to BRK's exact return address"
        );
        assert_eq!(
            cpu.status, status_before_brk,
            "RTI must restore the pre-BRK flags exactly"
        );
        assert_eq!(
            cpu.sp, 0xFD,
            "sp should return to its original value after a full brk+rti round trip"
        );
    }
}
