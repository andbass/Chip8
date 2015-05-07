
use std::fmt;

use opcode::{Opcode, OpcodeError, SetRegMode};

const PROGRAM_START: u16 = 0x200;

const MEMORY_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;

pub enum RuntimeError {
    EmptyCallStack,
    InvalidRegister(u8),
    AddressOutOfBounds(u16),
    OpcodeErr(OpcodeError),
}

pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    regs: [u8; REGISTER_COUNT], // registers V0 - V15
    addressReg: u16, // register I

    pc: u16,
    // Stores the program counters of sub routine calls, used to return after a sub routine ends
    stack: Vec<u16>,

    screen: [[bool; 64]; 32],
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            memory: [0; 4096],
            regs: [0; 16],
            addressReg: 0,
            
            pc: PROGRAM_START,
            stack: Vec::new(),

            screen: [[false; 64]; 32],
        }
    }

    pub fn cycle(&mut self) -> Result<(), RuntimeError> {
        use self::RuntimeError::*;
        use opcode::Opcode::*;

        let pc_index = self.pc as usize;
        let opcode_bytes = (self.memory[pc_index] as u16) << 8 | (self.memory[pc_index + 1] as u16);

        let opcode = match Opcode::from_u16(opcode_bytes) {
            Ok(opcode) => opcode,
            Err(err) => return Err(OpcodeErr(err)),
        };

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
            },
            Call(addr) => {
                self.stack.push(self.pc);
                self.pc = addr;
            },

            SkipIfRegEqualConst { not_equal, reg, value } => {
                let mut should_jump = self.regs[reg as usize] == value;

                if not_equal {
                    should_jump = !should_jump; // Effectively computes self.regs[reg] != value
                }

                if should_jump {
                    self.pc += 4;
                    return Ok(());
                }
            },
            SkipIfRegsEqual { not_equal, regs: (v_x, v_y) } => {
                let mut should_jump = self.regs[v_x as usize] == self.regs[v_y as usize];

                if not_equal {
                    should_jump = !should_jump;
                }

                if should_jump {
                    self.pc += 4;
                    return Ok(());
                }
            },

            SetRegToConst { add, reg, value } => {
                if add {
                    self.regs[reg as usize] += value;
                } else {
                    self.regs[reg as usize] = value;
                }
            },
            SetRegToReg { regs: (v_x, v_y), mode } => {
                let v_x = v_x as usize;
                let v_y = v_y as usize;

                match mode {
                    SetRegMode::Copy => {
                        self.regs[v_x] = self.regs[v_y];
                    },
                    SetRegMode::Or => {
                        self.regs[v_x] |= self.regs[v_y];
                    },
                    SetRegMode::And => {
                        self.regs[v_x] &= self.regs[v_y];
                    },
                    SetRegMode::Xor => {
                        self.regs[v_x] ^= self.regs[v_y];
                    },

                    SetRegMode::Add => {
                        let reg_value = self.regs[v_x] as usize + self.regs[v_y] as usize;
                        if reg_value > 255 {
                            reg_value -= 255;
                            self.regs[0xF] = 1;
                        }

                        self.regs[v_x] = reg_value as u8;
                    },
                    SetRegMode::Subtract | SetRegMode::InverseSubtract => {
                        let reg_value = if let SetRegMode::Subtract = mode {
                            self.regs[v_x] as isize - self.regs[v_y] as isize;
                        else { // Must be InverseSubtract
                            self.regs[v_y] as isize - self.regs[v_x] as isize;
                        };

                        if reg_value < 0 {
                            reg_value += 255;
                            self.regs[0xF] = 1;
                        }

                        self.regs[v_x] = reg_value as u8;
                    },
                    SetRegMode::InverseSubtract => {

                    },

                    SetRegMode::ShiftLeft => {

                    },
                    SetRegMode::ShiftRight => {

                    }
                }
            },
            _ => unreachable!(),
        }

        self.pc += 2;

        Ok(())
    }

    pub fn clear_screen(&mut self) {
        for row in self.screen.iter_mut() {
            for col in row.iter_mut() {
                *col = false;
            }
        }
    }
}

impl fmt::