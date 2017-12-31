
extern crate rand;
extern crate sdl2;

use std::env;
use std::fs;

pub mod machine;
pub mod opcode;
pub mod frontend;

use machine::Chip8;
use frontend::{SdlFrontend, Frontend};

fn main() {
    let path = env::args().nth(1).unwrap();
    
    let file = fs::File::open(&path).unwrap_or_else(|err| {
        panic!("Could not open program '{}': {}", path, err);
    });
    
    let mut chip8 = Chip8::new();
    let mut sdl = SdlFrontend::new(sdl2::init().unwrap());

    chip8.load_program(file).unwrap_or_else(|err| {
        panic!("Could not load program '{}': {}", path, err);
    });
    
    sdl.emulate_loop(chip8);
}
