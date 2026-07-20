use zebes_core::cpu::instructions::Instruction;
use zebes_core::cpu::{addressing::AddressingMode, opcodes::opcode_table};
use zebes_core::cpu_bus::CpuBus;

pub struct DecodedInstruction {
    pub address: u16,
    pub bytes: Vec<u8>,
    pub mnemonic: String,
    pub operand: String,
    pub next: u16,
}

/// Decodes the instruction at `address`.
pub fn disassemble(bus: &CpuBus, address: u16, x: u8, y: u8) -> DecodedInstruction {
    let opcode = bus.peek(address);
    let info = opcode_table()[opcode as usize];

    let mut next = address.wrapping_add(1);
    let mut bytes = vec![opcode];

    let is_jump = matches!(info.instruction, Instruction::JMP | Instruction::JSR);

    let operand = match info.mode {
        AddressingMode::Implied => String::new(),
        AddressingMode::Accumulator => "A".to_string(),
        AddressingMode::Immediate => {
            let value = bus.peek(next);
            bytes.push(value);
            next = next.wrapping_add(1);
            format!("#${:02X}", value)
        }
        AddressingMode::ZeroPage => {
            let addr = bus.peek(next);
            bytes.push(addr);
            next = next.wrapping_add(1);

            let value = bus.peek(addr as u16);
            format!("${:02X} = {:02X}", addr, value)
        }
        AddressingMode::ZeroPageX => {
            let base = bus.peek(next);
            bytes.push(base);
            next = next.wrapping_add(1);

            let eff = base.wrapping_add(x);
            let value = bus.peek(eff as u16);
            format!("${:02X},X @ {:02X} = {:02X}", base, eff, value)
        }
        AddressingMode::ZeroPageY => {
            let base = bus.peek(next);
            bytes.push(base);
            next = next.wrapping_add(1);

            let eff = base.wrapping_add(y);
            let value = bus.peek(eff as u16);
            format!("${:02X},Y @ {:02X} = {:02X}", base, eff, value)
        }
        AddressingMode::Relative => {
            let offset = bus.peek(next) as i8;
            bytes.push(offset as u8);
            next = next.wrapping_add(1);

            let target = ((next as i32) + (offset as i32)) as u16;
            format!("${:04X}", target)
        }
        AddressingMode::Absolute => {
            let lo = bus.peek(next);
            let hi = bus.peek(next.wrapping_add(1));
            bytes.push(lo);
            bytes.push(hi);
            next = next.wrapping_add(2);

            let addr = u16::from_le_bytes([lo, hi]);
            if is_jump {
                format!("${:04X}", addr)
            } else {
                let value = bus.peek(address);
                format!("${:04X} = {:02X}", addr, value)
            }
        }
        AddressingMode::AbsoluteX => {
            let lo = bus.peek(next);
            let hi = bus.peek(next.wrapping_add(1));
            bytes.push(lo);
            bytes.push(hi);
            next = next.wrapping_add(2);

            let base = u16::from_le_bytes([lo, hi]);
            let eff = base.wrapping_add(x as u16);
            let value = bus.peek(eff);
            format!("${:04X},X @ {:04X} = {:02X}", base, eff, value)
        }
        AddressingMode::AbsoluteY => {
            let lo = bus.peek(next);
            let hi = bus.peek(next.wrapping_add(1));
            bytes.push(lo);
            bytes.push(hi);
            next = next.wrapping_add(2);

            let base = u16::from_le_bytes([lo, hi]);
            let eff = base.wrapping_add(y as u16);
            let value = bus.peek(eff);
            format!("${:04X},Y @ {:04X} = {:02X}", base, eff, value)
        }
        AddressingMode::Indirect => {
            let lo = bus.peek(next);
            let hi = bus.peek(next.wrapping_add(1));
            bytes.push(lo);
            bytes.push(hi);
            next = next.wrapping_add(2);

            let ptr = u16::from_le_bytes([lo, hi]);
            let lo_target = bus.peek(ptr);
            let hi_target = bus.peek((ptr & 0xFF00) | (ptr.wrapping_add(1) & 0x00FF));
            let target = u16::from_le_bytes([lo_target, hi_target]);

            format!("(${:04X}) = {:04X}", ptr, target)
        }
        AddressingMode::IndirectX => {
            let zp = bus.peek(next);
            bytes.push(zp);
            next = next.wrapping_add(1);

            let ptr = zp.wrapping_add(x);
            let lo = bus.peek(ptr as u16);
            let hi = bus.peek(ptr.wrapping_add(1) as u16);
            let eff = u16::from_le_bytes([lo, hi]);
            let value = bus.peek(eff);

            format!(
                "(${:02X},X) @ {:02X} = {:04X} = {:02X}",
                zp, ptr, eff, value
            )
        }
        AddressingMode::IndirectY => {
            let zp = bus.peek(next);
            bytes.push(zp);
            next = next.wrapping_add(1);

            let lo = bus.peek(zp as u16);
            let hi = bus.peek(zp.wrapping_add(1) as u16);
            let base = u16::from_le_bytes([lo, hi]);
            let eff = base.wrapping_add(y as u16);
            let value = bus.peek(eff);

            format!(
                "(${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                zp, base, eff, value
            )
        }
    };

    DecodedInstruction {
        address,
        bytes,
        mnemonic: format!("{:?}", info.instruction),
        operand,
        next,
    }
}
