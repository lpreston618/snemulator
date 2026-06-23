/// Shared registers between the S-CPU and SPC700
#[derive(Clone, Copy, Default)]
pub struct ApuIoPorts {
    /// SPC700 -> S-CPU register 0
    pub apuio0: u8,
    /// SPC700 -> S-CPU register 1
    pub apuio1: u8,
    /// SPC700 -> S-CPU register 2
    pub apuio2: u8,
    /// SPC700 -> S-CPU register 3
    pub apuio3: u8,

    /// S-CPU -> SPC700 register 0
    pub cpuio0: u8,
    /// S-CPU -> SPC700 register 1
    pub cpuio1: u8,
    /// S-CPU -> SPC700 register 2
    pub cpuio2: u8,
    /// S-CPU -> SPC700 register 3
    pub cpuio3: u8,
}

impl ApuIoPorts {
    pub fn power_on(&mut self) {
        self.apuio0 = 0;
        self.apuio1 = 0;
        self.apuio2 = 0;
        self.apuio3 = 0;
        
        self.cpuio0 = 0;
        self.cpuio1 = 0;
        self.cpuio2 = 0;
        self.cpuio3 = 0;
    }
    
    pub fn reset(&mut self) {}
}