use mlua::{UserData, Value};
use snemcore::{Snemulator, probe::DebugProbe, scpu::{self, dma::Direction}, sppu::TileSize};

use crate::debug::{debugger::Debugger, watchpoints::engine::WatchpointControls};

macro_rules! register_fields {
    (
        $fields:expr,
        $(
            // Getter: get "name" => |this, core| expression;
            get $g_name:expr => |$g_this:ident, $g_core:ident| $g_expr:expr;
        )*
        $(
            // Setter: set "name" => |this, core, value: type| expression;
            set $s_name:expr => |$s_this:ident, $s_core:ident, $val:ident : $ty:ty| $s_expr:expr;
        )*
    ) => {
        $(
            $fields.add_field_method_get($g_name, |_, $g_this| {
                let $g_core = unsafe { &*$g_this.core };
                Ok($g_expr)
            });
        )*
        $(
            $fields.add_field_method_set($s_name, |_, $s_this, $val: $ty| {
                let $s_core = unsafe { &mut *$s_this.core };
                $s_expr;
                Ok(())
            });
        )*
    };
}

pub struct ControlInterface {
    controls: *mut WatchpointControls,
}

impl ControlInterface {
    pub fn new(controls: &mut WatchpointControls) -> Self {
        Self { controls: controls as *mut WatchpointControls }
    }
}

pub struct SnemulatorInterface {
    core: *mut Snemulator<Debugger>
}

pub struct MetaInterface {
    core: *mut Snemulator<Debugger>
}

pub struct CpuInterface {
    core: *mut Snemulator<Debugger>
}

pub struct PpuInterface {
    core: *mut Snemulator<Debugger>
}

pub struct DmaInterface {
    core: *mut Snemulator<Debugger>,
    channel: usize,
}

impl SnemulatorInterface {
    pub fn new(core: &mut Snemulator<Debugger>) -> Self {
        Self { core }
    }
}

impl UserData for ControlInterface {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("Break", |_, this, ()| {
            let controls = unsafe { &mut *this.controls };
            controls.watchpoint_hit = true;
            Ok(())
        });
        methods.add_method("Reset", |_, this, ()| {
            let controls = unsafe { &mut *this.controls };
            controls.should_reset = true;
            Ok(())
        });
        methods.add_method("SetAudioEnabled", |_, this, enabled: bool| {
            let controls = unsafe { &mut *this.controls };
            controls.audio_enable_cmd = Some(enabled);
            Ok(())
        });
        methods.add_method("SetVideoEnabled", |_, this, enabled: bool| {
            let controls = unsafe { &mut *this.controls };
            controls.video_enable_cmd = Some(enabled);
            Ok(())
        });
        methods.add_method("SetFastForwardEnabled", |_, this, enabled: bool| {
            let controls = unsafe { &mut *this.controls };
            controls.ff_enable_cmd = Some(enabled);
            Ok(())
        });
        methods.add_method("SetInputEnabled", |_, this, enabled: bool| {
            let controls = unsafe { &mut *this.controls };
            controls.input_enable_cmd = Some(enabled);
            Ok(())
        });
    }
}

impl UserData for SnemulatorInterface {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("meta", |lua, this| {
            lua.create_userdata(MetaInterface { core: this.core })
        });
        
        fields.add_field_method_get("cpu", |lua, this| {
            lua.create_userdata(CpuInterface { core: this.core })
        });
        
        fields.add_field_method_get("ppu", |lua, this| {
            lua.create_userdata(PpuInterface { core: this.core })
        });
        
        fields.add_field_method_get("dma", |lua, this| {
            let dma_table = lua.create_table()?;
            let dma_meta = lua.create_table()?;
            
            let core = this.core;
            
            dma_meta.set("__index", lua.create_function(move |lua, (_table, channel): (mlua::Table, usize)| {
                if channel > 7 {
                    Ok(Value::Nil)
                } else {
                    let dma_interface = lua.create_userdata(DmaInterface { 
                        core,
                        channel,
                    })?;
                    Ok(Value::UserData(dma_interface))
                }
            })?)?;
            
            dma_table.set_metatable(Some(dma_meta)).ok();
            
            Ok(dma_table)
        });
    }
}

impl UserData for MetaInterface {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        register_fields! { fields,
            get "frame"  => |_this, core| core.frame;
            get "rom_size" => |_this, core| core.cart.as_ref().unwrap().rom_slice().len();
        }
    }
}

impl UserData for CpuInterface {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        register_fields! { fields,
            // --- GETTERS ---
            get "pb"     => |_this, core| core.cpu.pb;
            get "db"     => |_this, core| core.cpu.db;
            get "p"      => |_this, core| core.cpu.p;
            get "apuio0" => |_this, core| core.apu_ports.cpuio0;
            get "apuio1" => |_this, core| core.apu_ports.cpuio1;
            get "apuio2" => |_this, core| core.apu_ports.cpuio2;
            get "apuio3" => |_this, core| core.apu_ports.cpuio3;
            
