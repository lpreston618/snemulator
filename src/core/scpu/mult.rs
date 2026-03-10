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
    pub fn power_on(&mut self) {
        self.mult_factor1 = 0xFF;
        self.mult_factor2 = 0;
        self.div_numer = 0xFFFF;
        self.div_denom = 0;
        self.div_quotient = 0;
        self.result = 0;
    }
}
