
use super::Frontend;

use ncurses::*;

const DRAW_CHAR: &'static str = "█";

pub struct Terminal {
    width: i32,
    height: i32,
}

impl Terminal {
    fn new() -> Terminal {
        initscr();

        noecho();
        nodelay(stdscr, true);
        keypad(stdscr, true);

        let mut width = 0;
        let mut height = 0;

        getmaxyx(stdscr, &mut height, &mut width);

        Terminal {
            width: width,
            height: height,
        }
    }
}

impl Frontend for Terminal {
    fn draw(&mut self, screen: &[[bool; 64]; 32]) {
        clear();

        for (y, row) in screen.iter().enumerate() {
            for (x, elem) in row.iter().enumerate() {
                if *elem {
                    mvprintw(y as i32, x as i32, DRAW_CHAR);
                }
            }
        }
    }

    fn get_keys(&mut self) -> [bool; 16] {
        let mut keys = [false; 16];

        let ch = (getch() as u8) as char;
        let key = match ch {
            '1' ... '3' => Some(ch as usize),
            '4' => Some(0xC),

            'q' => Some(0x4),
            'w' => Some(0x5),
            'e' => Some(0x6),
            'r' => Some(0xD),

            'a' => Some(0x7),
            's' => Some(0x8),
            'd' => Some(0x9),
            'f' => Some(0xE),

            'z' => Some(0xA),
            'x' => Some(0x0),
            'c' => Some(0xB),
            'v' => Some(0xF),
            
            _ => None,
        };

        if let Some(key) = key {
            keys[key] = true; 
        }

        keys
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        endwin();
    }
}
