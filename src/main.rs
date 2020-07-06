mod chip8;

fn main() {
    let mut emu = chip8::Chip8::new();
    emu.load_game("pong.c8").unwrap();

    loop {
        emu.emulate_cycle();
    }
}
