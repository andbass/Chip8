
use rand::{thread_rng, Rng};

use std::io;
use std::fmt;

use opcode::{Opcode, OpcodeError, SetRegMode};

const PROGRAM_START: u16 = 0x200;
const FONT_START: u16 = 0x50;

const MEMORY_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;

// Thanks to: http://www.multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/
const FONTMAP: [u8; 80] = [
  0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
  0x20, 0x60, 0x20, 0x20, 0x70, // 1
  0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
  0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
  0x90, 0x90, 0xF0, 0x10, 0x10, // 4
  0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
  0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
  0xF0, 0x10, 0x20, 0x40, 0x40, // 7
  0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
  0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
  0xF0, 0x90, 0xF0, 0x90, 0x90, // A
  0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
  0xF0, 0x80, 0x80, 0x80, 0xF0, // C
  0xE0, 0x90, 0x90, 0x90, 0xE0, // D
  0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
  0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

#[derive(Debug)]
pub enum RuntimeError {
    EmptyCallStack,
    InvalidRegister(u8),
    AddressOutOfBounds(u16),
    OpcodeErr(OpcodeError),
}

pub struct Chip8 {
    pub memory: [u8; MEMORY_SIZE],
    pub regs: [u8; REGISTER_COUNT], // registers V0 - V15
    pub addressReg: u16, // register I

    pub pc: u16,
    pub stack: Vec<u16>,

    pub delay_timer: u16,
    pub sound_timer: u16,

    pub screen: [[bool; 64]; 32],
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut chip8 = Chip8 {
            memory: [0; 4096],
            regs: [0; 16],
            addressReg: 0,
            
            pc: PROGRAM_START,
            stack: Vec::new(),

            delay_timer: 0,
            sound_timer: 0,

            screen: [[false; 64]; 32],
        };

        chip8.inject_fontmap();
        chip8
    }

    pub fn inject_fontmap(&mut self) {
        for (offset, byte) in FONTMAP.iter().enumerate() {
            self.memory[FONT_START as usize + offset] = *byte;
        }
    }

    pub fn load_program<R: io::Read>(&mut self, mut program: R) -> io::Result<()> {
        let mut bytes = Vec::new();
        try!(program.read_to_end(&mut bytes));
        
        for (offset, byte) in bytes.iter().enumerate() {
            self.memory[offset + PROGRAM_START as usize] = *byte;
        }

        Ok(())
    }

    pub fn cycle(&mut self, keys: [bool; 16]) -> Result<(), RuntimeError> {
        use self::RuntimeError::*;

        let pc_index = self.pc as usize;
        let opcode_bytes = (self.memory[pc_index] as u16) << 8 | (self.memory[pc_index + 1] as u16);

        let opcode = match Opcode::from_u16(opcode_bytes) {
            Ok(opcode) => opcode,
            Err(err) => return Err(OpcodeErr(err)),
        };

        self.pc += 2;
        try!(self.execute_opcode(opcode, keys));

        Chip8::decrement_timer(&mut self.delay_timer);
        Chip8::decrement_timer(&mut self.sound_timer);

        Ok(())
    }

    pub fn clear_screen(&mut self) {
        for row in self.screen.iter_mut() {
            for col in row.iter_mut() {
                *col = false;
            }
        }
    }

    // Wrapping is performed in this function, no need to perform it outside
    pub fn set_pixel(&mut self, x: usize, y: usize, state: bool) {
        // Equivalent to using the mod operator, but faster
        let x = x & 63;
        let y = y & 31;
    
        let previous_state = self.screen[y][x];
        self.screen[y][x] ^= state;
        
        // If a pixel was previously set and then now unset, set VF
        if previous_state && !self.screen[y][x] {
            self.regs[0xF] = 1; 
            return;
        }

        self.regs[0xF] = 0;
    }

    pub fn execute_opcode(&mut self, opcode: Opcode, keys: [bool; 16]) -> Result<(), RuntimeError> {
        use self::RuntimeError::*;
        use opcode::Opcode::*;

        println!("{:?}", opcode);

        match opcode { 
            ClearScreen => self.clear_screen(),
            Return => {
                self.pc = match self.stack.pop() {
                    Some(addr) => addr,
                    None => return Err(EmptyCallStack),
                };
            },

            JumpTo { addr, plus_v0 } => {
                self.pc = addr;

                if plus_v0 { 
                    self.pc += self.regs[0] as u16; 
                }

                return Ok(());
            },
            Call(addr) => {
                self.stack.push(self.pc);
                self.pc = addr;

                return Ok(());
            },

            SkipIfRegEqualConst { not_equal, reg, value } => {
                let mut should_jump = self.regs[reg as usize] == value;

                if not_equal {
                    should_jump = !should_jump; // Effectively computes self.regs[reg] != value
                }

                if should_jump {
                    self.pc += 2;
                }
            },
            SkipIfRegsEqual { not_equal, regs: (v_x, v_y) } => {
                let mut should_jump = self.regs[v_x as usize] == self.regs[v_y as usize];

                if not_equal {
                    should_jump = !should_jump;
                }

                if should_jump {
                    self.pc += 2;
                    return Ok(());
                }
            },

            SetRegToConst { add, reg, value } => {
                if add {
                    let mut value = self.regs[reg as usize] as u32 + value as u32;
                    if value > 255 {
                        value -= 255;
                    }

                    self.regs[reg as usize] = value as u8;
                } else {
                    self.regs[reg as usize] = value;
                }
            },
            SetRegToReg { regs: (v_x, v_y), mode } => {
                let v_x = v_x as usize;
                let v_y = v_y as usize;

                match mode {
                    SetRegMode::Copy => self.regs[v_x] = self.regs[v_y],
                    
                    SetRegMode::Or => self.regs[v_x] |= self.regs[v_y],
                    SetRegMode::And => self.regs[v_x] &= self.regs[v_y],
                    SetRegMode::Xor => self.regs[v_x] ^= self.regs[v_y],

                    SetRegMode::Add => {
                        let mut reg_value = self.regs[v_x] as usize + self.regs[v_y] as usize;
                        if reg_value > 255 {
                            reg_value -= 255;
                            self.regs[0xF] = 1;
                        }

                        self.regs[v_x] = reg_value as u8;
                    },
                    SetRegMode::Subtract | SetRegMode::InverseSubtract => {
                        let mut reg_value = if mode == SetRegMode::Subtract {
                            self.regs[v_x] as isize - self.regs[v_y] as isize
                        } else { // Must be InverseSubtract
                            self.regs[v_y] as isize - self.regs[v_x] as isize
                        };

                        if reg_value < 0 {
                            reg_value += 255;
                            self.regs[0xF] = 1;
                        }

                        self.regs[v_x] = reg_value as u8;
                    },
                        
                    // v_y is ignored for the shift opcodes, not sure why
                    SetRegMode::ShiftLeft => {
                        self.regs[0xF] = self.regs[v_x] & 0x1000000;

                        let mut value = (self.regs[v_x] as usize) << 1;
                        if value > 255 {
                            value -= 255;
                        }

                        self.regs[v_x] = value as u8;
                    },
                    SetRegMode::ShiftRight => {
                        self.regs[0xF] = self.regs[v_x] & 0x1;
                        self.regs[v_x] >>= 1;
                    }
                }
            },

            SetAddressReg(addr) => self.addressReg = addr,
            SetRegToRandom { reg, mask } => {
                let rand: u8 = thread_rng().gen();
                self.regs[reg as usize] = rand & mask;
            },

            DrawSprite { regs: (v_x, v_y), rows } => {
                let x = self.regs[v_x as usize] as usize;
                let y = self.regs[v_y as usize] as usize;

                for row in 0..rows {
                    let sprite_slice = self.memory[(self.addressReg + row as u16) as usize];
                    
                    for col in 0..8 {
                        let bit = (sprite_slice & (128 >> col)) > 0;
                        self.set_pixel(x + col as usize, y + row as usize, bit);        
                    }
                }
            },

            SetRegToDelayTimer(reg) => self.regs[reg as usize] = self.delay_timer as u8,

            SetDelayTimerToReg(reg) => self.delay_timer = self.regs[reg as usize] as u16,
            SetSoundTimerToReg(reg) => self.sound_timer = self.regs[reg as usize] as u16,

            AddRegToAddressReg(reg) => self.addressReg += self.regs[reg as usize] as u16,
            SetAddressRegToCharInReg(reg) => {
                let ch = self.regs[reg as usize];
                self.addressReg = FONT_START + ch as u16 * 5;
            },

            WaitForKeyInReg(reg) => {

            },
            SkipIfKeyInRegPressed { not_pressed, reg } => {
                let mut should_jump = keys[self.regs[reg as usize] as usize];     

                if not_pressed {
                    should_jump = !should_jump;
                }

                if should_jump {
                    self.pc += 2;
                }
            },

            // See http://en.wikipedia.org/wiki/Binary-coded_decimal
            // n mod 10 => Gets the ones digit out of a number
            RegToBCD(reg) => {
                let number = self.regs[reg as usize];

                let hundreds_digit = number / 100;
                let tens_digit = (number / 10) % 10; // Dividing by ten slides the tens digit into the ones digit
                let ones_digit = number % 10;

                self.memory[(self.addressReg) as usize] = hundreds_digit;
                self.memory[(self.addressReg + 1) as usize] = tens_digit;
                self.memory[(self.addressReg + 2) as usize] = ones_digit;
            },

            DumpRegsToAddr(reg) => {
                for cur_reg in 0..(reg + 1) {
                    self.memory[(self.addressReg + cur_reg as u16)  as usize] = self.regs[cur_reg as usize];  
                }
            },
            LoadRegsFromAddr(reg) => {
                for cur_reg in 0..(reg + 1) {
                    self.regs[cur_reg as usize] = self.memory[(self.addressReg + cur_reg as u16) as usize];
                }
            }
        }

        Ok(())
    }

    fn decrement_timer(timer: &mut u16) {
        if *timer != 0 {
            *timer -= 1;
        }
    }
}

impl fmt::Debug for Chip8 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(fmt, "Program Counter: 0x{:X}", self.pc));
        try!(writeln!(fmt, "Address Register: 0x{:X}", self.addressReg));
        try!(writeln!(fmt, "Stack: {:?}", self.stack));
        try!(writeln!(fmt, "Delay Timer: {}", self.delay_timer));
        try!(writeln!(fmt, "Sound Timer: {}", self.sound_timer));

        write!(fmt, "Register Contents: {:?}", self.regs)
    }
}
