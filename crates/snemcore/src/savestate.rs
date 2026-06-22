use serde::{Serialize, ser::SerializeStruct};

use crate::Snemulator;

impl Serialize for Snemulator {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let mut s = serializer.serialize_struct("Snemulator", 21)?;
        s.serialize_field("cpu", &self.cpu)?;
        s.serialize_field("ppu", &self.ppu)?;
        s.serialize_field("ssmp", &self.ssmp)?;
        s.serialize_field("wram", &self.wram.as_slice())?;
        s.serialize_field("vram", &self.vram.as_slice())?;
        s.serialize_field("cgram", &self.cgram.as_slice())?;
        s.serialize_field("oam", &self.oam.as_slice())?;
        s.serialize_field("ppu_regs", &self.ppu_regs)?;
        s.serialize_field("cpu_regs", &self.cpu_regs)?;
        s.serialize_field("apu_ports", &self.apu_ports)?;
        s.serialize_field("cpu_open_bus", &self.cpu_open_bus)?;
        s.serialize_field("dma", &self.dma)?;
        s.serialize_field("controller_data", &self.controller_data)?;
        s.serialize_field("frame_ready", &self.frame_ready)?;
        s.serialize_field("cart", &self.cart)?;
        s.end()
    }
}