/// Implementation of the S-CPU's contained multiplication and division circuit.
#[derive(Default)]
pub struct Mult5A22 {
    pub mult_factor1: u8,
    pub mult_factor2: u8,
    pub div_numer: u16,
    pub div_denom: u8,
    
    pub div_quotient: u16,
    pub result: u16,
}

impl Mult5A22 {
    pub fn new() -> Mult5A22 {
        Mult5A22 { 
            mult_factor1: 0xFF, 
            mult_factor2: 0,
            div_numer: 0xFFFF, 
            div_denom: 0,
            div_quotient: 0, 
            result: 0,
        }
    }
}
