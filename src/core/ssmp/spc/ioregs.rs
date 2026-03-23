#[derive(Default)]
pub struct SpcIoRegs {
    // $F1
    pub ipl_read_en: bool,
    
    // $F2
    pub sdsp_read_only: bool,
    pub sdsp_addr: u8,
}

impl SpcIoRegs {
    pub fn power_on(&mut self) {
        self.ipl_read_en = true;
        self.sdsp_read_only = false;
        self.sdsp_addr = 0;
    }
    
    pub fn reset(&mut self) {
        self.ipl_read_en = true;
        self.sdsp_read_only = false;
        self.sdsp_addr = 0;
    }
}