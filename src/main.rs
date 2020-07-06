mod chip8;

fn main() {
    let mut emu = chip8::Chip8::new();
    // emu.loadGame();

    loop {
        emu.emulate_cycle();
    }
}
