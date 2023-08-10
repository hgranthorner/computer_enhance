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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mode {
    Memory,
    Displace8Bits,
    Displace16Bits,
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
            [true, false] => Mode::Displace16Bits,
            [false, true] => Mode::Displace8Bits,
            [false, false] => Mode::Memory,
        }
    }
}

#[derive(Debug)]
enum Instruction {
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

fn deserialize_effective_address(rm: &[bool; 3], r#mod: Mode, disp: Option<u16>) -> String {
    match rm {
        [false, false, false] => "bx + si".to_string(),
        [false, false, true] => "bx + di".to_string(),
        [false, true, false] => "bp + si".to_string(),
        [false, true, true] => "bp + di".to_string(),
        [true, false, false] => "si".to_string(),
        [true, false, true] => "di".to_string(),
        [true, true, false] => {
            if r#mod == Mode::Memory {
                return format!("[{}]", disp.unwrap());
            }
            "bp".to_string()
        }
        [true, true, true] => "bx".to_string(),
    }
}

fn deserialize_displacement(
    r#mod: Mode,
    disp: Option<u16>,
    wide: bool,
    signed_output: bool,
) -> String {
    if r#mod == Mode::Memory {
        return String::from("");
    }
    let val = disp.unwrap();
    if val == 0 {
        return String::from("");
    }
    if !signed_output {
        return format!(" + {}", val);
    }
    if wide {
        let signed_val = val as i16;
        let op = if signed_val < 0 { "-" } else { "+" };
        format!(" {} {}", op, signed_val.abs())
    } else {
        let signed_val = val as i8;
        let op = if signed_val < 0 { "-" } else { "+" };
        format!(" {} {}", op, signed_val.abs())
    }
}

impl Instruction {
    pub fn bytes(&self) -> u8 {
        match self {
            Instruction::RegisterMemoryMov { bytes_used, .. } => *bytes_used,
            Instruction::ImmediateRegisterMov { bytes_used, .. } => *bytes_used,
        }
    }

    pub fn opcode_name(&self) -> &str {
        match self {
            Instruction::RegisterMemoryMov { .. } => "mov",
            Instruction::ImmediateRegisterMov { .. } => "mov",
            _ => unimplemented!("{:?}", self),
        }
    }

    pub fn to_asm(&self, signed_output: bool) -> String {
        match self {
            Instruction::RegisterMemoryMov {
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
                    let effective_address = deserialize_effective_address(rm, *r#mod, *disp);
                    let disp_str = deserialize_displacement(*r#mod, *disp, *wide, signed_output);

                    if effective_address.starts_with('[') {
                        effective_address
                    } else {
                        format!("[{}{}]", effective_address, disp_str)
                    }
                };
                let (src, dest) = if *d {
                    (rm_reg, reg.to_string())
                } else {
                    (reg.to_string(), rm_reg)
                };
                format!("{} {}, {}", self.opcode_name(), dest, src)
            }

            Instruction::ImmediateRegisterMov {
                reg, data, wide, ..
            } => {
                if *wide {
                    format!("{} {}, {}", self.opcode_name(), reg, *data as i16)
                } else {
                    format!("{} {}, {}", self.opcode_name(), reg, *data as i8)
                }
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
            Mode::Displace8Bits => {
                if bits.len() < 24 {
                    return Err(ParseInstructionError::new(
                        "Incoming instruction has an 8 bit displacement, but the `disp_lo` byte wasn't provided. Requires at least 24 bits.",
                    ));
                }
                disp = Some(bits[16..24].load::<u8>() as u16);
                bytes_used = 3;
            }
            Mode::Displace16Bits => {
                if bits.len() < 32 {
                    return Err(ParseInstructionError::new(
                        "Incoming instruction has an 16 bit displacement, but the `disp_hi` byte wasn't provided. Requires at least 32 bits.",
                    ));
                }
                disp = Some(bits[16..].load::<u16>());
                bytes_used = 4;
            }
            Mode::Memory => {
                if rm == [true, true, false] {
                    if bits.len() < 32 {
                        return Err(ParseInstructionError::new(
                        "Incoming instruction has an 16 bit displacement, but the `disp_hi` byte wasn't provided. Requires at least 32 bits.",
                    ));
                    }
                    disp = Some(bits[16..].load::<u16>());
                    bytes_used = 4;
                }
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
    ) -> Result<Instruction, ParseInstructionError> {
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

impl<'a> TryFrom<&'a BitSlice<u8, Msb0>> for Instruction {
    type Error = ParseInstructionError;

    fn try_from(bits: &BitSlice<u8, Msb0>) -> Result<Self, Self::Error> {
        match (bits[0], bits[1], bits[2], bits[3], bits[4], bits[5]) {
            (true, false, false, false, true, false) => Self::try_parse_register_memory_mov(bits),
            (true, false, true, true, _, _) => Self::try_parse_immediate_register_mov(bits),
            _ => unimplemented!("{:?}", bits),
        }
    }
}

pub fn disassemble(input: &BitSlice<u8, Msb0>, signed_output: bool) -> String {
    let mut strs: Vec<String> = Vec::new();
    let mut bit_ptr = 0;
    while bit_ptr < input.len() {
        let end = if input[bit_ptr..].len() >= 32 {
            32
        } else if input[bit_ptr..].len() >= 24 {
            24
        } else {
            16
        };
        let current = &input[bit_ptr..bit_ptr + end];
        let instruction = Instruction::try_from(current).unwrap();

        strs.push(instruction.to_asm(signed_output).to_string());

        bit_ptr += instruction.bytes() as usize * 8;
    }
    strs.join("\n")
}

fn main() {
    // let input = std::fs::read("perfaware/part1/listing_0037_single_register_mov").unwrap();
    // let bits = input.view_bits::<Msb0>();
    // let output = disassemble(bits);
    // println!("{output}");
    // println!("----------");
    // let input = std::fs::read("perfaware/part1/listing_0038_many_register_mov").unwrap();
    // let bits = input.view_bits::<Msb0>();
    // let output = disassemble(bits);
    // println!("{output}");
    // println!("----------");
    let input = std::fs::read("perfaware/part1/listing_0039_more_movs").unwrap();
    let bits = input.view_bits::<Msb0>();
    let output = disassemble(bits, false);
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
        let actual = disassemble(bits, false);
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
        let actual = disassemble(bits, false);
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
        let actual = disassemble(bits, false);
        // Assert
        assert_eq!(actual, expected);
    }
}
