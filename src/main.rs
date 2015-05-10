
extern crate rand;
extern crate sdl2;

use std::env;
use std::fs;
use std::thread;
use std::io::Cursor;

mod machine;
mod opcode;
mod frontend;

use machine::Chip8;
use frontend::{SdlFrontend, Frontend};

fn main() {
    let path = env::args().nth(1).unwrap();

    
    let program = [
        0x00, 0xE0,
    
        // set v0 and v1 to 0
        0x60, 0x00,
        0x61, 0x00,

        // Set v2 to A, and set I to the char of v2
        0x62, 0x0A,
        0xF3, 0x29,

        0xD0, 0x15,

        // loop
        0x12, 0x00
    ];

    let file = Cursor::new(&program[..]);

    let file = fs::File::open(path).unwrap();
    
    let mut chip8 = Chip8::new();
    let mut sdl = SdlFrontend::new(sdl2::init(sdl2::INIT_EVERYTHING).unwrap()).unwrap();

    chip8.load_program(file);
    
    sdl.emulate_loop(chip8);
}