            get "a"      => |_this, core| core.cpu.a;
            get "x"      => |_this, core| core.cpu.x;
            get "y"      => |_this, core| core.cpu.y;
            get "sp"     => |_this, core| core.cpu.sp;
            get "pc"     => |_this, core| core.cpu.pc;
            get "dp"     => |_this, core| core.cpu.dp;
            
            get "flagc"  => |_this, core| core.cpu.is_flag_set(scpu::Flag::FlagC);
            get "flagz"  => |_this, core| core.cpu.is_flag_set(scpu::Flag::FlagZ);
            get "flagi"  => |_this, core| core.cpu.is_flag_set(scpu::Flag::FlagI);
            get "flagd"  => |_this, core| core.cpu.is_flag_set(scpu::Flag::FlagD);
            get "flagx"  => |_this, core| core.cpu.is_flag_set(scpu::Flag::FlagX);
            get "flagm"  => |_this, core| core.cpu.is_flag_set(scpu::Flag::FlagM);
            get "flagv"  => |_this, core| core.cpu.is_flag_set(scpu::Flag::FlagV);
            get "flagn"  => |_this, core| core.cpu.is_flag_set(scpu::Flag::FlagN);
            get "e"      => |_this, core| core.cpu.e;
            get "halted" => |_this, core| core.cpu.halted;
            get "stopped" => |_this, core| core.cpu.stopped;
            get "nmi_pending" => |_this, core| core.cpu.nmi_pending;
            get "irq_pending" => |_this, core| core.cpu.irq_pending;
            get "waiting" => |_this, core| core.cpu.waiting_for_interrupt;
            
            get "prg0"   => |_this, core| core.cpu_read_mem(scpu::Address { bank: core.cpu.pb, offset: core.cpu.pc + 0 });
            get "prg1"   => |_this, core| core.cpu_read_mem(scpu::Address { bank: core.cpu.pb, offset: core.cpu.pc + 1 });
            get "prg2"   => |_this, core| core.cpu_read_mem(scpu::Address { bank: core.cpu.pb, offset: core.cpu.pc + 2 });
            get "prg3"   => |_this, core| core.cpu_read_mem(scpu::Address { bank: core.cpu.pb, offset: core.cpu.pc + 3 });
            get "full_pc"=> |_this, core| scpu::Address { bank: core.cpu.pb, offset: core.cpu.pc }.to_u32();
            
            // --- SETTERS ---
            set "pb"     => |_this, core, value: u8| core.cpu.pb = value;
            set "db"     => |_this, core, value: u8| core.cpu.db = value;
            set "p"      => |_this, core, value: u8| core.cpu.p = value;
            set "apuio0" => |_this, core, value: u8| core.apu_ports.cpuio0 = value;
            set "apuio1" => |_this, core, value: u8| core.apu_ports.cpuio1 = value;
            set "apuio2" => |_this, core, value: u8| core.apu_ports.cpuio2 = value;
            set "apuio3" => |_this, core, value: u8| core.apu_ports.cpuio3 = value;
            
            set "a"      => |_this, core, value: u16| core.cpu.a = value;
            set "x"      => |_this, core, value: u16| core.cpu.x = value;
            set "y"      => |_this, core, value: u16| core.cpu.y = value;
            set "sp"     => |_this, core, value: u16| {
                if core.cpu.e {
                    core.cpu.sp = 0x100 | (value & 0xFF);
                } else {
                    core.cpu.sp = value;
                }
            };
            set "pc"     => |_this, core, value: u16| core.cpu.pc = value;
            set "dp"     => |_this, core, value: u16| core.cpu.dp = value;
            
