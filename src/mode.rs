#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Memory,
    Displace8Bits,
    Displace16Bits,
    Register,
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
