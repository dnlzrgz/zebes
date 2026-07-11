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

        assert_eq!(
            bus.peek(0x01FD),
            (expected_pushed_addr >> 8) as u8,
            "high byte"
        );
        assert_eq!(
            bus.peek(0x01FC),
            (expected_pushed_addr & 0x00FF) as u8,
            "low byte"
        );
        assert_eq!(cpu.sp, 0xFB, "sp must decrement by 2 (two pushes)");

        assert_eq!(cpu.pc, 0x8000, "pc must jump to the subroutine address");
        assert_eq!(extra_cycles, 0);
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
        assert_eq!(
            bus.peek(0x01FD),
            (expected_return_addr >> 8) as u8,
            "high byte of return address"
        );
        assert_eq!(
            bus.peek(0x01FC),
            (expected_return_addr & 0x00FF) as u8,
            "low byte of return address"
        );
        assert_eq!(cpu.sp, 0xFA, "sp must decrement by exactly 3");

        // The pushed status byte must have BREAK and UNUSED forced high while preserving flags
        // already set.
        let pushed_status = bus.peek(0x01FB);
        let expected_status = to_pushed_byte(status_before);
        assert_eq!(pushed_status, expected_status);

        // BREAK is not a "real"" flag. self.status should not have BREAK set after brk() returns.
        assert!(
            !contains(cpu.status, BREAK),
            "BREAK must never be set on the live status"
        );

        assert!(
            contains(cpu.status, INTERRUPT_DISABLE),
            "BRK must set INTERRUPT_DISABLE going forward"
        );

        // Execution must have jumped to the IRQ/BRK vector.
        assert_eq!(cpu.pc, 0x8000);

        assert_eq!(extra_cycles, 0);
    }
}
