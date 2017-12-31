
use sdl2;
use sdl2::{Sdl, EventPump};
use sdl2::rect::Rect;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::video::{Window, WindowPos};
use sdl2::render::{WindowCanvas};
use sdl2::keyboard;
use sdl2::pixels::Color;

use super::Frontend;
use machine::Chip8;

const GRID_SIZE: i32 = 20;

pub struct SdlFrontend {
    ctx: Sdl,
    renderer: WindowCanvas,
    events: EventPump,
}

impl SdlFrontend {
    pub fn new(ctx: Sdl) -> SdlFrontend {
        let video = ctx.video().unwrap();
        let window = video.window("Chip8", (GRID_SIZE * 64) as u32, (GRID_SIZE * 32) as u32)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let renderer = window.into_canvas().build().unwrap();
        let events = ctx.event_pump().unwrap();

        SdlFrontend {
            ctx: ctx,
            renderer: renderer,
            events: events,
        }
    }
}

impl Frontend for SdlFrontend {
    fn draw(&mut self, screen: &[[bool; 64]; 32]) {
        let mut drawer = &mut self.renderer;

        drawer.set_draw_color(Color::RGB(0, 0, 0));
        drawer.clear();
        drawer.set_draw_color(Color::RGB(255, 255, 255));

        for (y, row) in screen.iter().enumerate() {
            for (x, elem) in row.iter().enumerate() {
                if *elem {
                    drawer.fill_rect(Rect::new(
                        x as i32 * GRID_SIZE,
                        y as i32 * GRID_SIZE,

                        GRID_SIZE as u32,
                        GRID_SIZE as u32,
                    ));
                }
            }
        }

        drawer.present();
    }

    fn get_keys(&mut self) -> [bool; 16] {
        let keys = self.events.keyboard_state(); 
        let mut key_arr = [false; 16];
        
        key_arr[0x1] = keys.is_scancode_pressed(Scancode::Num1);
        key_arr[0x2] = keys.is_scancode_pressed(Scancode::Num2);
        key_arr[0x3] = keys.is_scancode_pressed(Scancode::Num3);
        key_arr[0xC] = keys.is_scancode_pressed(Scancode::Num4);

        key_arr[0x4] = keys.is_scancode_pressed(Scancode::Q);
        key_arr[0x5] = keys.is_scancode_pressed(Scancode::W);
        key_arr[0x6] = keys.is_scancode_pressed(Scancode::E);
        key_arr[0xD] = keys.is_scancode_pressed(Scancode::R);

        key_arr[0x7] = keys.is_scancode_pressed(Scancode::A);
        key_arr[0x8] = keys.is_scancode_pressed(Scancode::S);
        key_arr[0x9] = keys.is_scancode_pressed(Scancode::D);
        key_arr[0xE] = keys.is_scancode_pressed(Scancode::F);

        key_arr[0xA] = keys.is_scancode_pressed(Scancode::Z);
        key_arr[0x0] = keys.is_scancode_pressed(Scancode::X);
        key_arr[0xB] = keys.is_scancode_pressed(Scancode::C);
        key_arr[0xF] = keys.is_scancode_pressed(Scancode::V);

        return key_arr;
    }

    fn emulate_loop(&mut self, mut chip8: Chip8) {
        let mut paused = false;
        let mut step = false;

        let mut saved_state: Chip8 = chip8.clone();

        let mut timer = self.ctx.timer().unwrap();
        let mut start_time = timer.ticks();
        
        'main: loop {
            for event in self.events.poll_iter() {
                use sdl2::event::Event;

                match event {
                    Event::Quit { .. } => break 'main,

                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        paused = !paused;
                        println!("{}", if paused { "Now paused" } else { "Resumed" });
                    },
                    Event::KeyDown { keycode: Some(Keycode::Space), .. } => step = true,

                    Event::KeyDown { keycode: Some(Keycode::I), .. } => println!("\n{:?}\n", chip8),

                    Event::KeyDown { keycode: Some(Keycode::F5), .. } => {
                        saved_state = chip8.clone();
                        println!("State saved!\n")
                    },
                    Event::KeyDown { keycode: Some(Keycode::F6), .. } => {
                        chip8 = saved_state.clone();
                        println!("State restored!\n");
                    },

                    Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                        if chip8.speed - 1 >= 0 {
                            chip8.speed -= 1;
                            println!("Speed: {}", chip8.speed);
                        }
                    },

                    Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                        chip8.speed += 1;
                        println!("Speed: {}", chip8.speed);
                    },
                    
                    _ => (),
                }
            }
            
            if (!paused && timer.ticks() - start_time > 17) || step {
                match chip8.cycle(self.get_keys()) {
                    Ok(_) => (),
                    Err(err) => panic!("{:?}", err),
                }

                start_time = timer.ticks();
                step = false;
            }

            self.draw(&chip8.screen);
        }
    }
}
