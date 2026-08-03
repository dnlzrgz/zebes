use macroquad::prelude::*;
use zebes_core::controller::*;
use zebes_core::nes::Nes;
use zebes_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

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

    let mut fps_counter = false;
    loop {
        // Display FPS counter.
        if is_key_pressed(KeyCode::F) {
            fps_counter = !fps_counter;
        }

        // Reset binding.
        if is_key_pressed(KeyCode::R) {
            nes.reset();
        }

        // Controller input.
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

        nes.set_controller_state(0, state);

        // Clock until a frame from the PPU completes.
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

        if fps_counter {
            let fps_text = format!("FPS: {:.0}", get_fps());
            draw_text(&fps_text, 12.0, 25.0, 24.0, YELLOW);
        }

        next_frame().await;
    }
}

#[macroquad::main("Zebes")]
async fn main() {
    let rom_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Use: zebes <rom.nes>");
        std::process::exit(1);
    });

    run(&rom_path).await;
}
