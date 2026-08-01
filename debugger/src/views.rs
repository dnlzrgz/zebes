use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

use zebes_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

use crate::{
    app::{App, ViewMode},
    disassembler::disassemble,
};

pub fn render(frame: &mut Frame, app: &App) {
    let [main, footer] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .areas(frame.area());

    match app.view_mode {
        ViewMode::Cpu => {
            let [left, right] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .areas(main);

            let [top_left, bottom_left] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .areas(left);

            let [top_right, bottom_right] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .areas(right);

            render_ppu(frame, top_left, app);
            render_memory(frame, bottom_left, app);
            render_cpu(frame, top_right, app);
            render_disassembly(frame, bottom_right, app);
        }
        ViewMode::Ppu => {
            let [left, right] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .areas(main);

            let [top_right, bottom_right] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
                .areas(right);

            render_ppu(frame, left, app);
            render_ppu_registers(frame, top_right, app);
            render_oam(frame, bottom_right, app);
        }
    }

    render_footer(frame, footer, app);
}

fn block(title: String) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
}

/// Displays the current contents of the PPU's framebuffer.
///
/// Every visible NES pixel is stored as one RGB color (256x240) but since a terminal character cell
/// is taller than it is wide, rendering one character per pixel would distort the image too much.
/// To solve this we use the unicode upper half block. The foreground color represents the upper NES
/// pixel while the background color represents the lower one, allowing each terminal row to display
/// two vertical pixels.
fn render_ppu(frame: &mut Frame, area: Rect, app: &App) {
    let cols = area.width.saturating_sub(2) as usize;
    let rows = area.height.saturating_sub(2) as usize;

    if cols == 0 || rows == 0 {
        frame.render_widget(block("ppu".into()), area);
        return;
    }

    let framebuffer = app.nes.bus().ppu.framebuffer();
    let pixel_rows = rows * 2; // 2 NES pixel-rows per terminal row

    let mut lines = Vec::with_capacity(rows);
    for row in 0..rows {
        let mut spans = Vec::with_capacity(cols);
        let top_y = (row * 2 * SCREEN_HEIGHT) / pixel_rows;
        let bottom_y = ((row * 2 + 1) * SCREEN_HEIGHT) / pixel_rows;

        for col in 0..cols {
            let x = (col * SCREEN_WIDTH) / cols;
            let top = framebuffer[top_y.min(SCREEN_HEIGHT - 1) * SCREEN_WIDTH + x];
            let bottom = framebuffer[bottom_y.min(SCREEN_HEIGHT - 1) * SCREEN_WIDTH + x];

            spans.push(Span::styled(
                "▀",
                Style::default()
                    .fg(Color::Rgb(top.r, top.g, top.b))
                    .bg(Color::Rgb(bottom.r, bottom.g, bottom.b)),
            ));
        }

        lines.push(Line::from(spans));
    }

    let title = format!("ppu  (frame {})", app.nes.bus().ppu.frame());
    frame.render_widget(Paragraph::new(lines).block(block(title)), area);
}

/// Displays a hexadecimal and scrollable view of the CPU address space.
fn render_memory(frame: &mut Frame, area: Rect, app: &App) {
    let bus = app.nes.bus();
    let rows_visible = area.height.saturating_sub(2);
    let mut lines = Vec::with_capacity(rows_visible as usize);

    for row in 0..rows_visible {
        let row_addr = app.mem_base.wrapping_add(row * 16);
        let mut spans = vec![Span::styled(
            format!("${row_addr:04X} "),
            Style::default().add_modifier(Modifier::BOLD),
        )];

        for col in 0..16u16 {
            let value = bus.peek(row_addr.wrapping_add(col));
            spans.push(Span::raw(format!("{value:02X} ")));
        }
        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines).block(block("memory".into())), area);
}

