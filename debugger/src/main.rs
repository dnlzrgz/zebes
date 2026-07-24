use ratatui::{Terminal, backend::CrosstermBackend};
use zebes_core::nes::Nes;

fn main() -> std::io::Result<()> {
    let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    crossterm::terminal::enable_raw_mode()?;

    let mut nes = Nes::new();
    nes.load(&[0; 0x800])
        .unwrap_or_else(|err| panic!("Failed to load cartridge: {err}"));
    nes.reset();
    nes.cpu_mut().set_pc(0xC000);

    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
