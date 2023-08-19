use bitvec::{slice::BitSlice, prelude::*};

use crate::{mode::Mode, register::Register};

#[derive(Debug)]
pub enum Instruction {
    RegisterMemoryMov {
        // true  = destination in reg
        // false = destination in rm
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
    ImmediateRegisterMemoryMov {
        wide: bool,
        r#mod: Mode,
        rm: [bool; 3],
        disp: Option<u16>,
        data: u16,
        bytes_used: u8,
    },
    MemoryAccumMov {
        to_memory: bool,
        wide: bool,
        addr: u16,
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
            println!("Edge case!");
            if r#mod == Mode::Memory {
                return format!("[{}]", disp.unwrap());
            }
            "bp".to_string()
        }
        [true, true, true] => "bx".to_string(),
    }
}

fn deserialize_displacement(r#mod: Mode, disp: Option<u16>, wide: bool) -> String {
    if r#mod == Mode::Memory {
        return String::from("");
    }
    let val = disp.unwrap();
    if val == 0 {
        return String::from("");
    }
    if wide {
        let signed_val = val as i32;
        let op = if signed_val < 0 { "-" } else { "+" };
        format!(" {} {}", op, signed_val.abs())
    } else {
        let signed_val = val as i16;
        let op = if signed_val < 0 { "-" } else { "+" };
        format!(" {} {}", op, signed_val.abs())
    }
}

impl Instruction {
    pub fn bytes(&self) -> u8 {
        match self {
            Instruction::RegisterMemoryMov { bytes_used, .. } => *bytes_used,
            Instruction::ImmediateRegisterMov { bytes_used, .. } => *bytes_used,
            Instruction::ImmediateRegisterMemoryMov { bytes_used, .. } => *bytes_used,
            Instruction::MemoryAccumMov { bytes_used, .. } => *bytes_used,
        }
    }

    pub fn opcode_name(&self) -> &str {
        match self {
            Instruction::RegisterMemoryMov { .. } => "mov",
            Instruction::ImmediateRegisterMov { .. } => "mov",
            Instruction::ImmediateRegisterMemoryMov { .. } => "mov",
            Instruction::MemoryAccumMov { .. } => "mov",
        }
    }

    pub fn to_asm(&self) -> String {
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
                    let disp_str = deserialize_displacement(*r#mod, *disp, *wide);

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
            Instruction::ImmediateRegisterMemoryMov {
                wide,
                r#mod,
                rm,
                disp,
                data,
                ..
            } => {
                let dest = {
                    let effective_address = deserialize_effective_address(rm, *r#mod, *disp);
                    let disp_str = deserialize_displacement(*r#mod, *disp, *wide);

                    if effective_address.starts_with('[') {
                        effective_address
                    } else {
                        format!("[{}{}]", effective_address, disp_str)
                    }
                };
                let src = if *wide {
                    format!("word {}", data)
                } else {
                    format!("byte {}", data)
                };

                format!("{} {}, {}", self.opcode_name(), dest, src)
            }

            Instruction::MemoryAccumMov {
                to_memory,
                wide,
                addr,
                bytes_used,
            } => {
                    let (src, dest) = if *to_memory {
                        ("ax".to_string(), format!("[{}]", addr))
                    } else {
                        (format!("[{}]", addr), "ax".to_string())
                    };

                    format!("{} {}, {}", self.opcode_name(), dest, src)
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
        println!("{}", bits);
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
                disp = Some(bits[16..32].load::<u16>());
                bytes_used = 4;
            }
            Mode::Memory => {
                if rm == [true, true, false] {
                    if bits.len() < 32 {
                        return Err(ParseInstructionError::new(
                        "Incoming instruction has an 16 bit displacement, but the `disp_hi` byte wasn't provided. Requires at least 32 bits.",
                    ));
                    }
                    disp = Some(bits[16..32].load::<u16>());
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

    fn try_parse_immediate_register_memory_mov(
        bits: &BitSlice<u8, Msb0>,
    ) -> Result<Instruction, ParseInstructionError> {
        let wide = bits[7];
        let r#mod = Mode::from(&[bits[8], bits[9]]);
        let rm = [bits[13], bits[14], bits[15]];
        let (disp, data, bytes_used) = match r#mod {
            Mode::Displace8Bits => (
                Some(bits[16..24].load::<u8>() as u16),
                if wide {
                    bits[24..40].load::<u16>()
                } else {
                    bits[24..32].load::<u8>() as u16
                },
                if wide { 5 } else { 4 },
            ),
            Mode::Displace16Bits => (
                Some(bits[16..32].load::<u16>()),
                if wide {
                    bits[32..48].load::<u16>()
                } else {
                    bits[32..40].load::<u8>() as u16
                },
                if wide { 6 } else { 5 },
            ),
            Mode::Memory => {
                if rm == [true, true, false] {
                    (
                        Some(bits[16..32].load::<u16>()),
                        if wide {
                            bits[32..48].load::<u16>()
                        } else {
                            bits[32..40].load::<u8>() as u16
                        },
                        if wide { 6 } else { 5 },
                    )
                } else {
                    (
                        None,
                        if wide {
                            bits[16..32].load::<u16>()
                        } else {
                            bits[16..24].load::<u8>() as u16
                        },
                        if wide { 4 } else { 3 },
                    )
                }
            }

            Mode::Register => (
                None,
                if wide {
                    bits[16..32].load::<u16>()
                } else {
                    bits[16..24].load::<u8>() as u16
                },
                if wide { 4 } else { 3 },
            ),
        };
        Ok(Self::ImmediateRegisterMemoryMov {
            wide,
            r#mod,
            rm,
            disp,
            data,
            bytes_used,
        })
    }

    fn try_parse_memory_accum_mov(
        bits: &BitSlice<u8, Msb0>,
    ) -> Result<Instruction, ParseInstructionError> {
        let to_memory = bits[6];
        let wide = bits[7];
        let addr = if wide {
            bits[8..24].load::<u16>()
        } else {
            bits[8..16].load::<u8>() as u16
        };
        let bytes_used = if wide { 3 } else { 2 };
        Ok(Self::MemoryAccumMov {
            to_memory,
            wide,
            addr,
            bytes_used,
        })
    }
}

#[derive(Debug)]
pub struct ParseInstructionError {
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
        match (
            bits[0], bits[1], bits[2], bits[3], bits[4], bits[5], bits[6],
        ) {
            (true, false, false, false, true, false, _) => {
                Self::try_parse_register_memory_mov(bits)
            }
            (true, false, true, true, _, _, _) => Self::try_parse_immediate_register_mov(bits),
            // NOTE: we may need 7 bits for this one
            (true, true, false, false, false, true, true) => {
                Self::try_parse_immediate_register_memory_mov(bits)
            }
            (true, false, true, false, false, false, _) => Self::try_parse_memory_accum_mov(bits),
            _ => unimplemented!("This opcode is unimplemented: {:?}", bits),
        }
    }
}
