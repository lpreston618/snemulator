use crate::utils::{GetBytes, SetBytes};

/// Implementation of the S-CPU's contained multiplication and division circuit.
pub struct Mult5A22 {
    mult_factor1: u8,
    div_numer: u16,
    div_quotient: u16,
    result: u16,
}

impl Mult5A22 {
    pub fn new() -> Mult5A22 {
        Mult5A22 { 
            mult_factor1: 0, 
            div_numer: 0, 
            div_quotient: 0, 
            result: 0,
        }
    }

    pub fn set_factor1(&mut self, data: u8) {
        self.mult_factor1 = data;
    }

    pub fn set_factor2(&mut self, data: u8) {
        self.result = (self.mult_factor1 as u16) * (data as u16);
    }

    pub fn set_numer_lo(&mut self, data: u8) {
        self.div_numer.set_lo(data);
    }

    pub fn set_numer_hi(&mut self, data: u8) {
        self.div_numer.set_hi(data);
    }

    pub fn set_denom(&mut self, data: u8) {
        if data == 0 {
            self.div_quotient = 0xFFFF;
            self.result = self.div_numer;
        } else {
            self.div_quotient = self.div_numer / (data as u16);
            self.result = self.div_numer % (data as u16);
        }
    }

    pub fn get_result_lo(&self) -> u8 { self.result.get_lo() }
    pub fn get_result_hi(&self) -> u8 { self.result.get_hi() }
    pub fn get_quotient_result_lo(&self) -> u8 { self.div_quotient.get_lo() }
    pub fn get_quotient_result_hi(&self) -> u8 { self.div_quotient.get_hi() }
}