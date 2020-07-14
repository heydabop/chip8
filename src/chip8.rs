use rand::prelude::*;
use std::fs::File;
use std::io::prelude::*;

static SLEEP_MS: std::time::Duration = std::time::Duration::from_millis(3);

pub struct Chip8 {
    // CHIP-8 VM
    opcode: u16,        // current opcode
    memory: [u8; 4096], // system memory
    v: [u8; 16],        // registers V0-VE (VF is flag for some instructions)
    i: u16,             // address register
    pc: u16,            // program counter
    gfx: [u8; 64 * 32], // pixels state
    delay_timer: u8,
    sound_timer: u8, // timers count down at 60Hz
    stack: [u16; 16],
    sp: u16,       // stack pointer
    key: [u8; 16], // hex keypad state

    // emulator resources
    draw_flag: bool,
    rng: ThreadRng,
    timer_tick: u8, // since timers count at 60Hz but we run faster than that we'll only decrement when this timer is 0
    opcode_fns: [fn(&mut Self); 16],
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
            pc: 0x200, // programs start at 0x200
            gfx: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16],

            draw_flag: false,
            rng: rand::thread_rng(),
            timer_tick: 0,
            opcode_fns: [
                Self::cls_ret, // 00**
                Self::jmp,     // 1NNN
                Self::call,    // 2NNN
                Self::eb,      // 3XNN
                Self::neb,     // 4XNN
                Self::er,      // 5XY0
                Self::ld,      // 6XNN
                Self::addb,    // 7XNN
                Self::alu,     // 8XY*
                Self::ner,     // 9XY0
                Self::si,      // ANNN
                Self::jmpo,    // BNNN
                Self::rng,     // CXNN
                Self::draw,    // DXYN
                Self::key,     // EX**
                Self::ex,      // FX**
            ],
        }
    }

    pub fn load_game(&mut self, filename: &str) -> std::io::Result<()> {
        let mut file = File::open(filename)?;
        let _ = file.read(&mut self.memory[0x200..])?;
        Ok(())
    }

    pub fn draw_flag(&self) -> bool {
        self.draw_flag
    }

    pub fn gfx(&self) -> &[u8] {
        &self.gfx
    }

    pub fn sound_flag(&self) -> bool {
        self.sound_timer > 0
    }

    pub fn clear_keys(&mut self) {
        self.key = [0; 16];
    }

    pub fn press_key(&mut self, key: usize) {
        self.key[key] = 1;
    }

    pub fn emulate_cycle(&mut self) {
        std::thread::sleep(SLEEP_MS);

        let pc = self.pc as usize;
        // two-byte opcodes
        self.opcode = (self.memory[pc] as u16) << 8 | self.memory[pc + 1] as u16;

        #[cfg(debug_assertions)]
        println!("{:X}", self.opcode);

        self.draw_flag = false;

        let f = self.opcode_fns[((self.opcode & 0xF000) >> 12) as usize];
        f(self);

        if self.timer_tick == 0 {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }
        }
        self.timer_tick = (self.timer_tick + 1) % 5;

        #[cfg(debug_assertions)]
        {
            print!("[ ");
            for v in &self.v {
                print!("{:0>2X} ", v);
            }
            print!("]\n[ ");
            for s in &self.stack {
                print!("{:0>2X} ", s);
            }
            println!("]\nI: {:X}", self.i);
            print!("PC: {:X}\n[ ", self.pc);
            for b in &self.memory[0x200..0x300] {
                print!("{:0>2X} ", b);
            }
            println!("]\n");
        }
    }

    fn cls_ret(&mut self) {
        match self.opcode & 0xFF {
            0xE0 => {
                // 00E0
                // clear screen
                self.gfx = [0; 64 * 32];
                self.pc += 2;
            }
            0xEE => {
                // 00EE
                // return from subroutine
                if self.sp < 1 {
                    panic!("Hit opcode 0xEE with SP below 1");
                }
                self.sp -= 1;
                let sp = self.sp as usize;
                self.pc = self.stack[sp] + 2;
                self.stack[sp] = 0;
            }
            _ => panic!("Unhandled opcode {:X}", self.opcode),
        }
    }

    fn jmp(&mut self) {
        // 1NNN
        // jump to NNN
        self.pc = self.opcode & 0x0FFF;
    }

    fn call(&mut self) {
        // 2NNN
        // call subroutine at NNN
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = self.opcode & 0x0FFF;
    }

    fn eb(&mut self) {
        // 3XNN
        // skip if VX == NN
        let x = ((self.opcode & 0xF00) >> 8) as usize;
        let n = (self.opcode & 0xFF) as u8;
        self.pc += if self.v[x] == n { 4 } else { 2 };
    }

    fn neb(&mut self) {
        // 4XNN
        // skip if VX != NN
        let x = ((self.opcode & 0xF00) >> 8) as usize;
        let n = (self.opcode & 0xFF) as u8;
        self.pc += if self.v[x] != n { 4 } else { 2 };
    }

    fn er(&mut self) {
        // 5XY0
        // skip if VX == VY
        let x = ((self.opcode & 0xF00) >> 8) as usize;
        let y = ((self.opcode & 0xF0) >> 4) as usize;
        self.pc += if self.v[x] == self.v[y] { 4 } else { 2 };
    }

    fn ld(&mut self) {
        // 6XNN
        // set VX to NN
        let x = ((self.opcode & 0xF00) >> 8) as usize;
        let n = (self.opcode & 0xFF) as u8;
        self.v[x] = n;
        self.pc += 2;
    }

    fn addb(&mut self) {
        // 7XNN
        // add NN to VX (no carry)
        let x = ((self.opcode & 0xF00) >> 8) as usize;
        let n = (self.opcode & 0xFF) as u8;
        self.v[x] += n;
        self.pc += 2;
    }

    fn alu(&mut self) {
        let x = ((self.opcode & 0xF00) >> 8) as usize;
        let y = ((self.opcode & 0xF0) >> 4) as usize;
        match self.opcode & 0xF {
            0x0 => {
                // 8XY0
                // set VX to VY
                self.v[x] = self.v[y];
            }
            0x1 => {
                // 8XY1
                // set VX to VX OR VY
                self.v[x] |= self.v[y];
            }
            0x2 => {
                // 8XY2
                // set VX to VX AND VY
                self.v[x] &= self.v[y];
            }
            0x3 => {
                // 8XY3
                // set VX to VX XOR VY
                self.v[x] ^= self.v[y];
            }
            0x4 => {
                // 8XY4
                // add VY to VX (set VF = 1 if there's a carry)
                self.v[0xF] = if self.v[y] > 0xFF - self.v[x] { 1 } else { 0 };
                self.v[x] += self.v[y];
            }
            0x5 => {
                // 8XY5
                // sub VY from VX (set VF = 0 if there's a borrow and 1 if not)
                self.v[0xF] = if self.v[y] > self.v[x] { 0 } else { 1 };
                self.v[x] -= self.v[y];
            }
            0x6 => {
                // 8X06
                // store the LSB of VX in VF and shift VX one to the right
                self.v[0xF] = self.v[x] & 0x1;
                self.v[x] >>= 1;
            }
            0x7 => {
                // 8XY7
                // set VX to VY - VX (set VF = 0 if there's a borrow and 1 if not)
                self.v[0xF] = if self.v[x] > self.v[y] { 0 } else { 1 };
                self.v[x] = self.v[y] - self.v[x];
            }
            0xE => {
                // 8X0E
                // store the MSB of VX in VF and shift VX one to the left
                self.v[0xF] = if self.v[x] & 0x80 == 0x80 { 1 } else { 0 };
                self.v[x] <<= 1;
            }
            _ => panic!("Unhandled opcode {:X}", self.opcode),
        }
        self.pc += 2;
    }

    fn ner(&mut self) {
        // 9XY0
        // skip if VX != VY
        let x = ((self.opcode & 0xF00) >> 8) as usize;
        let y = ((self.opcode & 0xF0) >> 4) as usize;
        self.pc += if self.v[x] != self.v[y] { 4 } else { 2 };
    }

    fn si(&mut self) {
        // ANNN
        // set I to NNN
        self.i = self.opcode & 0xFFF;
        self.pc += 2;
    }

    fn jmpo(&mut self) {
        // BNNN
        // jump to NNN + V0
        let n = self.opcode & 0xFFF;
        self.pc = n + self.v[0] as u16;
    }

    fn rng(&mut self) {
        // CXNN
        // Set VX = RNG[0, 256) & NN
        let x = ((self.opcode & 0xF00) >> 8) as usize;
        let n = (self.opcode & 0xFF) as u8;
        self.v[x] = n & (self.rng.gen_range(0, 256) as u8);
        self.pc += 2;
    }

    fn draw(&mut self) {
        // DXYN
        // draw a sprite at VX,VY with a width of 8 pixels and a height of N pixels
        // each row of 8 pixels is bit-coded in memory starting at I
        // currently drawn pixels are XORd with pixels in memory
        // VF is set to 1 if any currently drawn pixels are unset during this
        let x = ((self.opcode & 0xF00) >> 8) as usize;
        let y = ((self.opcode & 0xF0) >> 4) as usize;
        let height = (self.opcode & 0xF) as usize;

        let vx = self.v[x] as usize;
        let vy = self.v[y] as usize;
        let i = self.i as usize;

        self.v[0xF] = 0; // gets set to 1 if any screen pixels are unset during draw
        for row in 0..height {
            let pixel = self.memory[i + row]; // load sprite starting at I
            for p in 0..8 {
                // iter bit shift across sprite pixel from memory
                if pixel & (0x80 >> p) != 0 {
                    // sprite pixel is set in memory
                    let gfx_offset = 64 * ((vy + row) % 32) + (vx + p) % 64;
                    self.gfx[gfx_offset] = if self.gfx[gfx_offset] == 1 {
                        // screen pixel is set and being unset
                        self.v[0xF] = 1;
                        0
                    } else {
                        // screen pixel isn't set and is being set
                        1
                    };
                }
            }
        }

        self.draw_flag = true;
        self.pc += 2;
    }

    fn key(&mut self) {
        let x = ((self.opcode & 0xF00) >> 8) as usize;
        let pressed = self.key[self.v[x] as usize] == 1;
        match self.opcode & 0xFF {
            0x9E => {
                // 0xEX9E
                // skip if key stored in VX is pressed
                self.pc += if pressed { 4 } else { 2 };
            }
            0xA1 => {
                // 0xEXA1
                // skip if key stored in VX isn't pressed
                self.pc += if !pressed { 4 } else { 2 };
            }

            _ => panic!("Unhandled opcode {:X}", self.opcode),
        }
    }

    fn ex(&mut self) {
        let x = ((self.opcode & 0xF00) >> 8) as usize;
        match self.opcode & 0xFF {
            0x7 => {
                // 0xFX07
                // set VX to delay timer
                self.v[x] = self.delay_timer;
            }
            0xA => {
                // 0xFX0A
                // store next key press in VX, blocking instruction
                let mut pressed = false;
                // check all keys recording the first pressed one
                for i in 0..0xF as u8 {
                    if self.key[i as usize] == 1 {
                        pressed = true;
                        self.v[x] = i;
                        break;
                    }
                }
                if !pressed {
                    self.pc -= 2; // repeat this instruction if no pressed key
                }
            }
            0x15 => {
                // 0xFX15
                // set delay timer to vx
                self.delay_timer = self.v[x] as u8;
            }
            0x18 => {
                // 0xFX18
                // set sound timer to vx
                self.sound_timer = self.v[x] as u8;
            }
            0x1E => {
                // 0xFX1E
                // add VX to I
                self.i += self.v[x] as u16;
            }
            0x29 => {
                // 0xFX29
                // set I to location in memory of sprite for character in VX
                self.i = 5 * self.v[x] as u16; // we're storing fontset in the first 80 bytes, 5 bytes per sprite
            }
            0x33 => {
                // 0xFX33
                // store the BCD representation of VX at I
                // so 193 becomes [1, 9, 3] in memory at I
                let vx = self.v[x];
                let i = self.i as usize;
                self.memory[i] = (vx / 100) as u8;
                self.memory[i + 1] = ((vx / 10) % 10) as u8;
                self.memory[i + 2] = ((vx % 100) % 10) as u8;
            }
            0x55 => {
                // 0xFX55
                // store V0 to VX (inclusive) in memory at I
                let i = self.i as usize;
                for offset in 0..=x {
                    self.memory[i + offset] = self.v[offset] as u8;
                }
            }
            0x65 => {
                // 0xFX65
                // fill V0 to VX (inclusive) from memory at I
                let i = self.i as usize;
                for offset in 0..=x {
                    self.v[offset] = self.memory[i + offset];
                }
            }
            _ => panic!("Unhandled opcode {:X}", self.opcode),
        }
        self.pc += 2;
    }
}
