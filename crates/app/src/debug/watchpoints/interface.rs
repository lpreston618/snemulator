use mlua::UserData;
use snemcore::{Snemulator, probe::DebugProbe, scpu};

pub struct SnemulatorInterface<'a, P: DebugProbe> {
    core: &'a Snemulator<P>
}

pub struct MetaInterface<'a, P: DebugProbe> {
    core: &'a Snemulator<P>
}

pub struct CpuInterface<'a, P: DebugProbe> {
    core: &'a Snemulator<P>
}

pub struct PpuInterface<'a, P: DebugProbe> {
    core: &'a Snemulator<P>
}

pub struct DmaInterface<'a, P: DebugProbe> {
    core: &'a Snemulator<P>
}

impl<'a, P: DebugProbe> SnemulatorInterface<'a, P> {
    pub fn new(core: &'a Snemulator<P>) -> Self {
        Self { core }
    }
}

impl<'a, P: DebugProbe> UserData for SnemulatorInterface<'a, P> {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("meta", |lua, this| {
            lua.scope(|scope| {
                scope.create_userdata(MetaInterface { core: &this.core })
            })
        });
        fields.add_field_method_get("cpu", |lua, this| {
            lua.scope(|scope| {
                scope.create_userdata(CpuInterface { core: &this.core })
            })
        });
        fields.add_field_method_get("ppu", |lua, this| {
            lua.scope(|scope| {
                scope.create_userdata(PpuInterface { core: &this.core })
            })
        });
    }
}

impl<'a, P: DebugProbe> UserData for MetaInterface<'a, P> {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("frame", |_, this| Ok(this.core.frame));
        fields.add_field_method_get("rom_size", |_, this| Ok(this.core.cart.as_ref().unwrap().rom_slice().len()));
    }
}

impl<'a, P: DebugProbe> UserData for CpuInterface<'a, P> {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("pb", |_, this| Ok(this.core.cpu.pb));
        fields.add_field_method_get("db", |_, this| Ok(this.core.cpu.db));
        fields.add_field_method_get("p", |_, this| Ok(this.core.cpu.p));
        fields.add_field_method_get("apuio0", |_, this| Ok(this.core.apu_ports.cpuio0));
        fields.add_field_method_get("apuio1", |_, this| Ok(this.core.apu_ports.cpuio1));
        fields.add_field_method_get("apuio2", |_, this| Ok(this.core.apu_ports.cpuio2));
        fields.add_field_method_get("apuio3", |_, this| Ok(this.core.apu_ports.cpuio3));
        fields.add_field_method_get("prg0", |_, this| Ok(this.core.cpu_read_mem(scpu::Address{ bank: this.core.cpu.pb, offset: this.core.cpu.pc + 0 })));
        fields.add_field_method_get("prg1", |_, this| Ok(this.core.cpu_read_mem(scpu::Address{ bank: this.core.cpu.pb, offset: this.core.cpu.pc + 1 })));
        fields.add_field_method_get("prg2", |_, this| Ok(this.core.cpu_read_mem(scpu::Address{ bank: this.core.cpu.pb, offset: this.core.cpu.pc + 2 })));
        fields.add_field_method_get("a", |_, this| Ok(this.core.cpu.a));
        fields.add_field_method_get("x", |_, this| Ok(this.core.cpu.x));
        fields.add_field_method_get("y", |_, this| Ok(this.core.cpu.y));
        fields.add_field_method_get("sp", |_, this| Ok(this.core.cpu.sp));
        fields.add_field_method_get("pc", |_, this| Ok(this.core.cpu.pc));
        fields.add_field_method_get("dp", |_, this| Ok(this.core.cpu.dp));
        fields.add_field_method_get("flagc", |_, this| Ok(this.core.cpu.is_flag_set(scpu::Flag::FlagC)));
        fields.add_field_method_get("flagz", |_, this| Ok(this.core.cpu.is_flag_set(scpu::Flag::FlagZ)));
        fields.add_field_method_get("flagi", |_, this| Ok(this.core.cpu.is_flag_set(scpu::Flag::FlagI)));
        fields.add_field_method_get("flagd", |_, this| Ok(this.core.cpu.is_flag_set(scpu::Flag::FlagD)));
        fields.add_field_method_get("flagx", |_, this| Ok(this.core.cpu.is_flag_set(scpu::Flag::FlagX)));
        fields.add_field_method_get("flagm", |_, this| Ok(this.core.cpu.is_flag_set(scpu::Flag::FlagM)));
        fields.add_field_method_get("flagv", |_, this| Ok(this.core.cpu.is_flag_set(scpu::Flag::FlagV)));
        fields.add_field_method_get("flagn", |_, this| Ok(this.core.cpu.is_flag_set(scpu::Flag::FlagN)));
        fields.add_field_method_get("e", |_, this| Ok(this.core.cpu.e));
        fields.add_field_method_get("halted", |_, this| Ok(this.core.cpu.halted));
        fields.add_field_method_get("stopped", |_, this| Ok(this.core.cpu.stopped));
        fields.add_field_method_get("nmi_pending", |_, this| Ok(this.core.cpu.nmi_pending));
        fields.add_field_method_get("irq_pending", |_, this| Ok(this.core.cpu.irq_pending));
        fields.add_field_method_get("waiting", |_, this| Ok(this.core.cpu.waiting_for_interrupt));
        fields.add_field_method_get("full_pc", |_, this| {
            Ok(scpu::Address {
                bank: this.core.cpu.pb,
                offset: this.core.cpu.pc,
            }.to_u32())
        });
    }
}

impl<'a, P: DebugProbe> UserData for PpuInterface<'a, P> {

}