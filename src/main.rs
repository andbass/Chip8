
extern crate rand;
extern crate ncurses;

use std::env;
use std::fs;

mod machine;
mod opcode;
mod frontend;

use machine::Chip8;
use frontend::Terminal;

fn main() {
    let path = match env::args.nth(1).unwrap();

    let file = fs::File::open(path).unwrap();

    let mut chip8 = Chip8::new();
    let mut term = Terminal::new();

    chip8.load_program(file);

    loop {
        chip8.cycle(&mut term);
    }
}
