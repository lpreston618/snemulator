pub trait GetBits {
    fn get_bit(self, bit: Self) -> Self;
    fn bit_en(self, bit: Self) -> bool;
}

impl GetBits for u8 {
    fn get_bit(self, bit: Self) -> Self { (self >> bit) & 1 }
    fn bit_en(self, bit: Self) -> bool { (self >> bit) & 1 != 0 }
}