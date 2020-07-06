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
        .window("CHIP-8", 64, 32)
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
            for (i, p) in gfx.iter().enumerate() {
                let i = i as i16;
                canvas
                    .pixel(i % 64, i / 64, if *p != 0 { white } else { black })
                    .unwrap();
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
