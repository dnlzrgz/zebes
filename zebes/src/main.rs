use std::time::{Duration, Instant};

use macroquad::prelude::*;
use zebes_core::controller::*;
use zebes_core::nes::Nes;
use zebes_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

// NTSC frame length: 341x262 dots at 5.369318 MHz. Odd frames are one dot shorter.
const FRAME_TIME: Duration = Duration::from_nanos(16_639_264);

fn window_conf() -> Conf {
    Conf {
        window_title: "Zebes".to_owned(),
        window_width: (SCREEN_WIDTH * 2) as i32,
        window_height: (SCREEN_HEIGHT * 2) as i32,
        ..Default::default()
    }
}

fn get_controller_state() -> u8 {
    let mut state = 0u8;
    if is_key_down(KeyCode::Z) {
        state |= BUTTON_A;
    }
    if is_key_down(KeyCode::X) {
        state |= BUTTON_B;
    }
    if is_key_down(KeyCode::Space) {
        state |= BUTTON_SELECT;
    }
    if is_key_down(KeyCode::Enter) {
        state |= BUTTON_START;
    }
    if is_key_down(KeyCode::Up) {
        state |= BUTTON_UP;
    }
    if is_key_down(KeyCode::Down) {
        state |= BUTTON_DOWN;
    }
    if is_key_down(KeyCode::Left) {
        state |= BUTTON_LEFT;
    }
    if is_key_down(KeyCode::Right) {
        state |= BUTTON_RIGHT;
    }

    state
}

fn emulate_frame(nes: &mut Nes, image: &mut Image, controller_state: u8) {
    nes.set_controller_state(0, controller_state);

    let start_frame = nes.bus().ppu.frame();
    while nes.bus().ppu.frame() == start_frame {
        nes.clock();
    }

    let fb = nes.bus().ppu.framebuffer();
    for (px, out) in fb.iter().zip(image.bytes.chunks_exact_mut(4)) {
        out[0] = px.r;
        out[1] = px.g;
        out[2] = px.b;
        out[3] = 255;
    }
}

pub async fn run(rom_path: &str) {
    let rom = std::fs::read(&rom_path)
        .unwrap_or_else(|err| panic!("Failed to read ROM at {rom_path}: {err}"));

    let mut nes = Nes::new();
    nes.load(&rom)
        .unwrap_or_else(|err| panic!("Failed to load cartridge: {err}"));
    nes.reset();

    let mut image = Image::gen_image_color(SCREEN_WIDTH as u16, SCREEN_HEIGHT as u16, BLACK);
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);

    let mut next_deadline = Instant::now();

    loop {
        if is_key_pressed(KeyCode::R) {
            nes.reset();
        }

        let controller_state = get_controller_state();
        emulate_frame(&mut nes, &mut image, controller_state);

        texture.update(&image);

        draw_texture_ex(
            &texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        next_deadline += FRAME_TIME;
        let now = Instant::now();
        match next_deadline.checked_duration_since(now) {
            Some(remaining) => spin_sleep::sleep(remaining),
            None => next_deadline = now,
        }

        next_frame().await;
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let rom_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Use: zebes <rom.nes>");
        std::process::exit(1);
    });

    run(&rom_path).await;
}