            set "flagc"  => |_this, core, value: bool| core.cpu.set_flag_to_bool(scpu::Flag::FlagC, value);
            set "flagz"  => |_this, core, value: bool| core.cpu.set_flag_to_bool(scpu::Flag::FlagZ, value);
            set "flagi"  => |_this, core, value: bool| core.cpu.set_flag_to_bool(scpu::Flag::FlagI, value);
            set "flagd"  => |_this, core, value: bool| core.cpu.set_flag_to_bool(scpu::Flag::FlagD, value);
            set "flagx"  => |_this, core, value: bool| core.cpu.set_flag_to_bool(scpu::Flag::FlagX, value | core.cpu.e);
            set "flagm"  => |_this, core, value: bool| core.cpu.set_flag_to_bool(scpu::Flag::FlagM, value | core.cpu.e);
            set "flagv"  => |_this, core, value: bool| core.cpu.set_flag_to_bool(scpu::Flag::FlagV, value);
            set "flagn"  => |_this, core, value: bool| core.cpu.set_flag_to_bool(scpu::Flag::FlagN, value);
            set "e"      => |_this, core, value: bool| {
                if value {
                    core.cpu.e = true;
                    core.cpu.set_flag_to_bool(scpu::Flag::FlagM, true);
                    core.cpu.set_flag_to_bool(scpu::Flag::FlagX, true);
                    core.cpu.sp &= 0xFF;
                    core.cpu.sp |= 0x100;
                } else {
                    core.cpu.e = false;
                }
            };
            set "halted" => |_this, core, value: bool| core.cpu.halted = value;
            set "nmi_pending" => |_this, core, value: bool| core.cpu.nmi_pending = value;
            set "irq_pending" => |_this, core, value: bool| core.cpu.irq_pending = value;
            set "waiting" => |_this, core, value: bool| core.cpu.waiting_for_interrupt = value;
        }
    }
}

impl UserData for PpuInterface {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        register_fields! { fields,
            // --- GETTERS ---
            get "screen_brightness" => |_this, core| core.ppu_regs.screen_brightness;
            get "obj_size"          => |_this, core| core.ppu_regs.obj_sprite_size as u8;
            get "bg_mode"           => |_this, core| core.ppu_regs.bg_mode as u8;
            get "mosaic_size"       => |_this, core| core.ppu_regs.mosaic_size;
            get "cgram_addr"        => |_this, core| core.ppu_regs.cgram_addr;
            get "window1_left"      => |_this, core| core.ppu_regs.w1_left_pos;
            get "window1_right"     => |_this, core| core.ppu_regs.w1_right_pos;
            get "window2_left"      => |_this, core| core.ppu_regs.w2_left_pos;
            get "window2_right"     => |_this, core| core.ppu_regs.w2_right_pos;
            get "name_base_addr"    => |_this, core| core.ppu_regs.name_base_addr;
            get "name_secondary_addr" => |_this, core| core.ppu_regs.name_secondary_base_addr;
            get "oam_addr"          => |_this, core| core.ppu_regs.internal_oam_addr;
            get "bg1_tilemap_addr"  => |_this, core| core.ppu_regs.bg_settings[0].tilemap_base_addr;
            get "bg2_tilemap_addr"  => |_this, core| core.ppu_regs.bg_settings[1].tilemap_base_addr;
            get "bg3_tilemap_addr"  => |_this, core| core.ppu_regs.bg_settings[2].tilemap_base_addr;
            get "bg4_tilemap_addr"  => |_this, core| core.ppu_regs.bg_settings[3].tilemap_base_addr;
            get "bg1_hofs"          => |_this, core| core.ppu_regs.bg_settings[0].scroll_x;
            get "bg1_vofs"          => |_this, core| core.ppu_regs.bg_settings[0].scroll_y;
            get "bg2_hofs"          => |_this, core| core.ppu_regs.bg_settings[1].scroll_x;
            get "bg2_vofs"          => |_this, core| core.ppu_regs.bg_settings[1].scroll_y;
            get "bg3_hofs"          => |_this, core| core.ppu_regs.bg_settings[2].scroll_x;
            get "bg3_vofs"          => |_this, core| core.ppu_regs.bg_settings[2].scroll_y;
            get "bg4_hofs"          => |_this, core| core.ppu_regs.bg_settings[3].scroll_x;
            get "bg4_vofs"          => |_this, core| core.ppu_regs.bg_settings[3].scroll_y;
            get "m7_hofs"           => |_this, core| core.ppu_regs.m7_scroll_x;
            get "m7_vofs"           => |_this, core| core.ppu_regs.m7_scroll_y;
            get "vram_addr"         => |_this, core| core.ppu_regs.vram_addr;
            get "m7_a"              => |_this, core| core.ppu_regs.m7_matrix_a;
            get "m7_b"              => |_this, core| core.ppu_regs.m7_matrix_b;
            get "m7_c"              => |_this, core| core.ppu_regs.m7_matrix_c;
            get "m7_d"              => |_this, core| core.ppu_regs.m7_matrix_d;
            get "m7_x"              => |_this, core| core.ppu_regs.m7_center_x;
            get "m7_y"              => |_this, core| core.ppu_regs.m7_center_y;
            get "h_counter"         => |_this, core| core.ppu_regs.h_counter;
            get "v_counter"         => |_this, core| core.ppu_regs.v_counter;
            get "priority_rotation" => |_this, core| core.ppu_regs.priority_rotation;
            get "bg1_large_tiles"   => |_this, core| matches!(core.ppu_regs.bg_settings[0].chr_size, TileSize::Size16x16);
            get "bg2_large_tiles"   => |_this, core| matches!(core.ppu_regs.bg_settings[1].chr_size, TileSize::Size16x16);
            get "bg3_large_tiles"   => |_this, core| matches!(core.ppu_regs.bg_settings[2].chr_size, TileSize::Size16x16);
            get "bg4_large_tiles"   => |_this, core| matches!(core.ppu_regs.bg_settings[3].chr_size, TileSize::Size16x16);
            get "bg3_mode1_priority" => |_this, core| core.ppu_regs.bg3_mode1_priority;
            get "bg1_mosaic_enable" => |_this, core| core.ppu_regs.bg_settings[0].mosaic_en;
            get "bg2_mosaic_enable" => |_this, core| core.ppu_regs.bg_settings[1].mosaic_en;
            get "bg3_mosaic_enable" => |_this, core| core.ppu_regs.bg_settings[2].mosaic_en;
            get "bg4_mosaic_enable" => |_this, core| core.ppu_regs.bg_settings[3].mosaic_en;
            get "bg1_main_enable"   => |_this, core| core.ppu_regs.bg_settings[0].main_en;
            get "bg2_main_enable"   => |_this, core| core.ppu_regs.bg_settings[1].main_en;
            get "bg3_main_enable"   => |_this, core| core.ppu_regs.bg_settings[2].main_en;
            get "bg4_main_enable"   => |_this, core| core.ppu_regs.bg_settings[3].main_en;
            get "obj_main_enable"   => |_this, core| core.ppu_regs.obj_settings.main_en;
            get "dot"               => |_this, core| core.ppu.dot;
            get "scanline"          => |_this, core| core.ppu.scanline;
            get "screen_x"          => |_this, core| core.ppu.x;
            get "screen_y"          => |_this, core| core.ppu.y;
            get "multiply_result"   => |_this, core| core.ppu_regs.multiply_result;
            get "f_blank"           => |_this, core| core.ppu_regs.in_fblank;
            get "v_blank"           => |_this, core| core.cpu_regs.vblank_flag;
            get "h_blank"           => |_this, core| core.cpu_regs.hblank_flag;
            get "v_blank_nmi"       => |_this, core| core.cpu_regs.vblank_nmi_en;
            get "hv_timer_mode"     => |_this, core| core.cpu_regs.hv_timer_irq_mode as u8;
        }
    }
}

