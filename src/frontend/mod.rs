
use machine::Chip8;

pub trait Frontend {
    fn draw(&mut self, screen: &[[bool; 64]; 32]);
    fn get_keys(&mut self) -> [bool; 16];
}

pub mod curses;

pub use self::curses::Terminal;
