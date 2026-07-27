use macroquad::prelude::*;
use zebes_core::nes::Nes;

#[macroquad::main("Zebes")]
async fn main() {
    let rom_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Use: zebes <rom.nes>");
        std::process::exit(1);
    });

    let rom = std::fs::read(&rom_path)
        .unwrap_or_else(|err| panic!("Failed to read ROM at {rom_path}: {err}"));

    let mut nes = Nes::new();
    nes.load(&rom)
        .unwrap_or_else(|err| panic!("Failed to load cartridge: {err}"));
    nes.reset();

    loop {
        // Clock until a frame completes
        let start_frame = nes.bus().ppu.frame();
        while nes.bus().ppu.frame() == start_frame {
            nes.clock();
        }

        let fb = nes.bus().ppu.framebuffer();
        let mut rgba = Vec::with_capacity(fb.len() * 4);
        for px in fb.iter() {
            rgba.extend_from_slice(&[px.r, px.g, px.b, 255]);
        }

        let texture = Texture2D::from_rgba8(256, 240, &rgba);
        texture.set_filter(FilterMode::Nearest);
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
