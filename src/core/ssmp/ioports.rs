/// Shared registers between the S-CPU and SPC700
pub struct ApuIORegs {
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

impl ApuIORegs {
    pub fn new() -> ApuIORegs {
        ApuIORegs {
            apuio0: 0, apuio1: 0, apuio2: 0, apuio3: 0,
            cpuio0: 0, cpuio1: 0, cpuio2: 0, cpuio3: 0,
        }
    }
}