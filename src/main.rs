#![feature(unchecked_math)]
#![allow(dead_code, unused)]

use std::fmt::Display;

use bitvec::prelude::*;

#[derive(Debug)]
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

#[derive(Debug)]
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
struct Instruction {
    pub opcode: [bool; 6],
    pub d: bool,
    pub w: bool,
    pub r#mod: Mode,
    pub reg: [bool; 3],
    pub rm: [bool; 3],
    pub disp: Option<u16>,
    pub bytes_used: u8,
}

impl Instruction {
    pub fn opcode_name(&self) -> &str {
        match self.opcode {
            // 100010
            [true, false, false, false, true, false] => "mov",
            _ => unimplemented!("opcode not supported: {:?}", self.opcode),
        }
    }

    pub fn src(&self) -> Register {
        if self.d {
            Register::from_bits(&self.rm, self.w)
        } else {
            Register::from_bits(&self.reg, self.w)
        }
    }

    pub fn dest(&self) -> Register {
        if self.d {
            Register::from_bits(&self.reg, self.w)
        } else {
            Register::from_bits(&self.rm, self.w)
        }
    }

    pub fn to_asm(&self) -> String {
        format!("{} {}, {}", self.opcode_name(), self.dest(), self.src())
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

impl<'a> TryFrom<&'a BitSlice<u8, Msb0>> for Instruction {
    type Error = ParseInstructionError;

    fn try_from(bits: &BitSlice<u8, Msb0>) -> Result<Self, Self::Error> {
        if bits.len() < 16 {
            return Err(ParseInstructionError::new(
                "Incoming bits has less than 16 bits!",
            ));
        }
        let mut instr = Self {
            opcode: [bits[0], bits[1], bits[2], bits[3], bits[4], bits[5]],
            d: bits[6],
            w: bits[7],
            r#mod: Mode::from(&[bits[8], bits[9]]),
            reg: [bits[10], bits[11], bits[12]],
            rm: [bits[13], bits[14], bits[15]],
            disp: None,
            bytes_used: 2,
        };

        match instr.r#mod {
            Mode::Memory => {
                todo!()
            }
            Mode::Bit8 => {
                if bits.len() < 24 {
                    return Err(ParseInstructionError::new(
                        "Incoming instruction has an 8 bit displacement, but the `disp_lo` byte wasn't provided. Requires at least 24 bits.",
                    ));
                }
                instr.disp = Some(bits[16..].load::<u8>() as u16);
                instr.bytes_used += 1;
            }
            Mode::Bit16 => {
                if bits.len() < 32 {
                    return Err(ParseInstructionError::new(
                        "Incoming instruction has an 16 bit displacement, but the `disp_hi` byte wasn't provided. Requires at least 32 bits.",
                    ));
                }
                instr.disp = Some(bits[16..].load::<u16>());
                instr.bytes_used += 2;
            }
            Mode::Register => {}
        }

        Ok(instr)
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
        let instruction = Instruction::try_from(current).unwrap();

        strs.push(instruction.to_asm().to_string());

        i += instruction.bytes_used as usize * 8;
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

        let mut lines = contents
            .lines()
            .skip_while(|l| !l.starts_with("bits"))
            .skip(2);
        let mut result = lines.next().unwrap().to_string();
        for line in lines {
            result.push_str("\n");
            result.push_str(line);
        }

        result
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
}
