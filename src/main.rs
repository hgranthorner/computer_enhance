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
    pub fn from_bv<T: BitOrder>(bv: &BitSlice<u8, T>, wide: bool) -> Self {
        let f = bv[0];
        let s = bv[1];
        let t = bv[2];
        match (f, s, t, wide) {
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

            _ => panic!("Invalid BitVec {:?}!", bv),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let debug = format!("{self:?}");
        write!(f, "{}", debug.to_lowercase())
    }
}

pub fn disassemble<T: BitOrder>(mut input: &BitSlice<u8, T>) -> String {
    let mut strs: Vec<String> = Vec::new();
    for i in (0..input.len()).step_by(16) {
        let current = &input[i..i + 16];
        let op = &current[0..6];
        let mov_op = &[0b100010 as u8].view_bits::<Msb0>()[2..];
        let opcode = if op == mov_op {
            "mov".to_string()
        } else {
            "not mov".to_string()
        };

        // Destination is in reg field?
        let d = current.get(6).unwrap();
        // Instruction on word sized data?
        let w = current.get(7).unwrap();
        // 11 = register to register
        // 00 = memory to memory
        // Otherwise, register to memory
        let r#mod = &current[8..10];
        assert_eq!(r#mod, &[0b11 as u8].view_bits::<Msb0>()[6..8]);
        let reg = Register::from_bv(&current[10..13], *w);
        let rm = Register::from_bv(&current[13..16], *w);
        let (src, dest) = if *d { (rm, reg) } else { (reg, rm) };

        strs.push(format!("{} {}, {}", opcode, dest, src));
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
        for line in lines 
        {
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
        assert_eq!(actual, expected.to_string());
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
        assert_eq!(actual, expected.to_string());
    }
}
