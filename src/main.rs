extern crate sdl2;

mod chip8;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels;
use sdl2::rect::Rect;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: {} <path to ROM>",
            if !args.is_empty() {
                &args[0]
            } else {
                "<program>"
            }
        );
        std::process::exit(1);
    }

    let keypad = [
        Scancode::X,    // 0
        Scancode::Num1, // 1
        Scancode::Num2, // 2
        Scancode::Num3, // 3
        Scancode::Q,    // 4
        Scancode::W,    // 5
        Scancode::E,    // 6
        Scancode::A,    // 7
        Scancode::S,    // 8
        Scancode::D,    // 9
        Scancode::Z,    // A
        Scancode::C,    // B
        Scancode::Num4, // C
        Scancode::R,    // D
        Scancode::F,    // E
        Scancode::V,    // F
    ];

    let sdl_ctx = sdl2::init().unwrap();
    let video = sdl_ctx.video().unwrap();

    let scale = 4;
    let window = video
        .window("CHIP-8", 64 * scale, 32 * scale)
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
    emu.load_game(&args[1]).unwrap();

    let sleep = std::time::Duration::from_millis(3);

    'main: loop {
        emu.emulate_cycle();

        if emu.draw_flag() {
            let gfx = emu.gfx();
            canvas.set_draw_color(black);
            canvas.clear();
            canvas.set_draw_color(white);
            let mut rects = Vec::new();
            for (i, p) in gfx.iter().enumerate() {
                if *p == 0 {
                    continue;
                }
                let i = i as i32;
                let x = (i % 64) * scale as i32;
                let y = (i / 64) * scale as i32;
                rects.push(Rect::new(x, y, scale, scale));
            }
            canvas.fill_rects(&rects).unwrap();
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

        emu.clear_keys();

        for key in event_pump.keyboard_state().pressed_scancodes() {
            if let Some(i) = keypad.iter().position(|&k| k == key) {
                emu.press_key(i);
            }
        }

        std::thread::sleep(sleep);
    }
}
