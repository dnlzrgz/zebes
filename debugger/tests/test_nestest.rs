use zebes_core::{cartridge::Cartridge, cpu::Cpu, cpu_bus::CpuBus};
use zebes_debugger::tracer::trace;

#[test]
fn test_against_nestest() {
    let rom = std::fs::read("tests/roms/nestest.nes").expect("Missing nestest.nes file");
    let golden_log =
        std::fs::read_to_string("tests/roms/nestest.log").expect("Missing nestest.log file");
    let mut golden_lines = golden_log.lines();

    let cartridge = Cartridge::try_from_ines(&rom)
        .unwrap_or_else(|err| panic!("Failed to load cartridge: {err}"));

    let mut bus = CpuBus::with_cartridge(cartridge);
    let mut cpu = Cpu::new();

    cpu.reset(&bus);
    cpu.set_pc(0xC000);

    let mut line_num = 0;
    loop {
        if cpu.cycles() == 0 {
            let mine = trace(&cpu, &bus);
            match golden_lines.next() {
                Some(expected) => {
                    assert_eq!(
                        expected, mine,
                        "\n{line_num}:\nexp: {expected}\ngot: {mine}\n",
                    );
                    line_num += 1;
                }
                None => break,
            }
        }

        cpu.clock(&mut bus);
    }
}
