#![allow(dead_code, unused)]

use std::fmt::Display;

use bit_vec::BitVec;

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
    pub fn from_bv(bv: &BitVec, wide: bool) -> Self {
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

fn slice(bv: &BitVec, start: usize, len: usize) -> BitVec {
    let mut bv = bv.clone();
    let mut bv = bv.split_off(start);
    bv.split_off(len);
    bv
}

fn disassemble(mut input: BitVec) -> String {
    let mut strs: Vec<String> = Vec::new();
    while input.len() >= 16 {
        let current = input.clone();
        input = input.split_off(16);
        let op = slice(&current, 0, 6);
        let mov_op = slice(&BitVec::from_bytes(&[0b100010]), 2, 6);
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
        let r#mod = slice(&current, 8, 2);
        assert_eq!(r#mod, slice(&BitVec::from_bytes(&[0b11]), 6, 2));
        let reg = Register::from_bv(&slice(&current, 10, 3), w);
        let rm = Register::from_bv(&slice(&current, 13, 3), w);
        let (src, dest) = if d { (rm, reg) } else { (reg, rm) };

        strs.push(format!("{} {}, {}", opcode, dest, src));
    }
    strs.join("\n")
}

// fn to_bits(input: &[u8]) -> Vec<bool> {
//     input.into_iter()
//         .flat_map(|b| vec![b >> 3, b >> 2])
//         .collect()
// }

fn main() {
    let input = std::fs::read("perfaware/part1/listing_0037_single_register_mov").unwrap();
    let bits = BitVec::from_bytes(&input);
    let output = disassemble(bits);
    println!("{output}");
    println!("----------");
    let input = std::fs::read("perfaware/part1/listing_0038_many_register_mov").unwrap();
    let bits = BitVec::from_bytes(&input);
    let output = disassemble(bits);
    println!("{output}");
}
