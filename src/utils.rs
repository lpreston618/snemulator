pub trait GetBits {
    fn get_bit(self, bit: Self) -> Self;
    fn bit_en(self, bit: Self) -> bool;
}

impl GetBits for u8 {
    fn get_bit(self, bit: Self) -> Self { (self >> bit) & 1 }
    fn bit_en(self, bit: Self) -> bool { (self >> bit) & 1 != 0 }
}

impl GetBits for u16 {
    fn get_bit(self, bit: Self) -> Self { (self >> bit) & 1 }
    fn bit_en(self, bit: Self) -> bool { (self >> bit) & 1 != 0 }
}

pub fn inc_low_byte(value: u16) -> u16 {
    (value & 0xFF00) | ((value + 1) & 0x00FF)
}
pub fn dec_low_byte(value: u16) -> u16 {
    (value & 0xFF00) | ((value - 1) & 0x00FF)
}

pub mod util_macros {
    macro_rules! bool2byte {
        ($val:expr) => {
            if $val {
                1
            } else {
                0
            }
        };
    }

    pub(crate) use bool2byte;
}