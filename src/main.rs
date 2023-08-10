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

struct Instruction {
    pub opcode: [bool; 6],
    pub d: bool,
    pub w: bool,
    pub r#mod: [bool; 2],
    pub reg: [bool; 3],
    pub rm: [bool; 3],
}

impl Instruction {
    pub fn opcode_name(&self) -> &str {
        match self.opcode {
            // 100010
            [true, false, false, false, true, false] => "mov",
            _ => unimplemented!("opcode not supported: {:?}", self.opcode),
        }
    }

    pub fn src_reg(&self) -> Register {
        if self.d {
            Register::from_bits(&self.rm, self.w)
        } else {
            Register::from_bits(&self.reg, self.w)
        }
    }

    pub fn dest_reg(&self) -> Register {
        if self.d {
            Register::from_bits(&self.reg, self.w)
        } else {
            Register::from_bits(&self.rm, self.w)
        }
    }

    pub fn to_asm(&self) -> String {
        format!(
            "{} {}, {}",
            self.opcode_name(),
            self.dest_reg(),
            self.src_reg()
        )
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

    fn try_from(bits: &'a BitSlice<u8, Msb0>) -> Result<Self, Self::Error> {
        if bits.len() < 16 {
            return Err(ParseInstructionError::new(
                "Incoming bits has less than 16 bits!",
            ));
        }
        Ok(Self {
            opcode: [bits[0], bits[1], bits[2], bits[3], bits[4], bits[5]],
            d: bits[6],
            w: bits[7],
            r#mod: [bits[8], bits[9]],
            reg: [bits[10], bits[11], bits[12]],
            rm: [bits[13], bits[14], bits[15]],
        })
    }
}

pub fn disassemble(input: &BitSlice<u8, Msb0>) -> String {
    let mut strs: Vec<String> = Vec::new();
    for i in (0..input.len()).step_by(16) {
        let current = &input[i..i + 16];
        let instruction = Instruction::try_from(current).unwrap();

        strs.push(format!("{}", instruction.to_asm()));
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
