// TODO: Implement Extended headers (0x33 in Dev ID. spot)


#[derive(Debug)]
pub enum MappingMode {
    LoROM,
    HiROM,
    ExHiROM,
    Unimplemented,
}

pub struct Header {
    title: [u8; 0x15],
    
    fast_rom: bool,
    map_mode: MappingMode,

    extra_ram: bool,
    battery: bool,
    coprocessor: bool,
    coprocessor_id: u8,

    rom_size: u8, // ROM size is (1 << rom_size) kb

    ram_size: u8, // RAM size is (1 << ram_size) kb

    is_ntsc: bool,

    interrupt_vectors: [u8; 32]   
}

impl Header {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut title: [u8; 0x15] = [0; 0x15];
        title.clone_from_slice(&bytes[..0x15]);

        let fast_rom = (bytes[0x15] & 0x10) > 0;
        let map_mode = match bytes[0x15] & 0x0F {
            0 => MappingMode::LoROM,
            1 => MappingMode::HiROM,
            5 => MappingMode::ExHiROM,
            _ => MappingMode::Unimplemented,
        };

        let (extra_ram, battery, coprocessor) = match bytes[0x16] & 0x0F {
            0 => (false, false, false), // $00 - ROM only
            1 => (true, false, false),  // $01 - ROM + RAM
            2 => (true, true, false),   // $02 - ROM + RAM + battery
            3 => (false, false, true),  // $x3 - ROM + coprocessor
            4 => (true, false, true),   // $x4 - ROM + coprocessor + RAM
            5 => (true, true, true),    // $x5 - ROM + coprocessor + RAM + battery
            6 => (false, true, true),   // $x6 - ROM + coprocessor + battery

            _ => (false, false, false) // Should not happen
        };
        let coprocessor_id = bytes[0x16] >> 4;

        let rom_size = bytes[0x17];

        let ram_size = bytes[0x18];

        let is_ntsc = bytes[0x19] > 0;

        let mut interrupt_vectors: [u8; 0x20] = [0; 0x20];
        interrupt_vectors.clone_from_slice(&bytes[0x20..0x40]);

        Self {
            title,
            fast_rom,
            map_mode,
            extra_ram,
            battery,
            coprocessor,
            coprocessor_id,
            rom_size,
            ram_size,
            is_ntsc,
            interrupt_vectors,
        }
    }

    pub fn print(&self) {
        println!("title: {:?}", std::str::from_utf8(&self.title).unwrap());
        println!("fast_rom: {:?}", self.fast_rom);
        println!("map_mode: {:?}", self.map_mode);
        println!("extra_ram: {:?}", self.extra_ram);
        println!("battery: {:?}", self.battery);
        println!("coprocessor: {:?}", self.coprocessor);
        println!("coprocessor_id: {:?}", self.coprocessor_id);
        println!("rom_size: {:?}", self.rom_size);
        println!("ram_size: {:?}", self.ram_size);
        println!("is_ntsc: {:?}", self.is_ntsc);
        println!("interrupt_vectors: {:?}", self.interrupt_vectors);
    }
}