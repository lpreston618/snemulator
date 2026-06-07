use rand::{Rng, rngs::StdRng};

#[macro_export]
macro_rules! set_byte_n {
    ($var:expr, $val:expr, $n:expr) => {
        let val = $val;
        let mask = 0xFF << ($n * 8);
        $var = ($var & !mask) | ((val & 0xFF) << ($n * 8));
    };
}

#[macro_export]
macro_rules! get_byte_n {
    ($var:expr, $n:expr) => {
        (($var >> ($n * 8)) & 0xFF) as u8
    };
}

#[macro_export]
macro_rules! set_bit_n {
    ($var:expr, $n:expr) => {
        $var |= 1 << $n;
    };
}

#[macro_export]
macro_rules! clr_bit_n {
    ($var:expr, $n:expr) => {
        $var &= !(1 << $n);
    };
}

#[macro_export]
macro_rules! get_bit_n {
    ($var:expr, $n:expr) => {
        (($var >> $n) & 1) != 0
    };
}

pub trait RandomExt {
    fn rand_bool(&mut self) -> bool {
        self.rand_byte() & 1 != 0
    }
    fn rand_byte(&mut self) -> u8;
    fn rand_word(&mut self) -> u16;
}

impl RandomExt for StdRng {
    fn rand_byte(&mut self) -> u8 {
        self.next_u32() as u8
    }

    fn rand_word(&mut self) -> u16 {
        self.next_u32() as u16
    }
}