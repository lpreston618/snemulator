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