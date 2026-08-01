use crossterm::event::KeyCode;
use zebes_core::nes::Nes;

pub enum ViewMode {
    Cpu,
    Ppu,
}

pub struct App {
    pub nes: Nes,
    pub mem_base: u16,
    pub running: bool,
    pub view_mode: ViewMode,
}

impl App {
    pub fn new(nes: Nes) -> Self {
        Self {
            nes,
            mem_base: 0,
            running: false,
            view_mode: ViewMode::Cpu,
        }
    }

    /// Handle key press.
    pub fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char(' ') => {
                self.running = false;
                self.nes.clock();
            }
            KeyCode::Char('n') => {
                self.running = false;
                let start_pc = self.nes.cpu().pc();
                while self.nes.cpu().pc() == start_pc {
                    self.nes.clock();
                }
            }
            KeyCode::Enter => self.running = !self.running,
            KeyCode::Char('r') => self.nes.reset(),
            KeyCode::Char('v') => {
                self.view_mode = match self.view_mode {
                    ViewMode::Cpu => ViewMode::Ppu,
                    ViewMode::Ppu => ViewMode::Cpu,
                }
            }
            KeyCode::Up => self.mem_base = self.mem_base.saturating_sub(16),
            KeyCode::Down => self.mem_base = self.mem_base.saturating_add(16),
            _ => {}
        }
        false
    }
}
