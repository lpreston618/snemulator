mod cartridge;
mod scpu;

pub struct Config {
    pub rom_path: String,
}

pub fn run(config: Config) {
    println!("ROM Path: {}", config.rom_path);

    let rom_path = std::path::Path::new(&config.rom_path);

    let cartridge = cartridge::Cartridge::from_path(&rom_path);

    match cartridge {
        Ok(_) => { println!("Finished :)"); },
        Err(err) => {println!("Errored >:( with message {err}"); }
    };
}