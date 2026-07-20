use ratatui::{Terminal, backend::CrosstermBackend};
use zebes_core::{cpu::Cpu, cpu_bus::CpuBus};

struct App {
    cpu: Cpu,
    bus: CpuBus,
}

impl App {
    fn step(&mut self) {
        self.cpu.clock(&mut self.bus);
        while self.cpu.cycles() > 0 {
            self.cpu.clock(&mut self.bus);
        }
    }
}

fn main() -> std::io::Result<()> {
    let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    crossterm::terminal::enable_raw_mode()?;

    let app = App {
        cpu: Cpu::new(),
        bus: CpuBus::new(),
    };

    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
