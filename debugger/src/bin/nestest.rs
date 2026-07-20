use zebes_core::{cartridge::Cartridge, cpu::Cpu, cpu_bus::CpuBus};
use zebes_debugger::tracer::trace;

fn strip_ppu_column(line: &str) -> String {
    let ppu_start = match line.find("PPU:") {
        Some(idx) => idx,
        None => return line.to_string(),
    };
    let cyc_start = match line[ppu_start..].find("CYC:") {
        Some(idx) => ppu_start + idx,
        None => return line[..ppu_start].trim_end().to_string(),
    };

    format!("{}{}", &line[..ppu_start], &line[cyc_start..])
}

fn main() {
    let rom = std::fs::read("tests/roms/nestest.nes").unwrap();
    let golden_log = std::fs::read_to_string("tests/roms/nestest.log").unwrap();
    let mut golden_lines = golden_log.lines();

    let cartridge = match Cartridge::try_from_ines(&rom) {
        Ok(cartridge) => cartridge,
        Err(err) => {
            eprintln!("Failed to load cartridge: {err}");
            return;
        }
    };

    let mut bus = CpuBus::with_cartridge(cartridge);
    let mut cpu = Cpu::new();

    cpu.reset(&bus);
    cpu.set_pc(0xC000);

    loop {
        if cpu.cycles() == 0 {
            let mine = trace(&cpu, &bus);
            match golden_lines.next() {
                Some(expected) => {
                    let expected = strip_ppu_column(expected);
                    if expected != mine {
                        eprintln!("MISMATCH:\n{expected}\n{mine}");
                        break;
                    }
                }
                None => break,
            }
        }

        cpu.clock(&mut bus);
    }
}
