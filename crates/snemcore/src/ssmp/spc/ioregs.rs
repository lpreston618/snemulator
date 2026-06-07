#[derive(Default)]
pub struct SpcIoRegs {
    // $F0 (unused)

    // $F1
    pub ipl_read_en: bool,
    // clr cpuio ports
    // timer 0-2 enable in timer struct
    
    // $F2
    pub sdsp_read_only: bool,
    pub sdsp_addr: u8,

    // $F3 - DSP Read/Write

    // $F4-F7 CPUIO

    // $F8, $F9 - Normal RAM

    // $FA-FC - Timer 0-2 Targets

    // $FD-FF - Timer 0-2 Outputs
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