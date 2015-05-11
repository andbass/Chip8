
pub type OpcodeResult = Result<Opcode, OpcodeError>;

#[derive(Debug)]
pub enum OpcodeError {
    UnrecognizedOpcode(u16),
    InvalidModeForSetRegToReg(u8),
}

#[derive(Debug, PartialEq)]
pub enum SetRegMode {
    Copy = 0x0, // VX = VY

    Or = 0x1, // VX |= VY
    And = 0x2, // VX &= VY
    Xor = 0x3, // VX ^= VY

    Add = 0x4, // VX += VY
    Subtract = 0x5, // VX -= VY
    InverseSubtract = 0x7, // VX = VY - VX

    // For these, Y is ignored.  I am not sure why
    ShiftLeft = 0xE, // VX <<= 1
    ShiftRight = 0x6, // VX >>= 1
}

impl SetRegMode {
    pub fn from_u8(byte: u8) -> Option<SetRegMode> {
        use self::SetRegMode::*;

        match byte {
            0x0 => Some(Copy),
            0x1 => Some(Or),
            0x2 => Some(And),
            0x3 => Some(Xor),
            0x4 => Some(Add),
            0x5 => Some(Subtract),
            0x6 => Some(ShiftRight),
            0x7 => Some(InverseSubtract),
            0xE => Some(ShiftLeft),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum Opcode {
    /* KEY
     * NNNN => address,
     * NN => 8 bit number,
     * N => 4 bit number,
     * X and Y => regs
     */
    ClearScreen,    // 0x00E0
    Return,         // 0x00EE
    JumpTo {        // 1NNN | BNNN
        addr: u16,
        plus_v0: bool
    },    
    Call(u16),      // 2NNN

    SkipIfRegEqualConst {   // 3XNN or 4XNN
        not_equal: bool, // Check to see if reg is NOT equal to value instead

        reg: u8,
        value: u8,
    },
    SkipIfRegsEqual { // 5XY0 | 9XY0
        not_equal: bool, // Check to see if regs are NOT equal instead
        regs: (u8, u8),
    },

    SetRegToConst { // 6XNN | 7XNN
        add: bool,

        reg: u8,
        value: u8,
    },
    SetRegToReg { // 8XY[0 - E]
        regs: (u8, u8),
        mode: SetRegMode,
    }, 
     
    SetAddressReg(u16), // ANNN
    SetRegToRandom { // CXNN
        reg: u8,
        mask: u8,
    },

    DrawSprite { // DXYN, draws from sprite addr stored in I. Each sprite is 8 bits wide
        regs: (u8, u8),
        rows: u8,
    },

    SkipIfKeyInRegPressed { // EX9E | EXA1 
        not_pressed: bool,
        reg: u8,
    },
    WaitForKeyInReg(u8), // FX0A

    SetRegToDelayTimer(u8), // FX07

    SetDelayTimerToReg(u8), // FX15
    SetSoundTimerToReg(u8), // FX18

    AddRegToAddressReg(u8), // FX1E
    SetAddressRegToCharInReg(u8), // FX29, sets the address pointer to point to the text character specified in X
    RegToBCD(u8), // FX33, see http://en.wikipedia.org/wiki/Binary-coded_decimal
    
    DumpRegsToAddr(u8), // FX55
    LoadRegsFromAddr(u8), // FX65
}

impl Opcode {
    pub fn from_u16(bytes: u16) -> OpcodeResult {
        use self::Opcode::*;
        use self::OpcodeError::*;

        let msb = bytes & 0xF000;
        match msb {
            0x0000 => {
                match bytes & 0x0F00 {
                    0x000 => match bytes & 0x00F0 {
                        0xE0 => match bytes & 0x000F {
                            0x0 => Ok(Opcode::ClearScreen),
                            0xE => Ok(Opcode::Return),
                            _ => Err(UnrecognizedOpcode(bytes)),
                        },
                        _ => Err(UnrecognizedOpcode(bytes)),
                    },
                    _ => Err(UnrecognizedOpcode(bytes)),
                }
            },
            0x1000 | 0xB000 => {
                Ok(JumpTo { 
                    addr: bytes & 0x0FFF, 
                    plus_v0: msb == 0xB000 
                })
            },
            0x2000 => Ok(Call(bytes & 0x0FFF)),
            0x3000 | 0x4000 => {
                Ok(SkipIfRegEqualConst {
                    not_equal: msb == 0x4000,

                    reg: ((bytes & 0x0F00) >> 8) as u8,
                    value: (bytes & 0x00FF) as u8,
                })
            },
            0x5000 | 0x9000 => match bytes & 0x000F {
                0x0 => {
                    Ok(SkipIfRegsEqual {
                        not_equal: msb == 0x9000,
                        regs: (((bytes & 0x0F00) >> 8) as u8,
                               ((bytes & 0x00F0) >> 4) as u8),
                    })
                },
                _ => Err(UnrecognizedOpcode(bytes)),
            },
            0x6000 | 0x7000 => {
                Ok(SetRegToConst {
                    add: msb == 0x7000,

                    reg: ((bytes & 0x0F00) >> 8) as u8,
                    value: (bytes & 0x00FF) as u8,
                })
            },
            0x8000 => {
                let mode_byte = (bytes & 0x000F) as u8;
                let mode = match SetRegMode::from_u8(mode_byte) {
                    Some(mode) => mode,
                    None => return Err(InvalidModeForSetRegToReg(mode_byte)),
                };

                Ok(SetRegToReg {
                    regs: (((bytes & 0x0F00) >> 8) as u8,
                           ((bytes & 0x00F0) >> 4) as u8),
                    mode: mode,
                })
            },
            0xA000 => Ok(SetAddressReg(bytes & 0x0FFF)),
            0xC000 => {
                Ok(SetRegToRandom {
                    reg: ((bytes & 0x0F00) >> 8) as u8,
                    mask: (bytes & 0x0FF) as u8,                    
                })
            },
            0xD000 => {
                Ok(DrawSprite {
                    regs: (((bytes & 0x0F00) >> 8) as u8,
                           ((bytes & 0x00F0) >> 4) as u8),
                    rows: (bytes & 0x000F) as u8,
                })
            },
            0xE000 => match bytes & 0x00FF {
                0x9E | 0xA1 => {
                    Ok(SkipIfKeyInRegPressed {
                        not_pressed: (bytes & 0x00FF) == 0xA1,
                        reg: ((bytes & 0x0F00) >> 8) as u8,
                    })
                },
                _ => Err(UnrecognizedOpcode(bytes)),
            },
            0xF000 => {
                let reg = ((bytes & 0x0F00) >> 8) as u8;

                match bytes & 0x00FF {
                    0x07 => Ok(SetRegToDelayTimer(reg)),
                    0x0A => Ok(WaitForKeyInReg(reg)),
                    0x15 => Ok(SetDelayTimerToReg(reg)),
                    0x18 => Ok(SetSoundTimerToReg(reg)),
                    0x1E => Ok(AddRegToAddressReg(reg)),
                    0x29 => Ok(SetAddressRegToCharInReg(reg)),
                    0x33 => Ok(RegToBCD(reg)),
                    0x55 => Ok(DumpRegsToAddr(reg)),
                    0x65 => Ok(LoadRegsFromAddr(reg)),
                    _ => Err(UnrecognizedOpcode(bytes)),
                }
            },
            _ => Err(UnrecognizedOpcode(bytes)),
        }
    }
}
