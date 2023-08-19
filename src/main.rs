#![feature(unchecked_math)]
#![allow(dead_code, unused)]
mod mode;
mod register;
mod instruction;

use crate::mode::Mode;
use crate::register::Register;
use crate::instruction::Instruction;

use std::fmt::Display;

use bitvec::prelude::*;

pub fn disassemble(input: &BitSlice<u8, Msb0>, signed_output: bool) -> String {
    let mut strs: Vec<String> = vec!["bits 16".to_string()];
    let mut bit_ptr = 0;
    while bit_ptr < input.len() {
        let end = if input[bit_ptr..].len() >= 48 {
            48
        } else if input[bit_ptr..].len() >= 40 {
            40
        } else if input[bit_ptr..].len() >= 32 {
            32
        } else if input[bit_ptr..].len() >= 24 {
            24
        } else {
            16
        };
        let current = &input[bit_ptr..bit_ptr + end];
        let instruction = Instruction::try_from(current).unwrap();

        let asm = instruction.to_asm();
        println!("{}", asm);
        strs.push(asm);

        bit_ptr += instruction.bytes() as usize * 8;
    }
    strs.join("\n")
}

fn main() {
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

    fn compare(actual: &str, listing: &str, expected_bin_path: &str) {
        let actual_asm_path = format!("tmp/{}_actual.asm", listing);
        let actual_bin_path = format!("tmp/{}_actual", listing);
        std::fs::write(&actual_asm_path, &actual);
        std::process::Command::new("nasm")
            .arg(&actual_asm_path)
            .output()
            .unwrap();
        let actual_contents = std::fs::read(actual_bin_path).unwrap();
        let expected_contents = std::fs::read(expected_bin_path).unwrap();
        let bs1 = actual_contents[1].view_bits::<Msb0>();
        let bs2 = expected_contents[1].view_bits::<Msb0>();
        assert_eq!(actual_contents, expected_contents, "actual: {}, expected: {}", bs1, bs2);
    }

    #[test]
    fn correctly_handles_single_register_mov() {
        // Arrange
        let input = std::fs::read("perfaware/part1/listing_0037_single_register_mov").unwrap();
        let bits = input.view_bits::<Msb0>();
        // Act
        let actual = disassemble(bits, false);
        // Assert
        compare(
            &actual,
            "0037",
            "perfaware/part1/listing_0037_single_register_mov",
        )
    }

    #[test]
    fn correctly_handles_many_register_mov() {
        // Arrange
        let input = std::fs::read("perfaware/part1/listing_0038_many_register_mov").unwrap();
        let bits = input.view_bits::<Msb0>();
        // Act
        let actual = disassemble(bits, false);
        // Assert
        compare(
            &actual,
            "0038",
            "perfaware/part1/listing_0038_many_register_mov",
        )
    }

    #[test]
    fn correctly_handles_more_movs() {
        // Arrange
        let input = std::fs::read("perfaware/part1/listing_0039_more_movs").unwrap();
        let bits = input.view_bits::<Msb0>();
        // Act
        let actual = disassemble(bits, false);
        // Assert
        compare(&actual, "0039", "perfaware/part1/listing_0039_more_movs")
    }

    // #[test]
    // fn correctly_handles_more_movs_challenge() {
    //     // Arrange
    //     let binary_file = "perfaware/part1/listing_0040_challenge_movs";
    //     let input = std::fs::read(binary_file).unwrap();
    //     let bits = input.view_bits::<Msb0>();
    //     // Act
    //     let actual = disassemble(bits, false);
    //     // Assert
    //     compare(
    //         &actual,
    //         "0040",
    //         "perfaware/part1/listing_0040_challenge_movs",
    //     )
    // }

}
