
extern crate rand;

use std::io::Cursor;

mod machine;
mod opcode;

use opcode::Opcode;
use machine::Chip8;

fn main() {
    let program = vec![
        // Set reg0 and reg1 to 5
        0x60, 0x05,
        0x61, 0x05,
       
        // Add them
        0x80, 0x14,
    ];

    let mut chip8 = Chip8::new();
    chip8.load_program(Cursor::new(program));
    println!("{:?}\n", chip8);

    // we have three instructions total
    for _ in 0..3 {
        chip8.cycle(); 
        println!("{:?}\n", chip8);
    }
}
