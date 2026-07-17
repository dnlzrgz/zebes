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
pub fn disassemble(bus: &CpuBus, address: u16) -> DecodedInstruction {
    let opcode = bus.peek(address);
    let info = opcode_table()[opcode as usize];

    let mut next = address.wrapping_add(1);
    let mut bytes = vec![opcode];

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

            format!("${:02X}", addr)
        }
        AddressingMode::ZeroPageX => {
            let addr = bus.peek(next);

            bytes.push(addr);
            next = next.wrapping_add(1);

            format!("${:02X},X", addr)
        }
        AddressingMode::ZeroPageY => {
            let addr = bus.peek(next);

            bytes.push(addr);
            next = next.wrapping_add(1);

            format!("${:02X},Y", addr)
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
            format!("${:04X}", addr)
        }
        AddressingMode::AbsoluteX => {
            let lo = bus.peek(next);
            let hi = bus.peek(next.wrapping_add(1));

            bytes.push(lo);
            bytes.push(hi);

            next = next.wrapping_add(2);

            let addr = u16::from_le_bytes([lo, hi]);
            format!("${:04X},X", addr)
        }
        AddressingMode::AbsoluteY => {
            let lo = bus.peek(next);
            let hi = bus.peek(next.wrapping_add(1));

            bytes.push(lo);
            bytes.push(hi);

            next = next.wrapping_add(2);

            let addr = u16::from_le_bytes([lo, hi]);
            format!("${:04X},Y", addr)
        }
        AddressingMode::Indirect => {
            let lo = bus.peek(next);
            let hi = bus.peek(next.wrapping_add(1));

            bytes.push(lo);
            bytes.push(hi);

            next = next.wrapping_add(2);

            let addr = u16::from_le_bytes([lo, hi]);
            format!("(${:04X})", addr)
        }
        AddressingMode::IndirectX => {
            let zp = bus.peek(next);

            bytes.push(zp);
            next = next.wrapping_add(1);

            format!("(${:02X},X)", zp)
        }
        AddressingMode::IndirectY => {
            let zp = bus.peek(next);

            bytes.push(zp);
            next = next.wrapping_add(1);

            format!("(${:02X}),Y", zp)
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
