use zebes_core::nes::Nes;
use zebes_debugger::tracer::trace;

#[test]
fn test_against_nestest() {
    let rom = std::fs::read("tests/roms/nestest.nes").expect("Missing nestest.nes file");
    let golden_log =
        std::fs::read_to_string("tests/roms/nestest.log").expect("Missing nestest.log file");
    let mut golden_lines = golden_log.lines();

    let mut nes = Nes::new();
    nes.load(&rom)
        .unwrap_or_else(|err| panic!("Failed to load cartridge: {err}"));
    nes.reset();
    nes.cpu_mut().set_pc(0xC000);

    let mut line_num = 0;
    loop {
        if nes.cpu().cycles() == 0 {
            let mine = trace(nes.cpu(), nes.bus());
            match golden_lines.next() {
                Some(expected) if expected.contains('*') => break, // TODO: unofficial opcodes
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

        nes.clock();
    }
}
