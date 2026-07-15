mod disassembler;

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use zebes_core::{bus::Bus, cpu::Cpu};

struct App {
    cpu: Cpu,
    bus: Bus,

    /// Current 256-byte memory page being displayed.
    memory_page: u16,
}

impl App {
    fn step(&mut self) {
        self.cpu.clock(&mut self.bus);
        while self.cpu.cycles() > 0 {
            self.cpu.clock(&mut self.bus);
        }
    }
}

// Load Program (assembled at https://www.masswerk.at/6502/assembler.html)
// *=$8000
// LDX #10
// STX $0000
// LDX #3
// STX $0001
// LDY $0000
// LDA #0
// CLC
// loop
// ADC $0001
// DEY
// BNE loop
// STA $0002
// NOP
// NOP
// NOP
pub const DEMO_PROGRAM: [u8; 32] = [
    0xA2, 0x0A, 0x8E, 0x00, 0x00, 0xA2, 0x03, 0x8E, 0x01, 0x00, 0xAC, 0x00, 0x00, 0xA9, 0x00, 0x18,
    0x6D, 0x01, 0x00, 0x88, 0xD0, 0xFA, 0x8D, 0x02, 0x00, 0xEA, 0xEA, 0xEA, 0x00, 0x00, 0x00, 0x00,
];

fn main() -> std::io::Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    crossterm::terminal::enable_raw_mode()?;

    let mut app = App {
        cpu: Cpu::new(),
        bus: Bus::new(),
        memory_page: 0,
    };

    app.bus.load_bytes(&DEMO_PROGRAM, 0x8000);
    app.bus.write(0xFFFC, 0x00);
    app.bus.write(0xFFFD, 0x80);
    app.cpu.reset(&app.bus);

    loop {
        terminal.draw(|frame| {
            let outer_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(frame.area());

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(outer_chunks[0]);

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(9), Constraint::Min(0)])
                .split(chunks[1]);

            // Render panels
            frame.render_widget(memory_view(&app.bus, app.memory_page), chunks[0]);
            frame.render_widget(cpu_view(&app.cpu), right_chunks[0]);
            frame.render_widget(disassembly_view(&app.bus, app.cpu.pc()), right_chunks[1]); //[cite: 1]

            // Render Help text
            let help_text = " [Space] Step | [r] Reset | [Left/Right] Scroll Memory | [q] Quit";
            let help_widget = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::ALL).title("Help"));
            frame.render_widget(help_widget, outer_chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char(' ') => app.step(),
                KeyCode::Char('r') => app.cpu.reset(&app.bus),
                KeyCode::Char('q') => break,
                // Memory paging controls
                KeyCode::Right => {
                    app.memory_page = (app.memory_page + 1) % 256;
                }
                KeyCode::Left => {
                    app.memory_page = if app.memory_page == 0 {
                        255
                    } else {
                        app.memory_page - 1
                    };
                }
                _ => {}
            }
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

fn cpu_view(cpu: &Cpu) -> Paragraph<'_> {
    let status = cpu.status();

    let flag = |mask: u8, char: char| {
        if (status & mask) != 0 { char } else { ' ' }
    };

    let status_line = Line::from(format![
        "[{}{}{}{}{}{}{}{}]",
        flag(0x80, 'N'),
        flag(0x40, 'V'),
        flag(0x20, 'U'),
        flag(0x10, 'B'),
        flag(0x08, 'D'),
        flag(0x04, 'I'),
        flag(0x02, 'Z'),
        flag(0x01, 'C'),
    ]);

    let lines = vec![
        status_line,
        Line::from(""),
        Line::from(format!(
            "A: 0x{:02X} ({:>3})  X: 0x{:02X} ({:>3})  Y: 0x{:02X} ({:>3})",
            cpu.a(),
            cpu.a(),
            cpu.x(),
            cpu.x(),
            cpu.y(),
            cpu.y()
        )),
        Line::from(format!(
            "PC: 0x{:04X} ({:>5})  SP: 0x{:02X} ({:>3})",
            cpu.pc(),
            cpu.pc(),
            cpu.sp(),
            cpu.sp()
        )),
        Line::from(format!("Cycles: {}", cpu.cycles())),
    ];

    Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("CPU"))
}

fn disassembly_view(bus: &Bus, pc: u16) -> Paragraph<'_> {
    let mut lines = Vec::new();
    let mut addr = pc;

    for _ in 0..10 {
        let inst = disassembler::disassemble(bus, addr);

        let bytes = inst
            .bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let line = if inst.operand.is_empty() {
            format!("{:04X}  {:<8} {}", inst.address, bytes, inst.mnemonic)
        } else {
            format!(
                "{:04X}  {:<8} {:<3} {}",
                inst.address, bytes, inst.mnemonic, inst.operand
            )
        };

        lines.push(line);
        addr = inst.next;
    }

    Paragraph::new(lines.join("\n"))
        .block(Block::default().borders(Borders::ALL).title("Disassembly"))
}

fn memory_view(bus: &Bus, page: u16) -> Paragraph<'_> {
    let mut lines = Vec::new();

    let page_start = (page as usize) * 0x0100;

    for row in 0..16 {
        let base = page_start + (row * 16);
        let bytes: Vec<String> = (0..16)
            .map(|i| format!("{:02X}", bus.peek((base + i) as u16)))
            .collect();
        lines.push(format!("{:04X}: {}", base, bytes.join(" ")));
    }

    let title = format!("Memory (Page 0x{:02X})", page);
    Paragraph::new(lines.join("\n")).block(Block::default().borders(Borders::ALL).title(title))
}
