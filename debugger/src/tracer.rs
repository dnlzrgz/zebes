use zebes_core::{
    cpu::{Cpu, addressing::AddressingMode, instructions::Instruction, opcodes::opcode_table},
    cpu_bus::CpuBus,
};

use crate::disassembler::disassemble;

pub fn trace(cpu: &Cpu, bus: &CpuBus) -> String {
    let pc = cpu.pc();
    let decoded = disassemble(bus, pc);
    let info = opcode_table()[decoded.bytes[0] as usize];

    let operand = resolve_operand(&decoded, info, cpu.x(), cpu.y(), bus);
    let bytes_str = decoded
        .bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ");

    let asm = if operand.is_empty() {
        decoded.mnemonic.clone()
    } else {
        format!("{} {}", decoded.mnemonic, operand)
    };

    format!(
        "{:04X}  {:<8}  {:<32}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
        pc,
        bytes_str,
        asm,
        cpu.a(),
        cpu.x(),
        cpu.y(),
        cpu.status(),
        cpu.sp(),
        cpu.total_cycles()
    )
}

fn resolve_operand(
    decoded: &super::disassembler::DecodedInstruction,
    info: zebes_core::cpu::opcodes::Opcode,
    x: u8,
    y: u8,
    bus: &CpuBus,
) -> String {
    let operand_bytes = &decoded.bytes[1..];

    match info.mode {
        AddressingMode::Implied
        | AddressingMode::Accumulator
        | AddressingMode::Immediate
        | AddressingMode::Relative => decoded.operand.clone(),

        AddressingMode::ZeroPage => {
            let val = bus.peek(operand_bytes[0] as u16);
            format!("{} = {val:02X}", decoded.operand)
        }
        AddressingMode::ZeroPageX => {
            let eff = operand_bytes[0].wrapping_add(x);
            let val = bus.peek(eff as u16);
            format!("{} @ {eff:02X} = {val:02X}", decoded.operand)
        }
        AddressingMode::ZeroPageY => {
            let eff = operand_bytes[0].wrapping_add(y);
            let val = bus.peek(eff as u16);
            format!("{} @ {eff:02X} = {val:02X}", decoded.operand)
        }
        AddressingMode::Absolute => {
            if matches!(info.instruction, Instruction::JMP | Instruction::JSR) {
                decoded.operand.clone()
            } else {
                let addr = u16::from_le_bytes([operand_bytes[0], operand_bytes[1]]);
                let val = bus.peek(addr);
                format!("{} = {val:02X}", decoded.operand)
            }
        }
        AddressingMode::AbsoluteX => {
            let base = u16::from_le_bytes([operand_bytes[0], operand_bytes[1]]);
            let eff = base.wrapping_add(x as u16);
            let val = bus.peek(eff);
            format!("{} @ {eff:04X} = {val:02X}", decoded.operand)
        }
        AddressingMode::AbsoluteY => {
            let base = u16::from_le_bytes([operand_bytes[0], operand_bytes[1]]);
            let eff = base.wrapping_add(y as u16);
            let val = bus.peek(eff);
            format!("{} @ {eff:04X} = {val:02X}", decoded.operand)
        }
        AddressingMode::Indirect => {
            let pointer = u16::from_le_bytes([operand_bytes[0], operand_bytes[1]]);
            let lo = bus.peek(pointer);
            let hi_addr = if pointer & 0x00FF == 0x00FF {
                pointer & 0xFF00
            } else {
                pointer.wrapping_add(1)
            };
            let hi = bus.peek(hi_addr);
            let target = u16::from_le_bytes([lo, hi]);
            format!("{} = {target:04X}", decoded.operand)
        }
        AddressingMode::IndirectX => {
            let ptr = operand_bytes[0].wrapping_add(x);
            let lo = bus.peek(ptr as u16);
            let hi = bus.peek(ptr.wrapping_add(1) as u16);
            let addr = u16::from_le_bytes([lo, hi]);
            let val = bus.peek(addr);
            format!("{} @ {ptr:02X} = {addr:04X} = {val:02X}", decoded.operand)
        }
        AddressingMode::IndirectY => {
            let zp = operand_bytes[0];
            let lo = bus.peek(zp as u16);
            let hi = bus.peek(zp.wrapping_add(1) as u16);
            let base = u16::from_le_bytes([lo, hi]);
            let eff = base.wrapping_add(y as u16);
            let val = bus.peek(eff);
            format!("(${zp:02X}),Y = {base:04X} @ {eff:04X} = {val:02X}")
        }
    }
}
