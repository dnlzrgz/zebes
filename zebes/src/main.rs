use std::time::{Duration, Instant};

use macroquad::prelude::*;
use zebes_core::controller::*;
use zebes_core::nes::Nes;
use zebes_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

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

fn emulate_frames(nes: &mut Nes, image: &mut Image, controller_state: u8) {
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

    // NES NTSC frame rate (real is 60.0988138974405 Hz)
    let target_frame_time = Duration::from_secs_f64(1.0 / 60.0988);
    // Time after is assumed that the process was stalled and reset the accumulator.
    let max_delta_time: Duration = Duration::from_millis(250);

    // Fixed-step accumulator
    let mut accumulator = Duration::ZERO;
    let mut last_time = Instant::now();

    loop {
        if is_key_pressed(KeyCode::R) {
            nes.reset();
        }

        let current_time = Instant::now();
        let mut dt = current_time - last_time;
        last_time = current_time;

        if dt > max_delta_time {
            dt = Duration::ZERO;
            accumulator = Duration::ZERO;
        }
        accumulator += dt;

        let controller_state = get_controller_state();

        while accumulator >= target_frame_time {
            emulate_frames(&mut nes, &mut image, controller_state);
            accumulator -= target_frame_time;
        }

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
