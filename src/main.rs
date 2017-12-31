
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
    
    let file = match fs::File::open(&path) {
        Ok(file) => file,
        Err(err) => { 
            println!("Could not open {}: {:?}", path, err);
            return;
        }
    };
    
    let mut chip8 = Chip8::new();
    let mut sdl = match SdlFrontend::new(sdl2::init().unwrap()) {
        Ok(frontend) => frontend,
        Err(err) => {
            println!("Could not create SdlFrontend: {:?}", err); 
            return;
        }
    };

    match chip8.load_program(file) {
        Ok(_) => (),
        Err(err) => {
            println!("Could not load program: {:?}", err);
            return;
        }
    }
    
    sdl.emulate_loop(chip8);
}
