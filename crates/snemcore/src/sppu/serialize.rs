use serde::{Serialize, ser::SerializeStruct};

use crate::sppu::Ppu5C7x;

impl Serialize for Ppu5C7x {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let mut s = serializer.serialize_struct("sppu", 8)?;
        s.serialize_field("dot", &self.dot)?;
        s.serialize_field("scanline", &self.scanline)?;
        s.serialize_field("x", &self.x)?;
        s.serialize_field("y", &self.y)?;
        s.serialize_field("frame", &self.frame)?;
        s.serialize_field("in_w1", &self.in_w1)?;
        s.serialize_field("in_w2", &self.in_w2)?;
        s.serialize_field("clocks", &self.clocks)?;
        s.end()
    }
}