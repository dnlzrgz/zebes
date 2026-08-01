use std::{
    env, fs, io, process,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

use zebes_core::nes::Nes;
use zebes_debugger::app::App;
use zebes_debugger::views::render;

// Real NES/PPU frame rate (NTSC).
const TARGET_FRAME_TIME: Duration = Duration::from_micros(16_639); // ~60 Hz

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: zebes-debugger <path-to-rom.nes>");
        process::exit(1);
    });
    let rom = fs::read(&path)?;

    let mut nes = Nes::new();
    nes.load(&rom)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    nes.reset();

    let terminal = ratatui::init();
    let result = run(terminal, App::new(nes));
    ratatui::restore();
    Ok(result?)
}

fn run(mut terminal: DefaultTerminal, mut app: App) -> io::Result<()> {
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| render(frame, &app))?;

        let poll_timeout = if app.running {
            Duration::from_millis(0)
        } else {
            Duration::from_millis(50)
        };

        if event::poll(poll_timeout)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && app.handle_key(key.code)
        {
            return Ok(());
        }

        if app.running {
            let now = Instant::now();
            if now.duration_since(last_tick) >= TARGET_FRAME_TIME {
                last_tick += TARGET_FRAME_TIME;
                let start_frame = app.nes.bus().ppu.frame();
                while app.nes.bus().ppu.frame() == start_frame {
                    app.nes.clock();
                }
            } else {
                std::thread::sleep(Duration::from_millis(1));
            }
        } else {
            last_tick = Instant::now();
        }
    }
}
