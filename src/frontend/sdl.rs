
use std::thread;

use sdl2::{Sdl, SdlResult};
use sdl2::rect::Rect;
use sdl2::video::{Window, WindowPos, OPENGL};
use sdl2::render::{RenderDriverIndex, ACCELERATED, Renderer};
use sdl2::keyboard;
use sdl2::pixels::Color;

use super::Frontend;
use machine::Chip8;

const GRID_SIZE: i32 = 20;

pub struct SdlFrontend<'a> {
    ctx: Sdl,
    renderer: Renderer<'a>,
}

impl<'a> SdlFrontend<'a> {
    pub fn new(ctx: Sdl) -> SdlResult<SdlFrontend<'a>> {
        let window = try!(Window::new(&ctx, "Chip8", WindowPos::PosCentered, WindowPos::PosCentered, GRID_SIZE * 64, GRID_SIZE * 32, OPENGL));
        let renderer = try!(Renderer::from_window(window, RenderDriverIndex::Auto, ACCELERATED));

        Ok(SdlFrontend {
            ctx: ctx,
            renderer: renderer,
        })
    }
}

impl<'a> Frontend for SdlFrontend<'a> {
    fn draw(&mut self, screen: &[[bool; 64]; 32]) {
        let mut drawer = self.renderer.drawer(); 

        drawer.set_draw_color(Color::RGB(0, 0, 0));
        drawer.clear();
        drawer.set_draw_color(Color::RGB(255, 255, 255));

        for (y, row) in screen.iter().enumerate() {
            for (x, elem) in row.iter().enumerate() {
                if *elem {
                    drawer.fill_rect(Rect {
                        x: x as i32 * GRID_SIZE,
                        y: y as i32 * GRID_SIZE,

                        w: GRID_SIZE,
                        h: GRID_SIZE,
                    });
                }
            }
        }

        drawer.present();
    }

    fn get_keys(&mut self) -> [bool; 16] {
        let keys = keyboard::get_keyboard_state(); 


        [false; 16]
    }

    fn emulate_loop(&mut self, mut chip8: Chip8) {
        'main: loop {
            for event in self.ctx.event_pump().poll_iter() {
                use sdl2::event::Event;

                match event {
                    Event::Quit { ..} => break 'main,
                    _ => (),
                }
            }

            match chip8.cycle(self.get_keys()) {
                Ok(_) => (),
                Err(err) => panic!("{:?}", err),
            }

            self.draw(&chip8.screen);

            thread::sleep_ms(17);
        }
    }
}
