
use opcode::Opcode;

const PROGRAM_START: usize = 0x200;

const MEMORY_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;

pub enum RuntimeError {
    EmptyCallStack,
    InvalidRegister(u8),
    AddressOutOfBounds(u16),
}

pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    registers: [u8; REGISTER_COUNT], // registers V0 - V15
    addressReg: u16, // register I

    pc: usize,
    // Stores the program counters of sub routine calls, used to return after a sub routine ends
    stack: Vec<u8>,

    screen: [[bool; 64]; 32],
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            memory: [0; 4096],
            registers: [0; 16],
            addressReg: 0,
            
            pc: PROGRAM_START,
            stack: Vec::new(),

            screen: [[false; 64]; 32],
        }
    }

    pub fn cycle(&mut self) {
        let opcode_bytes = (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16);
        let opcode = Opcode::from_u16(opcode_bytes);

        println!("{:?}", opcode);

        self.pc += 2;
    }
}
