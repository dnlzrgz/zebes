use zebes_core::{cpu::Cpu, cpu_bus::CpuBus};

use crate::disassembler::disassemble;

pub fn trace(cpu: &Cpu, bus: &CpuBus) -> String {
    let pc = cpu.pc();
    let decoded = disassemble(bus, pc, cpu.x(), cpu.y());

    let bytes_str = decoded
        .bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ");

    let asm = if decoded.operand.is_empty() {
        decoded.mnemonic.clone()
    } else {
        format!("{} {}", decoded.mnemonic, decoded.operand)
    };

    format!(
        "{:04X}  {:<8}  {:<32}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:>3},{:>3} CYC:{}",
        pc,
        bytes_str,
        asm,
        cpu.a(),
        cpu.x(),
        cpu.y(),
        cpu.status(),
        cpu.sp(),
        bus.ppu.scanline(),
        bus.ppu.cycle(),
        cpu.total_cycles()
    )
}
