#![feature(unchecked_math)]
#![allow(dead_code, unused)]

use std::fmt::Display;

use bitvec::prelude::*;

#[derive(Copy, Clone, Debug)]
enum Register {
    AL,
    AH,
    AX,
    BL,
    BH,
    BX,
    CL,
    CH,
    CX,
    DL,
    DH,
    DX,
    SP,
    BP,
    SI,
    DI,
}

impl Register {
    pub fn from_bits(bits: &[bool; 3], wide: bool) -> Self {
        match (bits[0], bits[1], bits[2], wide) {
            (false, false, false, false) => Self::AL,
            (false, false, false, true) => Self::AX,
            (true, false, false, false) => Self::AH,

            (true, false, false, true) => Self::SP,

            (false, true, true, false) => Self::BL,
            (false, true, true, true) => Self::BX,
            (true, true, true, false) => Self::BH,

            (true, true, true, true) => Self::DI,

            (false, false, true, false) => Self::CL,
            (false, false, true, true) => Self::CX,
            (true, false, true, false) => Self::CH,

            (true, false, true, true) => Self::BP,

            (false, true, false, false) => Self::DL,
            (false, true, false, true) => Self::DX,
            (true, true, false, false) => Self::DH,

            (true, true, false, true) => Self::SI,
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let debug = format!("{self:?}");
        write!(f, "{}", debug.to_lowercase())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Mode {
    Memory,
    Bit8,
    Bit16,
    Register,
}

#[derive(Debug)]
struct ParseModeError {
    pub msg: &'static str,
}

impl ParseModeError {
    pub fn new(msg: &'static str) -> Self {
        Self { msg }
    }
}

impl<'a> From<&'a [bool; 2]> for Mode {
    fn from(bits: &'a [bool; 2]) -> Self {
        match *bits {
            [true, true] => Mode::Register,
            [true, false] => Mode::Bit16,
            [false, true] => Mode::Bit8,
            [false, false] => Mode::Memory,
        }
    }
}

#[derive(Debug)]
enum Instruction_ {
    RegisterMemoryMov {
        d: bool,
        wide: bool,
        r#mod: Mode,
        reg: Register,
        rm: [bool; 3],
        disp: Option<u16>,
        bytes_used: u8,
    },
    ImmediateRegisterMov {
        wide: bool,
        reg: Register,
        data: u16,
        bytes_used: u8,
    },
}

impl Instruction_ {
    pub fn bytes(&self) -> u8 {
        match self {
            Instruction_::RegisterMemoryMov { bytes_used, .. } => *bytes_used,
            Instruction_::ImmediateRegisterMov { bytes_used, .. } => *bytes_used,
        }
    }

    pub fn opcode_name(&self) -> &str {
        match self {
            Instruction_::RegisterMemoryMov { .. } => "mov",
            Instruction_::ImmediateRegisterMov { .. } => "mov",
            _ => unimplemented!("{:?}", self),
        }
    }

    pub fn to_asm(&self) -> String {
        match self {
            Instruction_::RegisterMemoryMov {
                d,
                wide,
                r#mod,
                reg,
                rm,
                disp,
                bytes_used,
            } => {
                let rm_reg = if *r#mod == Mode::Register {
                    Register::from_bits(rm, *wide).to_string()
                } else {
                    let effective_address = match rm {
                        [false, false, false] => "bx + si",
                        [false, false, true] => "bx + di",
                        [false, true, false] => "bp + si",
                        [false, true, true] => "bp + di",
                        [true, false, false] => "si",
                        [true, false, true] => "di",
                        [true, true, false] => {
                            if *r#mod == Mode::Register {
                                todo!()
                            } else {
                                "bp"
                            }
                        }
                        [true, true, true] => "bx",
                    };

                    let disp_str = if *r#mod == Mode::Memory {
                        String::from("")
                    } else {
                        format!(" + {}", disp.unwrap())
                    };

                    format!("[{}{}]", effective_address, "")
                };
                let (src, dest) = if *d {
                    (rm_reg, reg.to_string())
                } else {
                    (reg.to_string(), rm_reg)
                };
                format!("{} {}, {}", self.opcode_name(), dest, src)
            }

            Instruction_::ImmediateRegisterMov { reg, data, .. } => {
                format!("{} {}, {}", self.opcode_name(), reg, data)
            }
        }
    }

