use std::path::Path;

use crate::header::Header;

pub struct Cartridge {
    header: Header,
    rom_path: Path,
}

impl Cartridge {
    pub fn from_path(path: Path) -> Self {

    }
}