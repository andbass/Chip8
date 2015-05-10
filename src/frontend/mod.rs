
use machine::Chip8;

pub trait Frontend {
    fn draw(&mut self, screen: &[[bool; 64]; 32]);
    fn get_keys(&mut self) -> [bool; 16];

    fn emulate_loop(&mut self, Chip8);
}

mod sdl;

pub use self::sdl::SdlFrontend;