    fn try_parse_register_memory_mov(
        bits: &BitSlice<u8, Msb0>,
    ) -> Result<Self, ParseInstructionError> {
        if bits.len() < 16 {
            return Err(ParseInstructionError::new(
                "Incoming bits has less than 16 bits!",
            ));
        };
        let d = bits[6];
        let wide = bits[7];
        let r#mod = Mode::from(&[bits[8], bits[9]]);
        let reg = Register::from_bits(&[bits[10], bits[11], bits[12]], wide);
        let rm = [bits[13], bits[14], bits[15]];
        let mut disp = None;
        let mut bytes_used = 2;

        match r#mod {
            Mode::Bit8 => {
                if bits.len() < 24 {
                    return Err(ParseInstructionError::new(
                        "Incoming instruction has an 8 bit displacement, but the `disp_lo` byte wasn't provided. Requires at least 24 bits.",
                    ));
                }
                disp = Some(bits[16..24].load::<u8>() as u16);
                bytes_used = 3;
            }
            Mode::Bit16 => {
                if bits.len() < 32 {
                    return Err(ParseInstructionError::new(
                        "Incoming instruction has an 16 bit displacement, but the `disp_hi` byte wasn't provided. Requires at least 32 bits.",
                    ));
                }
                disp = Some(bits[16..].load::<u16>());
                bytes_used = 4;
            }
            _ => {}
        }

        Ok(Self::RegisterMemoryMov {
            d,
            wide,
            r#mod,
            reg,
            rm,
            disp,
            bytes_used,
        })
    }

    fn try_parse_immediate_register_mov(
        bits: &BitSlice<u8, Msb0>,
    ) -> Result<Instruction_, ParseInstructionError> {
        if bits.len() < 16 {
            return Err(ParseInstructionError::new(
                "Incoming bits has less than 16 bits!",
            ));
        };
        let wide = bits[4];
        let reg = Register::from_bits(&[bits[5], bits[6], bits[7]], wide);
        let mut bytes_used = 2;
        let data = if wide {
            if bits.len() < 24 {
                return Err(ParseInstructionError::new(
                    "Expected wide data. Received less than 24 bits.",
                ));
            };
            bytes_used = 3;
            bits[8..24].load::<u16>()
        } else {
            bits[8..16].load::<u8>() as u16
        };

        Ok(Self::ImmediateRegisterMov {
            wide,
            reg,
            data,
            bytes_used,
        })
    }
}

#[derive(Debug)]
struct ParseInstructionError {
    pub msg: &'static str,
}

impl ParseInstructionError {
    pub fn new(msg: &'static str) -> Self {
        Self { msg }
    }
}

impl<'a> TryFrom<&'a BitSlice<u8, Msb0>> for Instruction_ {
    type Error = ParseInstructionError;

    fn try_from(bits: &BitSlice<u8, Msb0>) -> Result<Self, Self::Error> {
        match (bits[0], bits[1], bits[2], bits[3], bits[4], bits[5]) {
            (true, false, false, false, true, false) => Self::try_parse_register_memory_mov(bits),
            (true, false, true, true, _, _) => Self::try_parse_immediate_register_mov(bits),
            _ => unimplemented!("{:?}", bits),
        }
    }
}

pub fn disassemble(input: &BitSlice<u8, Msb0>) -> String {
    let mut strs: Vec<String> = Vec::new();
    let mut i = 0;
    while i < input.len() {
        let end = if input[i..].len() >= 32 {
            32
        } else if input[i..].len() >= 24 {
            24
        } else {
            16
        };
        let current = &input[i..i + end];
        let instruction = Instruction_::try_from(current).unwrap();

        strs.push(instruction.to_asm().to_string());

        i += instruction.bytes() as usize * 8;
    }
    strs.join("\n")
}

fn main() {
    let input = std::fs::read("perfaware/part1/listing_0037_single_register_mov").unwrap();
    let bits = input.view_bits::<Msb0>();
    let output = disassemble(bits);
    println!("{output}");
    println!("----------");
    let input = std::fs::read("perfaware/part1/listing_0038_many_register_mov").unwrap();
    let bits = input.view_bits::<Msb0>();
    let output = disassemble(bits);
    println!("{output}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skip_preamble(s: &str) -> String {
        let contents = std::fs::read_to_string(s).unwrap();

        contents
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with("bits") && !l.starts_with(";"))
            .collect::<Vec<&str>>()
            .join("\n")
    }

    #[test]
    fn correctly_handles_single_register_mov() {
        // Arrange
        let input = std::fs::read("perfaware/part1/listing_0037_single_register_mov").unwrap();
        let bits = input.view_bits::<Msb0>();
        let expected = skip_preamble("perfaware/part1/listing_0037_single_register_mov.asm");
        // Act
        let actual = disassemble(bits);
        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn correctly_handles_many_register_mov() {
        // Arrange
        let input = std::fs::read("perfaware/part1/listing_0038_many_register_mov").unwrap();
        let bits = input.view_bits::<Msb0>();
        let expected = skip_preamble("perfaware/part1/listing_0038_many_register_mov.asm");
        // Act
        let actual = disassemble(bits);
        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn correctly_handles_more_movs() {
        // Arrange
        let input = std::fs::read("perfaware/part1/listing_0039_more_movs").unwrap();
        let bits = input.view_bits::<Msb0>();
        let expected = skip_preamble("perfaware/part1/listing_0039_more_movs.asm");
        // Act
        let actual = disassemble(bits);
        // Assert
        assert_eq!(actual, expected);
    }
}
