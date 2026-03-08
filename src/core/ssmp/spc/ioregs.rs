pub struct SpcIoRegs {
    // $F1
    pub ipl_read_en: bool,
    
    // $F2
    pub sdsp_read_only: bool,
    pub sdsp_addr: u8,
}

impl SpcIoRegs {
    pub fn new() -> Self {
        Self {
            ipl_read_en: false,
            sdsp_read_only: false,
            sdsp_addr: 0,
        }
    }
}