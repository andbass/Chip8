
use std::thread;

use sdl2::{Sdl, SdlResult};
use sdl2::rect::Rect;
use sdl2::timer;
use sdl2::scancode::ScanCode;
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
        let mut key_arr = [false; 16];
        
        key_arr[0x1] = keys[&ScanCode::Num1];
        key_arr[0x2] = keys[&ScanCode::Num2];
        key_arr[0x3] = keys[&ScanCode::Num3];
        key_arr[0xC] = keys[&ScanCode::Num4];

        key_arr[0x4] = keys[&ScanCode::Q];
        key_arr[0x5] = keys[&ScanCode::W];
        key_arr[0x6] = keys[&ScanCode::E];
        key_arr[0xD] = keys[&ScanCode::R];

        key_arr[0x7] = keys[&ScanCode::A];
        key_arr[0x8] = keys[&ScanCode::S];
        key_arr[0x9] = keys[&ScanCode::D];
        key_arr[0xE] = keys[&ScanCode::F];

        key_arr[0xA] = keys[&ScanCode::Z];
        key_arr[0x0] = keys[&ScanCode::X];
        key_arr[0xB] = keys[&ScanCode::C];
        key_arr[0xF] = keys[&ScanCode::V];

        return key_arr;
    }

    fn emulate_loop(&mut self, mut chip8: Chip8) {
        let mut start_time = timer::get_ticks();
        
        'main: loop {
            for event in self.ctx.event_pump().poll_iter() {
                use sdl2::event::Event;

                match event {
                    Event::Quit { ..} => break 'main,
                    _ => (),
                }
            }
            
            if timer::get_ticks() - start_time > 5 {
                match chip8.cycle(self.get_keys()) {
                    Ok(_) => (),
                    Err(err) => panic!("{:?}", err),
                }

                println!("{:?}\n", chip8);
                start_time = timer::get_ticks();
            }

            self.draw(&chip8.screen);
        }
    }
}
