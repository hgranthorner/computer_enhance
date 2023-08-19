use std::fmt::Display;

#[derive(Copy, Clone, Debug)]
pub enum Register {
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
