pub(super) struct SuperDSP {

}

impl SuperDSP {
    pub fn new() -> SuperDSP {
        SuperDSP {  }
    }
}

impl SuperDSP {
    pub fn read_regs(&mut self, address: u8) -> u8 {
        println!("Read SDSP reg at ${address:02X}");

        0
    }

    pub fn write_regs(&mut self, address: u8, data: u8) {
        println!("Write SDSP reg ${address:02X} with 0x{data:02X}");
    }
}