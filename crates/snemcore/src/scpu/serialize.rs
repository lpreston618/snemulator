use serde::{Serialize, ser::SerializeStruct};

use crate::scpu::Cpu65c816;

impl Serialize for Cpu65c816 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let mut s = serializer.serialize_struct("scpu", 14)?;
        s.serialize_field("a", &self.a)?;
        s.serialize_field("x", &self.x)?;
        s.serialize_field("y", &self.y)?;
        s.serialize_field("sp", &self.sp)?;
        s.serialize_field("pc", &self.pc)?;
        s.serialize_field("pb", &self.pb)?;
        s.serialize_field("db", &self.db)?;
        s.serialize_field("dp", &self.dp)?;
        s.serialize_field("p", &self.p)?;
        s.serialize_field("e", &self.e)?;
        s.serialize_field("halted", &self.halted)?;
        s.serialize_field("stopped", &self.stopped)?;
        s.serialize_field("waiting_for_interrupt", &self.waiting_for_interrupt)?;
        s.serialize_field("clocks", &self.clocks)?;
        s.end()
    }
}