#![allow(dead_code)]

use std::cell::Cell;

pub(crate) trait GetBits {
    fn get_bit(self, bit: Self) -> Self;
    fn bit_en(self, bit: Self) -> bool;
}

macro_rules! get_bits_impl {
    ($type:ty) => {
        impl GetBits for $type {
            fn get_bit(self, bit: Self) -> Self { (self >> bit) & 1 }
            fn bit_en(self, bit: Self) -> bool { (self >> bit) & 1 != 0 }
        }
    };
}

pub(crate) trait GetBytes {
    fn get_hi(self) -> u8;
    fn get_lo(self) -> u8;
}

pub(crate) trait SetBytes {
    fn set_hi(&mut self, hi: u8);
    fn set_lo(&mut self, lo: u8);
}

pub(crate) trait SetCellBytes {
    fn set_hi(&self, hi: u8);
    fn set_lo(&self, lo: u8);
}

get_bits_impl!(u8);
get_bits_impl!(u16);
get_bits_impl!(u32);
get_bits_impl!(u64);
get_bits_impl!(usize);

get_bits_impl!(i8);
get_bits_impl!(i16);
get_bits_impl!(i32);
get_bits_impl!(i64);
get_bits_impl!(isize);

impl GetBytes for u16 {
    fn get_hi(self) -> u8 { (self >> 8) as u8 }
    fn get_lo(self) -> u8 { self as u8 }
}

impl SetBytes for u16 {
    fn set_hi(&mut self, hi: u8) { *self = ((hi as u16) << 8) | (*self & 0xFF); }
    fn set_lo(&mut self, lo: u8) { *self = (*self & 0xFF00) | (lo as u16); }
}

impl SetCellBytes for Cell<u16> {
    fn set_hi(&self, data: u8) {
        self.set((self.get() & 0x00FF) | ((data as u16) << 8));
    }
    fn set_lo(&self, data: u8) {
        self.set((self.get() & 0xFF00) | (data as u16));
    }
}

pub(crate) fn inc_low_byte(value: u16) -> u16 {
    (value & 0xFF00) | ((value + 1) & 0x00FF)
}
pub(crate) fn dec_low_byte(value: u16) -> u16 {
    (value & 0xFF00) | ((value - 1) & 0x00FF)
}

pub(crate) mod util_macros {
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