impl UserData for DmaInterface {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        register_fields! { fields,
            get "dma_en"                   => |this, core| core.dma_regs[this.channel].dma_en;
            get "hdma_en"                  => |this, core| core.dma_regs[this.channel].hdma_en;
            get "addr_inc_mode"            => |this, core| core.dma_regs[this.channel].inc_mode as u8;
            get "transfer_pattern"         => |this, core| core.dma_regs[this.channel].transfer_pattern as u8;
            get "a_bus_bank"               => |this, core| core.dma_regs[this.channel].a_bus_addr.bank;
            get "hdma_table_start_bank"    => |this, core| core.dma_regs[this.channel].a_bus_addr.bank;
            get "hdma_indirect_table_bank" => |this, core| core.dma_regs[this.channel].hdma_indirect_table_addr.bank;
            get "hdma_scanline_counter"    => |this, core| core.dma_regs[this.channel].scanline_counter;
            get "unused_reg"               => |this, core| core.dma_regs[this.channel].unused;
            get "b_bus_addr"               => |this, core| 0x2100 | core.dma_regs[this.channel].b_bus_addr as u16;
            get "a_bus_offset"             => |this, core| core.dma_regs[this.channel].a_bus_addr.offset;
            get "hdma_table_start_offset"  => |this, core| core.dma_regs[this.channel].a_bus_addr.offset;
            get "hdma_indirect_table_offset"    => |this, core| core.dma_regs[this.channel].hdma_indirect_table_addr.offset;
            get "hdma_table_offset"             => |this, core| core.dma_regs[this.channel].hdma_table_offset;
            get "b_to_a"                        => |this, core| matches!(core.dma_regs[this.channel].direction, Direction::BtoA);
            get "indirect_hdma"                 => |this, core| core.dma_regs[this.channel].indirect_hdma;
            get "hdma_reload"                   => |this, core| core.dma_regs[this.channel].hdma_reload_flag;
            get "full_a_bus_addr"               => |this, core| core.dma_regs[this.channel].a_bus_addr.to_u32();
            get "full_hdma_table_start_addr"    => |this, core| core.dma_regs[this.channel].a_bus_addr.to_u32();
            get "full_hdma_table_addr"          => |this, core| (core.dma_regs[this.channel].a_bus_addr.bank as u32) << 16 | core.dma_regs[this.channel].hdma_table_offset as u32;
            get "full_hdma_indirect_table_addr" => |this, core| core.dma_regs[this.channel].hdma_indirect_table_addr.to_u32();
        }
    }
}