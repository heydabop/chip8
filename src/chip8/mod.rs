pub struct Chip8 {
    opcode: u16,        // current opcode
    memory: [u8; 4096], // system memory
    v: [u16; 16],       // registers V0-VE (VF is flag for some instructions)
    i: u16,             // address register
    pc: u16,            // program counter
    gfx: [u8; 64 * 32], // pixels state
    delay_timer: u8,
    sound_timer: u8, // timers count down at 60Hz
    stack: [u16; 16],
    sp: u16,       // stack pointer
    key: [u8; 16], // hex keypad state
}

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = [0; 4096];

        let chip8_fontset: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        // CHIP-8 systems had the interpreter in the first 512 bytes of memory
        // since we're emulating that we can just store the fontset there
        memory[..80].copy_from_slice(&chip8_fontset);

        Self {
            opcode: 0,
            memory,
            v: [0; 16],
            i: 0,
            pc: 0,
            gfx: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16],
        }
    }
}
