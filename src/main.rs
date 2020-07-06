extern crate sdl2;

mod chip8;

use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels;

fn main() {
    let sdl_ctx = sdl2::init().unwrap();
    let video = sdl_ctx.video().unwrap();

    let window = video
        .window("CHIP-8", 64 * 2, 32 * 2)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    let black = pixels::Color::RGB(0, 0, 0);
    let white = pixels::Color::RGB(255, 255, 255);
    canvas.set_draw_color(black);
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_ctx.event_pump().unwrap();

    let mut emu = chip8::Chip8::new();
    emu.load_game("pong.c8").unwrap();

    let sleep = std::time::Duration::from_millis(16);

    'main: loop {
        emu.emulate_cycle();

        if emu.draw_flag() {
            let gfx = emu.gfx();
            canvas.set_draw_color(black);
            canvas.clear();
            canvas.set_draw_color(white);
            for (i, p) in gfx.iter().enumerate() {
                if *p == 0 {
                    continue;
                }
                let i = i as i16;
                let x = (i % 64) * 2;
                let y = (i / 64) * 2;
                canvas.pixel(x, y, white).unwrap();
                canvas.pixel(x + 1, y, white).unwrap();
                canvas.pixel(x, y + 1, white).unwrap();
                canvas.pixel(x + 1, y + 1, white).unwrap();
            }
            canvas.present();
        }

        for e in event_pump.poll_iter() {
            match e {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                _ => {}
            }
        }

        std::thread::sleep(sleep);
    }
}