/// Displays the current CPU states.
fn render_cpu(frame: &mut Frame, area: Rect, app: &App) {
    let cpu = app.nes.cpu();
    let p = cpu.status();
    let flag = |mask: u8, ch: char| if p & mask != 0 { ch } else { '_' };
    let flags = format!(
        "{}{}{}{}{}{}{}{}",
        flag(0x80, 'N'),
        flag(0x40, 'V'),
        flag(0x20, 'U'),
        flag(0x10, 'B'),
        flag(0x08, 'D'),
        flag(0x04, 'I'),
        flag(0x02, 'Z'),
        flag(0x01, 'C'),
    );

    let lines = vec![
        Line::from(format!("[{flags}]")),
        Line::from(format!(
            "A:  ${:02X}   X: ${:02X}   Y: ${:02X}",
            cpu.a(),
            cpu.x(),
            cpu.y()
        )),
        Line::from(format!("PC: ${:04X}", cpu.pc())),
        Line::from(format!("SP: ${:02X}", cpu.sp())),
        Line::from(""),
        Line::from(format!(
            "CYC: {}  (rem: {})",
            cpu.total_cycles(),
            cpu.cycles()
        )),
    ];

    frame.render_widget(Paragraph::new(lines).block(block("Cpu".into())), area);
}

/// Displays the current Ppu state.
fn render_ppu_registers(frame: &mut Frame, area: Rect, app: &App) {
    let ppu = &app.nes.bus().ppu;

    let lines = vec![
        Line::from(format!("CTRL:   ${:02X}", ppu.ctrl())),
        Line::from(format!("MASK:   ${:02X}", ppu.mask())),
        Line::from(format!("STATUS: ${:02X}", ppu.status())),
        Line::from(""),
        Line::from(format!(
            "POS: {:>3}, {:>3} (Scanline, Cycle)",
            ppu.scanline(),
            ppu.cycle()
        )),
        Line::from(""),
        Line::from(format!("V (Current VRAM): ${:04X}", ppu.v())),
        Line::from(format!("T (Temp VRAM):    ${:04X}", ppu.t())),
        Line::from(format!("X (Fine X):       ${:02X}", ppu.x())),
        Line::from(format!(
            "W (Write Toggle): {}",
            if ppu.w() { "2nd" } else { "1st" }
        )),
    ];

    frame.render_widget(Paragraph::new(lines).block(block("Registers".into())), area);
}

fn render_oam(frame: &mut Frame, area: Rect, app: &App) {
    let oam = app.nes.bus().ppu.oam();
    let mut rows = Vec::new();

    for i in 0..64 {
        let base = i * 4;
        let y = oam[base];
        let tile = oam[base + 1];
        let attr = oam[base + 2];
        let x = oam[base + 3];

        rows.push(Row::new(vec![
            Cell::from(format!("{:02X}", i)),
            Cell::from(format!("{:02X}", y)),
            Cell::from(format!("{:02X}", tile)),
            Cell::from(format!("{:02X}", attr)),
            Cell::from(format!("{:02X}", x)),
        ]));
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(4),
        ],
    )
    .header(Row::new(vec!["ID", "Y", "TILE", "ATTR", "X"]))
    .block(block("OAM (Sprites)".into()));

    frame.render_widget(table, area);
}

/// Disasemble instructions beginning at the current program counter (`pc`).
fn render_disassembly(frame: &mut Frame, area: Rect, app: &App) {
    let bus = app.nes.bus();
    let cpu = app.nes.cpu();
    let rows_visible = area.height.saturating_sub(2) as usize;
    let mut lines = Vec::with_capacity(rows_visible);
    let mut addr = cpu.pc();

    for i in 0..rows_visible {
        let decoded = disassemble(bus, addr, cpu.x(), cpu.y());
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

        let style = if i == 0 {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        lines.push(Line::from(Span::styled(
            format!("${:04X}  {:<8}  {}", decoded.address, bytes_str, asm),
            style,
        )));
        addr = decoded.next;
    }

    frame.render_widget(
        Paragraph::new(lines).block(block("Disassembly".into())),
        area,
    );
}

/// Shows the current execution mode as well as the available keybindings.
fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let mode = if app.running { "RUN" } else { "STEP" };
    let text = format!(
        "[{mode}] [enter]:run/pause  [space]:step cycle  [n]:step instr  [r]:reset  [v]:toggle view  ↑↓:scroll mem  q:quit"
    );
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::Gray)),
        area,
    );
}
